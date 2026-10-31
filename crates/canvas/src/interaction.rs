//! Pure input-state controller for the canvas viewport.
//!
//! All positions accepted by this module are logical pixels relative to the
//! canvas viewport. The application layer converts framework coordinates before
//! constructing [`CanvasCommand`] values.

use std::collections::BTreeSet;

use crate::{Camera, CanvasDataset, Point, Rect};

const DRAG_THRESHOLD_SQUARED: f32 = 16.0;

/// A deterministic canvas-viewport input command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanvasCommand {
    BeginPan {
        viewport_position: Point,
    },
    ContinuePan {
        viewport_position: Point,
    },
    EndPan,
    PanBy {
        viewport_delta: Point,
    },
    BeginSelection {
        viewport_position: Point,
    },
    ContinueSelection {
        viewport_position: Point,
    },
    EndSelection {
        viewport_position: Point,
        extend_selection: bool,
    },
    ZoomAt {
        viewport_position: Point,
        scale_factor: f32,
    },
}

/// The pure canvas interaction state consumed by the application renderer.
#[derive(Clone, Debug)]
pub struct CanvasInteraction {
    camera: Camera,
    selected_ids: BTreeSet<u32>,
    gesture: Gesture,
}

#[derive(Clone, Copy, Debug)]
enum Gesture {
    Idle,
    Pan {
        previous_viewport_position: Point,
    },
    Selection {
        anchor_viewport_position: Point,
        anchor_world_position: Point,
        current_viewport_position: Point,
        is_dragging: bool,
    },
}

impl CanvasInteraction {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            selected_ids: BTreeSet::new(),
            gesture: Gesture::Idle,
        }
    }

    pub fn apply(&mut self, dataset: &CanvasDataset, command: CanvasCommand) {
        match command {
            CanvasCommand::BeginPan { viewport_position } => {
                self.gesture = Gesture::Pan {
                    previous_viewport_position: viewport_position,
                };
            }
            CanvasCommand::ContinuePan { viewport_position } => {
                if let Gesture::Pan {
                    previous_viewport_position,
                } = &mut self.gesture
                {
                    self.camera
                        .pan_by_screen(viewport_position - *previous_viewport_position);
                    *previous_viewport_position = viewport_position;
                }
            }
            CanvasCommand::EndPan => {
                if matches!(self.gesture, Gesture::Pan { .. }) {
                    self.gesture = Gesture::Idle;
                }
            }
            CanvasCommand::PanBy { viewport_delta } => {
                self.camera.pan_by_screen(viewport_delta);
            }
            CanvasCommand::BeginSelection { viewport_position } => {
                self.gesture = Gesture::Selection {
                    anchor_viewport_position: viewport_position,
                    anchor_world_position: self.camera.screen_to_canvas(viewport_position),
                    current_viewport_position: viewport_position,
                    is_dragging: false,
                };
            }
            CanvasCommand::ContinueSelection { viewport_position } => {
                if let Gesture::Selection {
                    anchor_viewport_position,
                    current_viewport_position,
                    is_dragging,
                    ..
                } = &mut self.gesture
                {
                    *is_dragging |=
                        exceeds_drag_threshold(*anchor_viewport_position, viewport_position);
                    *current_viewport_position = viewport_position;
                }
            }
            CanvasCommand::EndSelection {
                viewport_position,
                extend_selection,
            } => {
                let gesture = std::mem::replace(&mut self.gesture, Gesture::Idle);
                if let Gesture::Selection {
                    anchor_viewport_position,
                    anchor_world_position,
                    is_dragging,
                    ..
                } = gesture
                {
                    let is_dragging = is_dragging
                        || exceeds_drag_threshold(anchor_viewport_position, viewport_position);
                    let selected_ids: Vec<u32> = if is_dragging {
                        dataset
                            .cards_in_selection_rect(
                                anchor_world_position,
                                self.camera.screen_to_canvas(viewport_position),
                            )
                            .into_iter()
                            .map(|card| card.id)
                            .collect()
                    } else {
                        dataset
                            .hit_test(self.camera.screen_to_canvas(viewport_position))
                            .map(|card| card.id)
                            .into_iter()
                            .collect()
                    };
                    self.replace_or_extend_selection(selected_ids, extend_selection);
                } else {
                    self.gesture = gesture;
                }
            }
            CanvasCommand::ZoomAt {
                viewport_position,
                scale_factor,
            } => self.camera.zoom_at_screen(viewport_position, scale_factor),
        }
    }

    pub fn camera(&self) -> Camera {
        self.camera
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn center_world_point_in_viewport(&mut self, world_point: Point, viewport_size: Point) {
        self.camera
            .center_world_point_in_viewport(world_point, viewport_size);
    }

    pub fn is_selected(&self, card_id: u32) -> bool {
        self.selected_ids.contains(&card_id)
    }

    pub fn selected_card_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.selected_ids.iter().copied()
    }

    pub fn remove_selected_card_id(&mut self, card_id: u32) -> bool {
        self.selected_ids.remove(&card_id)
    }

    pub fn select_only_card_id(&mut self, card_id: u32) {
        self.selected_ids.clear();
        self.selected_ids.insert(card_id);
    }

    /// Adds one card to the current selection without changing the camera or
    /// other selected cards.
    pub fn add_selected_card_id(&mut self, card_id: u32) {
        self.selected_ids.insert(card_id);
    }

    pub fn marquee_viewport_rect(&self) -> Option<Rect> {
        match self.gesture {
            Gesture::Selection {
                anchor_viewport_position,
                current_viewport_position,
                is_dragging: true,
                ..
            } => Some(Rect::from_points(
                anchor_viewport_position,
                current_viewport_position,
            )),
            Gesture::Idle | Gesture::Pan { .. } | Gesture::Selection { .. } => None,
        }
    }

    fn replace_or_extend_selection(
        &mut self,
        selected_ids: impl IntoIterator<Item = u32>,
        extend_selection: bool,
    ) {
        if !extend_selection {
            self.selected_ids.clear();
        }
        self.selected_ids.extend(selected_ids);
    }
}

