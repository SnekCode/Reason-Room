//! Pure domain and viewport logic for the infinite-note-canvas prototype.
//!
//! The application layer owns rendering. This crate owns deterministic data
//! generation, camera transforms, spatial queries, hit testing, and semantic
//! zoom decisions.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::{Add, AddAssign, Mul, Sub};

pub mod interaction;
pub mod routing;
pub use routing::RelationshipRouteCache;
pub mod workspace;

pub use interaction::{CanvasCommand, CanvasInteraction};
pub use workspace::{
    CardPasteMode, ConceptPlacement, CustomType, ExecutionMode, LensFilter, LensTemplate,
    MAXIMUM_ZOOM_PERCENT, MINIMUM_ZOOM_PERCENT, NEW_CARD_BASELINE_SIZE_AT_100_PERCENT,
    PlanningScenario, RelationshipKind, RelationshipOverlayPlacementOverrides,
    RelationshipOverlayTarget, SelectedConceptOverlaySource, SelectedConceptRelationshipOverlay,
    SelectedConceptRelationshipOverlayEntry, SelectedConceptRelationshipOverlayLine,
    SelectedConceptRelationshipOverlayPath, SemanticZoomCard, SemanticZoomCardLayerMode,
    SemanticZoomCardLayerModeEntry, SemanticZoomCardLayerVisibility, SemanticZoomFrame,
    SemanticZoomLayer, SemanticZoomLayerPercentSettings, SemanticZoomLayerProjection,
    SemanticZoomLayerSettings, SemanticZoomRelationship, TypedRelationship, VisualBoundary,
    WORKSPACE_SCHEMA_VERSION, WorkspaceConcept, WorkspaceDocument, WorkspaceError, WorkspaceLens,
    WorkspaceVersion, WorkspaceView, ZoomPercentRange, ZoomRange,
    relationship_path_around_obstacles,
};

pub const CONCEPT_COUNT: usize = 10_000;
pub const RELATIONSHIP_COUNT: usize = 50_000;
/// The smallest supported working scale: 1%.
///
/// This is a numerical-safety boundary, not a canvas boundary. At this scale
/// the renderer switches to its bounded overview representation.
pub const MIN_SCALE: f32 = 0.01;
/// The largest supported working scale: 409,600%.
///
/// This is a numerical-safety boundary, not a canvas boundary. It preserves
/// stable transforms while allowing deep inspection of a concept.
pub const MAX_SCALE: f32 = 4_096.0;
pub const WHEEL_ZOOM_SENSITIVITY: f32 = 0.0015;
pub const GRID_CELL_SIZE: f32 = 256.0;
pub const MAX_OVERVIEW_CLUSTERS: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Point,
}

impl Rect {
    pub const fn new(origin: Point, size: Point) -> Self {
        Self { origin, size }
    }

    pub fn from_points(first: Point, second: Point) -> Self {
        let minimum_x = first.x.min(second.x);
        let minimum_y = first.y.min(second.y);

        Self::new(
            Point::new(minimum_x, minimum_y),
            Point::new((first.x - second.x).abs(), (first.y - second.y).abs()),
        )
    }

    pub fn normalized(self) -> Self {
        Self::from_points(self.origin, self.origin + self.size)
    }

    pub fn max_x(self) -> f32 {
        self.origin.x + self.size.x
    }

    pub fn max_y(self) -> f32 {
        self.origin.y + self.size.y
    }

    pub fn is_empty(self) -> bool {
        self.size.x <= 0.0 || self.size.y <= 0.0
    }

    pub fn contains_point(self, point: Point) -> bool {
        let normalized = self.normalized();
        point.x >= normalized.origin.x
            && point.x < normalized.max_x()
            && point.y >= normalized.origin.y
            && point.y < normalized.max_y()
    }

    /// Returns true when this non-empty rectangle fully encloses another
    /// non-empty rectangle. Edges may touch; equal-size rectangles are not a
    /// containment relationship because neither rectangle is larger.
    pub fn fully_contains(self, other: Self) -> bool {
        let container = self.normalized();
        let contained = other.normalized();

        !container.is_empty()
            && !contained.is_empty()
            && container.size.x > contained.size.x
            && container.size.y > contained.size.y
            && container.origin.x <= contained.origin.x
            && container.origin.y <= contained.origin.y
            && container.max_x() >= contained.max_x()
            && container.max_y() >= contained.max_y()
    }

    pub fn intersects(self, other: Self) -> bool {
        let first = self.normalized();
        let second = other.normalized();

        !first.is_empty()
            && !second.is_empty()
            && first.origin.x < second.max_x()
            && first.max_x() > second.origin.x
            && first.origin.y < second.max_y()
            && first.max_y() > second.origin.y
    }

    pub fn center(self) -> Point {
        let normalized = self.normalized();
        normalized.origin + normalized.size * 0.5
    }
}