fn exceeds_drag_threshold(anchor: Point, current: Point) -> bool {
    let delta = current - anchor;
    delta.x * delta.x + delta.y * delta.y >= DRAG_THRESHOLD_SQUARED
}

#[cfg(test)]
mod tests {
    use super::{CanvasCommand, CanvasInteraction};
    use crate::{Camera, CanvasDataset, Point, Rect};

    fn dataset() -> CanvasDataset {
        CanvasDataset::generated()
    }

    fn interaction() -> CanvasInteraction {
        CanvasInteraction::new(Camera::new(Point::default(), 1.0))
    }

    fn selected_ids(interaction: &CanvasInteraction) -> Vec<u32> {
        interaction.selected_card_ids().collect()
    }

    fn click(
        interaction: &mut CanvasInteraction,
        dataset: &CanvasDataset,
        viewport_position: Point,
        extend_selection: bool,
    ) {
        interaction.apply(dataset, CanvasCommand::BeginSelection { viewport_position });
        interaction.apply(
            dataset,
            CanvasCommand::EndSelection {
                viewport_position,
                extend_selection,
            },
        );
    }

    #[test]
    fn pan_accumulates_viewport_deltas() {
        let dataset = dataset();
        let mut interaction = interaction();

        interaction.apply(
            &dataset,
            CanvasCommand::BeginPan {
                viewport_position: Point::new(10.0, 20.0),
            },
        );
        interaction.apply(
            &dataset,
            CanvasCommand::ContinuePan {
                viewport_position: Point::new(18.0, 25.0),
            },
        );
        interaction.apply(
            &dataset,
            CanvasCommand::ContinuePan {
                viewport_position: Point::new(15.0, 35.0),
            },
        );
        interaction.apply(&dataset, CanvasCommand::EndPan);

        assert_eq!(interaction.camera().offset, Point::new(5.0, 15.0));
        assert_eq!(interaction.marquee_viewport_rect(), None);
    }

    #[test]
    fn pan_stops_after_its_end_command() {
        let dataset = dataset();
        let mut interaction = interaction();

        interaction.apply(
            &dataset,
            CanvasCommand::BeginPan {
                viewport_position: Point::new(10.0, 10.0),
            },
        );
        interaction.apply(&dataset, CanvasCommand::EndPan);
        interaction.apply(
            &dataset,
            CanvasCommand::ContinuePan {
                viewport_position: Point::new(30.0, 30.0),
            },
        );

        assert_eq!(interaction.camera().offset, Point::default());
    }

    #[test]
    fn pan_by_applies_an_independent_trackpad_scroll_delta() {
        let dataset = dataset();
        let mut interaction = interaction();

        interaction.apply(
            &dataset,
            CanvasCommand::PanBy {
                viewport_delta: Point::new(18.0, -12.0),
            },
        );

        assert_eq!(interaction.camera().offset, Point::new(18.0, -12.0));
    }

    #[test]
    fn magnification_zoom_keeps_the_viewport_anchor_stable() {
        let dataset = dataset();
        let mut interaction = interaction();
        let anchor = Point::new(320.0, 180.0);
        let world_anchor = interaction.camera().screen_to_world(anchor);

        interaction.apply(
            &dataset,
            CanvasCommand::ZoomAt {
                viewport_position: anchor,
                scale_factor: 1.27,
            },
        );

        assert!(interaction.camera().scale > 1.0);
        assert_point_close(interaction.camera().world_to_screen(world_anchor), anchor);
    }

    #[test]
    fn centering_the_camera_keeps_selection_and_scale() {
        let dataset = dataset();
        let mut interaction = CanvasInteraction::new(Camera::new(Point::default(), 1.5));
        click(&mut interaction, &dataset, Point::new(10.0, 10.0), false);

        interaction
            .center_world_point_in_viewport(Point::new(200.0, 120.0), Point::new(1_280.0, 820.0));

        assert_eq!(selected_ids(&interaction), vec![0]);
        assert_eq!(interaction.camera().scale, 1.5);
        assert_point_close(
            interaction
                .camera()
                .world_to_screen(Point::new(200.0, 120.0)),
            Point::new(640.0, 410.0),
        );
    }

    #[test]
    fn restoring_a_camera_keeps_selection_unchanged() {
        let dataset = dataset();
        let mut interaction = interaction();
        click(&mut interaction, &dataset, Point::new(10.0, 10.0), false);
        let restored_camera = Camera::new(Point::new(-80.0, 140.0), 0.75);

        interaction.set_camera(restored_camera);

        assert_eq!(selected_ids(&interaction), vec![0]);
        assert_eq!(interaction.camera(), restored_camera);
    }

    #[test]
    fn click_replaces_the_existing_selection() {
        let dataset = dataset();
        let mut interaction = interaction();

        click(&mut interaction, &dataset, Point::new(10.0, 10.0), false);
        click(&mut interaction, &dataset, Point::new(230.0, 150.0), false);

        assert_eq!(selected_ids(&interaction), vec![101]);
        assert!(interaction.is_selected(101));
        assert!(!interaction.is_selected(0));
    }

    #[test]
    fn empty_click_clears_the_selection() {
        let dataset = dataset();
        let mut interaction = interaction();

        click(&mut interaction, &dataset, Point::new(10.0, 10.0), false);
        click(&mut interaction, &dataset, Point::new(190.0, 105.0), false);

        assert_eq!(selected_ids(&interaction), Vec::<u32>::new());
    }

    #[test]
    fn extended_click_extends_the_selection() {
        let dataset = dataset();
        let mut interaction = interaction();

        click(&mut interaction, &dataset, Point::new(10.0, 10.0), false);
        click(&mut interaction, &dataset, Point::new(230.0, 150.0), true);

        assert_eq!(selected_ids(&interaction), vec![0, 101]);
    }

    #[test]
    fn removing_a_selected_card_keeps_the_camera_unchanged() {
        let mut interaction = interaction();
        interaction.select_only_card_id(101);
        let camera = interaction.camera();

        assert!(interaction.remove_selected_card_id(101));
        assert!(!interaction.remove_selected_card_id(101));
        assert_eq!(selected_ids(&interaction), Vec::<u32>::new());
        assert_eq!(interaction.camera(), camera);
    }

    #[test]
    fn selecting_only_one_card_replaces_selection_without_moving_the_camera() {
        let dataset = dataset();
        let mut interaction = interaction();
        click(&mut interaction, &dataset, Point::new(10.0, 10.0), false);
        click(&mut interaction, &dataset, Point::new(230.0, 150.0), true);
        let camera = interaction.camera();

        interaction.select_only_card_id(77);

        assert_eq!(selected_ids(&interaction), vec![77]);
        assert_eq!(interaction.camera(), camera);
    }