/// Resolves the immediate visual parent for each fully enclosed card.
///
/// This is spatial presentation only. It does not create a persisted
/// relationship or alter the workspace graph. When more than one card fully
/// encloses a child, the smallest enclosing card wins. Equal-area candidates
/// resolve by card ID to keep the result deterministic.
pub fn card_parent_ids(card_bounds: impl IntoIterator<Item = (u32, Rect)>) -> BTreeMap<u32, u32> {
    let mut card_bounds: Vec<_> = card_bounds
        .into_iter()
        .map(|(card_id, bounds)| (card_id, bounds.normalized()))
        .collect();
    card_bounds.sort_by_key(|(card_id, _)| *card_id);

    card_bounds
        .iter()
        .filter_map(|(child_id, child_bounds)| {
            card_bounds
                .iter()
                .filter(|(parent_id, parent_bounds)| {
                    parent_id != child_id && parent_bounds.fully_contains(*child_bounds)
                })
                .min_by(|(left_id, left_bounds), (right_id, right_bounds)| {
                    rect_area(*left_bounds)
                        .total_cmp(&rect_area(*right_bounds))
                        .then(left_id.cmp(right_id))
                })
                .map(|(parent_id, _)| (*child_id, *parent_id))
        })
        .collect()
}

/// Returns true when either card is an immediate or transitive visual parent
/// of the other card.
///
/// Card containment is Lens-local presentation. A relationship line between a
/// card and one of its containers is redundant and cannot be authored or
/// painted while that containment exists.
pub fn cards_are_in_containment_chain(
    parent_by_child: &BTreeMap<u32, u32>,
    first_card_id: u32,
    second_card_id: u32,
) -> bool {
    if first_card_id == second_card_id {
        return false;
    }

    [
        (first_card_id, second_card_id),
        (second_card_id, first_card_id),
    ]
    .into_iter()
    .any(|(descendant_id, ancestor_id)| {
        let mut current_card_id = descendant_id;
        while let Some(parent_id) = parent_by_child.get(&current_card_id) {
            if *parent_id == ancestor_id {
                return true;
            }
            current_card_id = *parent_id;
        }
        false
    })
}

/// Expands a card-move selection to include every fully enclosed descendant.
///
/// Cards can act as spatial containers. Moving a container keeps its nested
/// cards in the same spatial context, while an overlapping card remains
/// independent. The returned IDs are ordered for deterministic previews and
/// persistence.
pub fn card_move_ids_with_contained_descendants(
    card_bounds: impl IntoIterator<Item = (u32, Rect)>,
    selected_card_ids: impl IntoIterator<Item = u32>,
) -> Vec<u32> {
    let card_bounds = card_bounds.into_iter().collect::<Vec<_>>();
    let selected_card_ids = selected_card_ids.into_iter().collect::<BTreeSet<_>>();
    let parent_by_child = card_parent_ids(card_bounds.iter().copied());

    card_bounds
        .iter()
        .filter_map(|(card_id, _)| {
            let mut current_card_id = *card_id;
            loop {
                if selected_card_ids.contains(&current_card_id) {
                    return Some(*card_id);
                }
                current_card_id = *parent_by_child.get(&current_card_id)?;
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Identifies one movable canvas item without assigning graph meaning to a
/// Lens-local frame.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanvasItemId {
    Card(u32),
    Boundary(String),
}

/// The minimal geometry required to resolve canvas containment and movement.
///
/// The application adapts graph-backed cards and Lens-local boundary frames
/// into this shared form. The resolver keeps their interaction behavior in
/// one place while their persisted meaning remains separate.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasItemBounds {
    pub id: CanvasItemId,
    pub bounds: Rect,
}

/// Expands a selected set of canvas items to every fully enclosed descendant.
///
/// A selected card or boundary frame moves nested cards and boundary frames.
/// An overlapping item remains independent. The output is ordered so preview,
/// persistence, and tests produce the same result.
pub fn canvas_item_move_ids_with_contained_descendants(
    items: impl IntoIterator<Item = CanvasItemBounds>,
    selected_item_ids: impl IntoIterator<Item = CanvasItemId>,
) -> Vec<CanvasItemId> {
    let items = items.into_iter().collect::<Vec<_>>();
    let mut moved_item_ids = selected_item_ids.into_iter().collect::<BTreeSet<_>>();

    loop {
        let additional_item_ids = items
            .iter()
            .filter_map(|candidate| {
                (!moved_item_ids.contains(&candidate.id)
                    && items.iter().any(|container| {
                        moved_item_ids.contains(&container.id)
                            && container.bounds.fully_contains(candidate.bounds)
                    }))
                .then_some(candidate.id.clone())
            })
            .collect::<BTreeSet<_>>();
        if additional_item_ids.is_empty() {
            break;
        }
        moved_item_ids.extend(additional_item_ids);
    }

    moved_item_ids.into_iter().collect()
}

fn rect_area(bounds: Rect) -> f32 {
    let bounds = bounds.normalized();
    bounds.size.x * bounds.size.y
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// The screen-space offset in logical pixels.
    pub offset: Point,
    /// Screen pixels per world unit.
    pub scale: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Point::new(80.0, 80.0), 1.0)
    }
}

impl Camera {
    pub fn new(offset: Point, scale: f32) -> Self {
        Self {
            offset,
            scale: clamp_scale(scale),
        }
    }

    pub fn pan_by_screen(&mut self, delta: Point) {
        self.offset += delta;
    }

    pub fn screen_to_world(self, screen: Point) -> Point {
        (screen - self.offset) * (1.0 / self.scale)
    }

    pub fn world_to_screen(self, world: Point) -> Point {
        self.offset + world * self.scale
    }

    pub fn screen_to_canvas(self, screen: Point) -> Point {
        self.screen_to_world(screen)
    }

    pub fn canvas_to_screen(self, canvas: Point) -> Point {
        self.world_to_screen(canvas)
    }

    /// Zooms around a screen position while keeping the anchored world point
    /// stable under that position.
    pub fn zoom_at_screen(&mut self, screen_anchor: Point, scale_factor: f32) {
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return;
        }

        let world_anchor = self.screen_to_world(screen_anchor);
        self.scale = clamp_scale(self.scale * scale_factor);
        self.offset = screen_anchor - world_anchor * self.scale;
    }

    /// Applies a wheel delta where negative values zoom in and positive values
    /// zoom out. The world point below `screen_anchor` remains unchanged.
    pub fn zoom_by_wheel_at_screen(&mut self, screen_anchor: Point, wheel_delta_y: f32) {
        if !wheel_delta_y.is_finite() {
            return;
        }

        let scale_factor = (-wheel_delta_y * WHEEL_ZOOM_SENSITIVITY).exp();
        self.zoom_at_screen(screen_anchor, scale_factor);
    }

    pub fn center_world_point_in_viewport(&mut self, world_point: Point, viewport_size: Point) {
        self.offset = viewport_size * 0.5 - world_point * self.scale;
    }

    pub fn visible_world_rect(self, viewport_size: Point) -> Rect {
        let origin = self.screen_to_world(Point::default());
        let lower_right = self.screen_to_world(viewport_size);
        Rect::new(origin, lower_right - origin)
    }

    pub fn visible_canvas_rect(self, viewport_size: Point) -> Rect {
        self.visible_world_rect(viewport_size)
    }
}