    #[test]
    fn subthreshold_drag_is_a_click_without_a_marquee() {
        let dataset = dataset();
        let mut interaction = interaction();

        interaction.apply(
            &dataset,
            CanvasCommand::BeginSelection {
                viewport_position: Point::new(10.0, 10.0),
            },
        );
        interaction.apply(
            &dataset,
            CanvasCommand::ContinueSelection {
                viewport_position: Point::new(12.0, 12.0),
            },
        );

        assert_eq!(interaction.marquee_viewport_rect(), None);

        interaction.apply(
            &dataset,
            CanvasCommand::EndSelection {
                viewport_position: Point::new(12.0, 12.0),
                extend_selection: false,
            },
        );

        assert_eq!(selected_ids(&interaction), vec![0]);
    }

    #[test]
    fn forward_marquee_replaces_the_selection() {
        let dataset = dataset();
        let mut interaction = interaction();

        marquee(
            &mut interaction,
            &dataset,
            Point::new(10.0, 10.0),
            Point::new(430.0, 250.0),
            false,
        );

        assert_eq!(selected_ids(&interaction), vec![0, 1, 100, 101]);
    }

    #[test]
    fn reverse_marquee_normalizes_the_viewport_rectangle() {
        let dataset = dataset();
        let mut interaction = interaction();
        let first = Point::new(430.0, 250.0);
        let second = Point::new(10.0, 10.0);

        interaction.apply(
            &dataset,
            CanvasCommand::BeginSelection {
                viewport_position: first,
            },
        );
        interaction.apply(
            &dataset,
            CanvasCommand::ContinueSelection {
                viewport_position: second,
            },
        );

        assert_eq!(
            interaction.marquee_viewport_rect(),
            Some(Rect::from_points(first, second))
        );

        interaction.apply(
            &dataset,
            CanvasCommand::EndSelection {
                viewport_position: second,
                extend_selection: false,
            },
        );

        assert_eq!(selected_ids(&interaction), vec![0, 1, 100, 101]);
    }

    #[test]
    fn extended_marquee_extends_the_selection() {
        let dataset = dataset();
        let mut interaction = interaction();

        click(&mut interaction, &dataset, Point::new(450.0, 10.0), false);
        marquee(
            &mut interaction,
            &dataset,
            Point::new(10.0, 10.0),
            Point::new(430.0, 250.0),
            true,
        );

        assert_eq!(selected_ids(&interaction), vec![0, 1, 2, 100, 101]);
    }

    #[test]
    fn end_selection_does_not_terminate_an_active_pan() {
        let dataset = dataset();
        let mut interaction = interaction();

        interaction.apply(
            &dataset,
            CanvasCommand::BeginPan {
                viewport_position: Point::new(10.0, 10.0),
            },
        );
        interaction.apply(
            &dataset,
            CanvasCommand::EndSelection {
                viewport_position: Point::new(20.0, 20.0),
                extend_selection: false,
            },
        );
        interaction.apply(
            &dataset,
            CanvasCommand::ContinuePan {
                viewport_position: Point::new(15.0, 25.0),
            },
        );

        assert_eq!(interaction.camera().offset, Point::new(5.0, 15.0));
    }

    fn marquee(
        interaction: &mut CanvasInteraction,
        dataset: &CanvasDataset,
        first: Point,
        second: Point,
        extend_selection: bool,
    ) {
        interaction.apply(
            dataset,
            CanvasCommand::BeginSelection {
                viewport_position: first,
            },
        );
        interaction.apply(
            dataset,
            CanvasCommand::ContinueSelection {
                viewport_position: second,
            },
        );
        interaction.apply(
            dataset,
            CanvasCommand::EndSelection {
                viewport_position: second,
                extend_selection,
            },
        );
    }

    fn assert_point_close(actual: Point, expected: Point) {
        const EPSILON: f32 = 0.0001;

        assert!(
            (actual.x - expected.x).abs() < EPSILON,
            "x: expected {}, got {}",
            expected.x,
            actual.x
        );
        assert!(
            (actual.y - expected.y).abs() < EPSILON,
            "y: expected {}, got {}",
            expected.y,
            actual.y
        );
    }
}