fn clamp_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale.clamp(MIN_SCALE, MAX_SCALE)
    } else {
        1.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticZoomLevel {
    Overview,
    Card,
    Detail,
}

pub fn semantic_zoom_level(scale: f32) -> SemanticZoomLevel {
    if scale < 0.65 {
        SemanticZoomLevel::Overview
    } else if scale < 1.5 {
        SemanticZoomLevel::Card
    } else {
        SemanticZoomLevel::Detail
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConceptCard {
    pub id: u32,
    pub title: String,
    pub summary: String,
    pub kind: ConceptKind,
    pub accent_color: Option<[f32; 4]>,
    pub bounds: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConceptKind {
    Organization,
    Team,
    Project,
    Process,
    Repository,
    Service,
    Environment,
    Access,
}

impl ConceptKind {
    pub const ALL: [Self; 8] = [
        Self::Organization,
        Self::Team,
        Self::Project,
        Self::Process,
        Self::Repository,
        Self::Service,
        Self::Environment,
        Self::Access,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Organization => "Organization",
            Self::Team => "Team",
            Self::Project => "Project",
            Self::Process => "Process",
            Self::Repository => "Repository",
            Self::Service => "Service",
            Self::Environment => "Environment",
            Self::Access => "Access",
        }
    }

    pub const fn type_id(self) -> &'static str {
        match self {
            Self::Organization => "organization",
            Self::Team => "team",
            Self::Project => "project",
            Self::Process => "process",
            Self::Repository => "repository",
            Self::Service => "service",
            Self::Environment => "environment",
            Self::Access => "access",
        }
    }

    pub fn from_type_id(type_id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.type_id() == type_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Relationship {
    pub source_id: u32,
    pub target_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RelationshipOverlayGeometry {
    pub source: Point,
    pub target: Point,
}

impl RelationshipOverlayGeometry {
    pub fn between(source_bounds: Rect, target_bounds: Rect) -> Self {
        Self {
            source: source_bounds.center(),
            target: target_bounds.center(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverviewCluster {
    pub bounds: Rect,
    pub card_count: usize,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct GridCell {
    column: i32,
    row: i32,
}

#[derive(Clone, Debug)]
struct UniformGridIndex {
    cell_size: f32,
    cells: HashMap<GridCell, Vec<usize>>,
}

impl UniformGridIndex {
    fn from_cards(cards: &[ConceptCard], cell_size: f32) -> Self {
        assert!(cell_size.is_finite() && cell_size > 0.0);

        let cells = HashMap::new();
        let mut index = Self { cell_size, cells };

        for (card_index, card) in cards.iter().enumerate() {
            index.insert(card_index, card.bounds);
        }

        index
    }

    fn insert(&mut self, card_index: usize, bounds: Rect) {
        if bounds.normalized().is_empty() {
            return;
        }

        for cell in grid_cells_for_bounds(bounds, self.cell_size) {
            self.cells.entry(cell).or_default().push(card_index);
        }
    }

    fn candidate_indices(&self, query: Rect) -> Vec<usize> {
        if query.normalized().is_empty() {
            return Vec::new();
        }

        let mut candidate_indices = Vec::new();
        for cell in grid_cells_for_bounds(query, self.cell_size) {
            if let Some(cell_indices) = self.cells.get(&cell) {
                candidate_indices.extend_from_slice(cell_indices);
            }
        }
        candidate_indices.sort_unstable();
        candidate_indices.dedup();
        candidate_indices
    }
}

fn grid_cells_for_bounds(bounds: Rect, cell_size: f32) -> Vec<GridCell> {
    let normalized = bounds.normalized();
    if normalized.is_empty() {
        return Vec::new();
    }

    let first_column = grid_coordinate(normalized.origin.x, cell_size);
    let last_column = grid_coordinate_before(normalized.max_x(), cell_size);
    let first_row = grid_coordinate(normalized.origin.y, cell_size);
    let last_row = grid_coordinate_before(normalized.max_y(), cell_size);
    let mut cells = Vec::with_capacity(
        (last_column - first_column + 1) as usize * (last_row - first_row + 1) as usize,
    );

    for row in first_row..=last_row {
        for column in first_column..=last_column {
            cells.push(GridCell { column, row });
        }
    }

    cells
}

fn grid_coordinate(value: f32, cell_size: f32) -> i32 {
    (value / cell_size).floor() as i32
}

fn grid_coordinate_before(value: f32, cell_size: f32) -> i32 {
    (value / cell_size).ceil() as i32 - 1
}

#[derive(Clone, Debug)]
pub struct CanvasDataset {
    cards: Vec<ConceptCard>,
    relationships: Vec<Relationship>,
    spatial_index: UniformGridIndex,
}

impl CanvasDataset {
    pub fn from_records(cards: Vec<ConceptCard>, relationships: Vec<Relationship>) -> Self {
        let spatial_index = UniformGridIndex::from_cards(&cards, GRID_CELL_SIZE);

        Self {
            cards,
            relationships,
            spatial_index,
        }
    }

    pub fn generated() -> Self {
        let cards: Vec<_> = (0..CONCEPT_COUNT)
            .map(|index| {
                let column = index % 100;
                let row = index / 100;
                let kind = ConceptKind::ALL[index % ConceptKind::ALL.len()];

                ConceptCard {
                    id: index as u32,
                    title: format!("{} {:04}", kind.label(), index + 1),
                    summary: format!(
                        "{} {:04} is a generated {} concept for canvas exploration.",
                        kind.label(),
                        index + 1,
                        kind.label()
                    ),
                    kind,
                    accent_color: None,
                    bounds: Rect::new(
                        Point::new(column as f32 * 220.0, row as f32 * 140.0),
                        Point::new(180.0, 104.0),
                    ),
                }
            })
            .collect();
        let relationships = (0..RELATIONSHIP_COUNT)
            .map(|index| Relationship {
                source_id: (index % CONCEPT_COUNT) as u32,
                target_id: ((index * 37 + 17) % CONCEPT_COUNT) as u32,
            })
            .collect();
        Self::from_records(cards, relationships)
    }

    pub fn card_count(&self) -> usize {
        self.cards.len()
    }

    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }

    /// Iterates every card in this dataset without creating canvas UI for it.
    /// Callers can use this for bounded spatial projection work, such as
    /// resolving relationship endpoints outside the current viewport.
    pub fn cards(&self) -> impl Iterator<Item = &ConceptCard> {
        self.cards.iter()
    }

    pub fn card_by_id(&self, card_id: u32) -> Option<&ConceptCard> {
        self.cards.iter().find(|card| card.id == card_id)
    }

    /// Resolves visual card containment for the current dataset geometry.
    /// This convenience query has the same spatial-only behavior as
    /// [`card_parent_ids`].
    pub fn card_parent_ids(&self) -> BTreeMap<u32, u32> {
        card_parent_ids(self.cards.iter().map(|card| (card.id, card.bounds)))
    }

    pub fn visible_cards(&self, world_rect: Rect) -> impl Iterator<Item = &ConceptCard> {
        self.cards_intersecting(world_rect).into_iter()
    }

    pub fn cards_intersecting(&self, world_rect: Rect) -> Vec<&ConceptCard> {
        self.candidate_card_indices(world_rect)
            .into_iter()
            .filter_map(|card_index| {
                let card = &self.cards[card_index];
                card.bounds.intersects(world_rect).then_some(card)
            })
            .collect()
    }

    pub fn overview_clusters(&self, world_rect: Rect) -> Vec<OverviewCluster> {
        let world_rect = world_rect.normalized();
        if world_rect.is_empty() {
            return Vec::new();
        }

        let visible_card_indices: Vec<_> = self
            .candidate_card_indices(world_rect)
            .into_iter()
            .filter(|&card_index| self.cards[card_index].bounds.intersects(world_rect))
            .collect();
        let cluster_limit = MAX_OVERVIEW_CLUSTERS.min(visible_card_indices.len());
        if cluster_limit == 0 {
            return Vec::new();
        }

        let (column_count, row_count) = overview_grid_dimensions(world_rect, cluster_limit);
        let cluster_width = world_rect.size.x / column_count as f32;
        let cluster_height = world_rect.size.y / row_count as f32;
        let mut counts = vec![0; column_count * row_count];

        for card_index in visible_card_indices {
            let bounds = self.cards[card_index].bounds.normalized();
            let center = Point::new(
                bounds.origin.x + bounds.size.x / 2.0,
                bounds.origin.y + bounds.size.y / 2.0,
            );
            let column = overview_grid_coordinate(
                center.x,
                world_rect.origin.x,
                cluster_width,
                column_count,
            );
            let row =
                overview_grid_coordinate(center.y, world_rect.origin.y, cluster_height, row_count);
            counts[row * column_count + column] += 1;
        }

        counts
            .into_iter()
            .enumerate()
            .filter(|(_, card_count)| *card_count > 0)
            .map(|(index, card_count)| {
                let column = index % column_count;
                let row = index / column_count;
                OverviewCluster {
                    bounds: Rect::new(
                        Point::new(
                            world_rect.origin.x + column as f32 * cluster_width,
                            world_rect.origin.y + row as f32 * cluster_height,
                        ),
                        Point::new(cluster_width, cluster_height),
                    ),
                    card_count,
                }
            })
            .collect()
    }

    /// Returns every card touched by the normalized selection rectangle.
    pub fn cards_in_selection_rect(&self, first: Point, second: Point) -> Vec<&ConceptCard> {
        self.cards_intersecting(Rect::from_points(first, second))
    }

    /// Returns the deepest fully enclosed card at this world point. Equal-depth
    /// overlaps resolve by highest ID, preserving deterministic selection.
    pub fn hit_test(&self, world_point: Point) -> Option<&ConceptCard> {
        let point_bounds = Rect::new(world_point, Point::new(1.0, 1.0));
        let candidates: Vec<_> = self
            .candidate_card_indices(point_bounds)
            .into_iter()
            .filter_map(|card_index| {
                let card = &self.cards[card_index];
                card.bounds.contains_point(world_point).then_some(card)
            })
            .collect();
        let parent_by_child = card_parent_ids(candidates.iter().map(|card| (card.id, card.bounds)));

        candidates.into_iter().max_by(|left, right| {
            card_containment_depth(left.id, &parent_by_child)
                .cmp(&card_containment_depth(right.id, &parent_by_child))
                .then(left.id.cmp(&right.id))
        })
    }

    fn candidate_card_indices(&self, world_rect: Rect) -> Vec<usize> {
        self.spatial_index.candidate_indices(world_rect)
    }
}

fn card_containment_depth(card_id: u32, parent_by_child: &BTreeMap<u32, u32>) -> usize {
    let mut depth = 0;
    let mut current_id = card_id;
    while let Some(parent_id) = parent_by_child.get(&current_id) {
        depth += 1;
        current_id = *parent_id;
    }
    depth
}

fn overview_grid_dimensions(world_rect: Rect, max_clusters: usize) -> (usize, usize) {
    let aspect_ratio = world_rect.size.x / world_rect.size.y;
    let column_count =
        ((max_clusters as f32 * aspect_ratio).sqrt().ceil() as usize).clamp(1, max_clusters);
    let row_count = (max_clusters / column_count).max(1);

    (column_count, row_count)
}

fn overview_grid_coordinate(value: f32, origin: f32, cell_size: f32, cell_count: usize) -> usize {
    (((value - origin) / cell_size).floor() as isize).clamp(0, cell_count as isize - 1) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    fn assert_close(actual: Point, expected: Point) {
        assert!((actual.x - expected.x).abs() < EPSILON, "x: {actual:?}");
        assert!((actual.y - expected.y).abs() < EPSILON, "y: {actual:?}");
    }

    fn dataset_with_card(bounds: Rect) -> CanvasDataset {
        CanvasDataset::from_records(
            vec![ConceptCard {
                id: 42,
                title: "Visible card".to_owned(),
                summary: "A deterministic visibility-test card.".to_owned(),
                kind: ConceptKind::Project,
                accent_color: None,
                bounds,
            }],
            Vec::new(),
        )
    }

    fn visible_card_ids(dataset: &CanvasDataset, camera: Camera, viewport_size: Point) -> Vec<u32> {
        dataset
            .visible_cards(camera.visible_world_rect(viewport_size))
            .map(|card| card.id)
            .collect()
    }

    #[test]
    fn world_and_screen_transforms_round_trip_after_pan() {
        let mut camera = Camera::new(Point::new(80.0, 80.0), 1.5);
        camera.pan_by_screen(Point::new(48.0, -21.0));
        let world_point = Point::new(420.5, -13.25);

        assert_close(
            camera.screen_to_world(camera.world_to_screen(world_point)),
            world_point,
        );
        assert_eq!(camera.offset, Point::new(128.0, 59.0));
        assert_eq!(camera.scale, 1.5);
    }

    #[test]
    fn wheel_zoom_keeps_the_world_anchor_under_the_pointer() {
        let mut camera = Camera::default();
        let screen_anchor = Point::new(600.0, 450.0);
        let world_anchor = camera.screen_to_world(screen_anchor);

        camera.zoom_by_wheel_at_screen(screen_anchor, -240.0);

        assert!(camera.scale > 1.0);
        assert_close(camera.world_to_screen(world_anchor), screen_anchor);
    }

    #[test]
    fn zoom_is_clamped_without_moving_the_pointer_anchor() {
        let mut camera = Camera::default();
        let screen_anchor = Point::new(100.0, 120.0);
        let world_anchor = camera.screen_to_world(screen_anchor);

        camera.zoom_at_screen(screen_anchor, MAX_SCALE * 2.0);
        assert_eq!(camera.scale, MAX_SCALE);
        assert_close(camera.world_to_screen(world_anchor), screen_anchor);

        camera.zoom_at_screen(screen_anchor, MIN_SCALE / MAX_SCALE / 2.0);
        assert_eq!(camera.scale, MIN_SCALE);
        assert_close(camera.world_to_screen(world_anchor), screen_anchor);
    }

    #[test]
    fn visible_cards_include_a_card_at_two_percent_scale() {
        let viewport_size = Point::new(1_280.0, 820.0);
        let bounds = Rect::new(Point::new(10_000.0, 5_000.0), Point::new(240.0, 132.0));
        let dataset = dataset_with_card(bounds);
        let world_anchor = bounds.origin + bounds.size * 0.5;
        let mut camera = Camera::new(Point::default(), 0.02);

        camera.center_world_point_in_viewport(world_anchor, viewport_size);

        assert_eq!(visible_card_ids(&dataset, camera, viewport_size), vec![42]);
        assert_eq!(dataset.card_by_id(42).map(|card| card.bounds), Some(bounds));
    }

    #[test]
    fn zooming_from_two_to_one_percent_keeps_visible_cards_at_their_canvas_position() {
        let viewport_size = Point::new(1_280.0, 820.0);
        let viewport_center = viewport_size * 0.5;
        let bounds = Rect::new(Point::new(10_000.0, 5_000.0), Point::new(240.0, 132.0));
        let dataset = dataset_with_card(bounds);
        let world_anchor = bounds.origin + bounds.size * 0.5;
        let mut camera = Camera::new(Point::default(), 0.02);

        camera.center_world_point_in_viewport(world_anchor, viewport_size);
        assert_eq!(visible_card_ids(&dataset, camera, viewport_size), vec![42]);

        camera.zoom_at_screen(viewport_center, 0.5);

        assert_eq!(camera.scale, MIN_SCALE);
        assert_close(camera.world_to_screen(world_anchor), viewport_center);
        assert_eq!(visible_card_ids(&dataset, camera, viewport_size), vec![42]);
        assert_eq!(dataset.card_by_id(42).map(|card| card.bounds), Some(bounds));
    }

    #[test]
    fn centering_a_world_point_preserves_scale() {
        let mut camera = Camera::new(Point::new(-140.0, 90.0), 2.5);
        let viewport = Point::new(1_280.0, 820.0);
        let world_point = Point::new(320.0, -48.0);

        camera.center_world_point_in_viewport(world_point, viewport);

        assert_eq!(camera.scale, 2.5);
        assert_close(
            camera.world_to_screen(world_point),
            Point::new(640.0, 410.0),
        );
    }

    #[test]
    fn pan_has_no_canvas_position_boundary() {
        let mut camera = Camera::new(Point::default(), 1.0);

        camera.pan_by_screen(Point::new(1_000_000.0, -1_000_000.0));

        assert_eq!(camera.offset, Point::new(1_000_000.0, -1_000_000.0));
    }

    #[test]
    fn invalid_zoom_inputs_leave_the_camera_unchanged() {
        let mut camera = Camera::default();
        let initial_camera = camera;

        camera.zoom_at_screen(Point::new(120.0, 80.0), -1.0);
        camera.zoom_by_wheel_at_screen(Point::new(120.0, 80.0), f32::NAN);

        assert_eq!(camera, initial_camera);
    }

    #[test]
    fn semantic_zoom_levels_change_at_documented_thresholds() {
        assert_eq!(semantic_zoom_level(0.25), SemanticZoomLevel::Overview);
        assert_eq!(semantic_zoom_level(0.649), SemanticZoomLevel::Overview);
        assert_eq!(semantic_zoom_level(0.65), SemanticZoomLevel::Card);
        assert_eq!(semantic_zoom_level(1.499), SemanticZoomLevel::Card);
        assert_eq!(semantic_zoom_level(1.5), SemanticZoomLevel::Detail);
        assert_eq!(semantic_zoom_level(4.0), SemanticZoomLevel::Detail);
    }

    #[test]
    fn relationship_overlay_geometry_uses_normalized_card_centers() {
        let geometry = RelationshipOverlayGeometry::between(
            Rect::new(Point::new(10.0, 20.0), Point::new(30.0, 40.0)),
            Rect::new(Point::new(100.0, 110.0), Point::new(-20.0, -10.0)),
        );

        assert_eq!(geometry.source, Point::new(25.0, 40.0));
        assert_eq!(geometry.target, Point::new(90.0, 105.0));
    }

    #[test]
    fn generated_dataset_has_required_scale_and_deterministic_records() {
        let dataset = CanvasDataset::generated();

        assert_eq!(dataset.card_count(), CONCEPT_COUNT);
        assert_eq!(dataset.relationship_count(), RELATIONSHIP_COUNT);
        assert_eq!(
            dataset.cards.first().map(|card| card.title.as_str()),
            Some("Organization 0001")
        );
        assert_eq!(
            dataset.cards.first().map(|card| card.summary.as_str()),
            Some("Organization 0001 is a generated Organization concept for canvas exploration.")
        );
        assert_eq!(dataset.cards.last().map(|card| card.id), Some(9_999));
        assert_eq!(
            dataset.relationships[0],
            Relationship {
                source_id: 0,
                target_id: 17
            }
        );
        assert_eq!(
            dataset.relationships[49_999],
            Relationship {
                source_id: 9_999,
                target_id: 9_980
            }
        );
    }

    #[test]
    fn realistic_ten_thousand_card_viewport_query_uses_a_small_candidate_set() {
        let dataset = CanvasDataset::generated();
        let viewport = Rect::new(Point::new(8_800.0, 7_000.0), Point::new(1_080.0, 680.0));
        let candidate_count = dataset.candidate_card_indices(viewport).len();
        let visible_cards = dataset.cards_intersecting(viewport);

        assert!(candidate_count < 100, "candidate count: {candidate_count}");
        assert!(visible_cards.len() >= 15);
        assert!(visible_cards.len() <= candidate_count);
        assert!(
            visible_cards
                .iter()
                .all(|card| card.bounds.intersects(viewport))
        );
    }

    #[test]
    fn hit_test_returns_the_card_at_a_world_point() {
        let dataset = CanvasDataset::generated();

        assert_eq!(
            dataset
                .hit_test(Point::new(230.0, 150.0))
                .map(|card| card.id),
            Some(101)
        );
        assert_eq!(dataset.hit_test(Point::new(190.0, 105.0)), None);
    }

    #[test]
    fn card_containment_selects_the_smallest_full_parent_only() {
        let parent_by_child = card_parent_ids([
            (
                90,
                Rect::new(Point::new(0.0, 0.0), Point::new(400.0, 400.0)),
            ),
            (
                20,
                Rect::new(Point::new(40.0, 40.0), Point::new(260.0, 260.0)),
            ),
            (
                4,
                Rect::new(Point::new(100.0, 100.0), Point::new(80.0, 80.0)),
            ),
        ]);

        assert_eq!(parent_by_child.get(&4), Some(&20));
        assert_eq!(parent_by_child.get(&20), Some(&90));
        assert!(
            card_parent_ids([
                (
                    7,
                    Rect::new(Point::new(120.0, 120.0), Point::new(80.0, 80.0)),
                ),
                (
                    4,
                    Rect::new(Point::new(100.0, 100.0), Point::new(80.0, 80.0)),
                ),
            ])
            .is_empty()
        );
        assert!(
            card_parent_ids([
                (
                    8,
                    Rect::new(Point::new(100.0, 100.0), Point::new(80.0, 80.0)),
                ),
                (
                    4,
                    Rect::new(Point::new(100.0, 100.0), Point::new(80.0, 80.0)),
                ),
            ])
            .is_empty()
        );
    }

    #[test]
    fn containment_chain_recognizes_direct_and_transitive_card_parents() {
        let parent_by_child = BTreeMap::from([(4, 20), (20, 90)]);

        assert!(cards_are_in_containment_chain(&parent_by_child, 4, 20));
        assert!(cards_are_in_containment_chain(&parent_by_child, 90, 4));
        assert!(!cards_are_in_containment_chain(&parent_by_child, 4, 7));
        assert!(!cards_are_in_containment_chain(&parent_by_child, 4, 4));
    }

    #[test]
    fn moving_a_card_container_includes_nested_descendants_but_not_overlaps() {
        let card_bounds = [
            (9, Rect::new(Point::new(0.0, 0.0), Point::new(500.0, 360.0))),
            (
                7,
                Rect::new(Point::new(80.0, 80.0), Point::new(240.0, 180.0)),
            ),
            (
                4,
                Rect::new(Point::new(120.0, 120.0), Point::new(80.0, 60.0)),
            ),
            (
                2,
                Rect::new(Point::new(440.0, 80.0), Point::new(120.0, 80.0)),
            ),
        ];

        assert_eq!(
            card_move_ids_with_contained_descendants(card_bounds, [9]),
            vec![4, 7, 9]
        );
        assert_eq!(
            card_move_ids_with_contained_descendants(card_bounds, [7]),
            vec![4, 7]
        );
    }

    #[test]
    fn moving_a_canvas_container_includes_cards_and_boundary_frames() {
        let parent = CanvasItemId::Card(9);
        let child_card = CanvasItemId::Card(4);
        let child_boundary = CanvasItemId::Boundary("boundary-child".to_string());
        let overlapping = CanvasItemId::Boundary("boundary-overlap".to_string());
        let items = vec![
            CanvasItemBounds {
                id: parent.clone(),
                bounds: Rect::new(Point::new(0.0, 0.0), Point::new(500.0, 360.0)),
            },
            CanvasItemBounds {
                id: child_card.clone(),
                bounds: Rect::new(Point::new(120.0, 120.0), Point::new(80.0, 60.0)),
            },
            CanvasItemBounds {
                id: child_boundary.clone(),
                bounds: Rect::new(Point::new(260.0, 120.0), Point::new(140.0, 100.0)),
            },
            CanvasItemBounds {
                id: overlapping.clone(),
                bounds: Rect::new(Point::new(440.0, 80.0), Point::new(120.0, 80.0)),
            },
        ];

        assert_eq!(
            canvas_item_move_ids_with_contained_descendants(items, [parent.clone()]),
            vec![child_card, parent.clone(), child_boundary]
        );
        assert_eq!(
            canvas_item_move_ids_with_contained_descendants(
                vec![
                    CanvasItemBounds {
                        id: CanvasItemId::Boundary("parent".to_string()),
                        bounds: Rect::new(Point::new(0.0, 0.0), Point::new(500.0, 360.0)),
                    },
                    CanvasItemBounds {
                        id: CanvasItemId::Card(4),
                        bounds: Rect::new(Point::new(120.0, 120.0), Point::new(80.0, 60.0)),
                    },
                ],
                [CanvasItemId::Boundary("parent".to_string())],
            ),
            vec![
                CanvasItemId::Card(4),
                CanvasItemId::Boundary("parent".to_string()),
            ]
        );
        assert!(
            !canvas_item_move_ids_with_contained_descendants(
                vec![CanvasItemBounds {
                    id: overlapping.clone(),
                    bounds: Rect::new(Point::new(440.0, 80.0), Point::new(120.0, 80.0)),
                }],
                [parent],
            )
            .contains(&overlapping)
        );
    }

    #[test]
    fn card_containment_accepts_a_child_touching_its_parent_edge() {
        let parent = Rect::new(Point::new(0.0, 0.0), Point::new(200.0, 160.0));
        let child = Rect::new(Point::new(0.0, 40.0), Point::new(80.0, 80.0));

        assert!(parent.fully_contains(child));
    }

    #[test]
    fn hit_test_prioritizes_a_fully_enclosed_child_over_a_higher_id_parent() {
        let parent = ConceptCard {
            id: 90,
            title: "Parent".to_string(),
            summary: String::new(),
            kind: ConceptKind::Organization,
            accent_color: None,
            bounds: Rect::new(Point::new(0.0, 0.0), Point::new(300.0, 220.0)),
        };
        let child = ConceptCard {
            id: 4,
            title: "Child".to_string(),
            summary: String::new(),
            kind: ConceptKind::Project,
            accent_color: None,
            bounds: Rect::new(Point::new(80.0, 60.0), Point::new(100.0, 70.0)),
        };
        let dataset = CanvasDataset::from_records(vec![parent, child], Vec::new());

        assert_eq!(
            dataset
                .hit_test(Point::new(120.0, 90.0))
                .map(|card| card.id),
            Some(4)
        );
    }

    #[test]
    fn card_lookup_returns_only_present_cards() {
        let dataset = CanvasDataset::generated();

        assert_eq!(
            dataset.card_by_id(101).map(|card| card.title.as_str()),
            Some("Service 0102")
        );
        assert_eq!(dataset.card_by_id(10_000), None);
    }

    #[test]
    fn selection_rectangle_is_normalized_and_returns_touched_cards() {
        let dataset = CanvasDataset::generated();
        let selected =
            dataset.cards_in_selection_rect(Point::new(430.0, 250.0), Point::new(10.0, 10.0));
        let selected_ids: Vec<_> = selected.into_iter().map(|card| card.id).collect();

        assert_eq!(selected_ids, vec![0, 1, 100, 101]);
    }

    #[test]
    fn visible_card_query_excludes_offscreen_cards() {
        let dataset = CanvasDataset::generated();
        let visible: Vec<_> = dataset
            .visible_cards(Rect::new(Point::new(0.0, 0.0), Point::new(200.0, 120.0)))
            .collect();

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, 0);
    }

    #[test]
    fn full_dataset_overview_query_returns_bounded_deterministic_clusters() {
        let dataset = CanvasDataset::generated();
        let world_rect = Rect::new(Point::new(0.0, 0.0), Point::new(22_000.0, 14_000.0));

        let clusters = dataset.overview_clusters(world_rect);

        assert!(!clusters.is_empty());
        assert!(
            clusters.len() <= MAX_OVERVIEW_CLUSTERS,
            "cluster count: {}",
            clusters.len()
        );
        assert_eq!(
            clusters
                .iter()
                .map(|cluster| cluster.card_count)
                .sum::<usize>(),
            CONCEPT_COUNT
        );
        assert!(
            clusters
                .iter()
                .all(|cluster| { cluster.card_count > 0 && cluster.bounds.intersects(world_rect) })
        );
        assert_eq!(clusters, dataset.overview_clusters(world_rect));
    }
}
