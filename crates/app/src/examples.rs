//! One immutable example catalog shared by native and browser menus.
use infinite_note_canvas_canvas::WorkspaceDocument;

#[derive(Clone, Copy)]
pub(crate) struct Example {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(crate) menu_id: &'static str,
    pub(crate) title: &'static str,
    source: &'static str,
}

impl Example {
    pub(crate) fn document(self) -> Result<WorkspaceDocument, String> {
        WorkspaceDocument::parse_json(self.source).map_err(|error| error.to_string())
    }
}

pub(crate) const ALL: &[Example] = &[
    Example {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        menu_id: "example-static-react-website",
        title: "Static React website",
        source: include_str!("../../../workspaces/static-react-website.infinite-note.json"),
    },
    Example {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        menu_id: "example-car-manufacturing-vsm",
        title: "Car manufacturing value stream",
        source: include_str!("../../../workspaces/car-manufacturing-vsm.infinite-note.json"),
    },
];

pub(crate) fn is_example(document: &WorkspaceDocument) -> bool {
    ALL.iter().any(|example| {
        example
            .document()
            .is_ok_and(|bundled| bundled.workspace_id == document.workspace_id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_reveal_detail_without_overwhelming_the_overview() {
        for example in ALL {
            let document = example.document().unwrap();
            let version = document.current_state_id();
            let lens = document.default_lens_id(version).unwrap();
            let counts: Vec<_> = [0.12, 0.45, 1.2]
                .into_iter()
                .map(|scale| {
                    document
                        .semantic_zoom_frame(version, lens, scale)
                        .unwrap()
                        .cards
                        .len()
                })
                .collect();
            assert!(
                counts[0] < counts[1] && counts[1] < counts[2],
                "{}: {counts:?}",
                example.title
            );
            assert!(is_example(&document));
            let view = &document.version(version).unwrap().lenses[0];
            assert!(!view.visual_boundaries.is_empty());
        }
    }
    #[test]
    fn examples_have_nested_cards_and_cross_boundary_relationships() {
        use infinite_note_canvas_canvas::card_parent_ids;
        for example in ALL {
            let document = example.document().unwrap();
            let version = document.version(document.current_state_id()).unwrap();
            let lens = &version.lenses[0];
            let parents = card_parent_ids(lens.placements.iter().map(|p| (p.concept_id, p.bounds)));
            assert!(
                parents.values().any(|parent| parents.contains_key(parent)),
                "{} needs two levels of nesting",
                example.title
            );
            let region = |id| {
                let bounds = lens
                    .placements
                    .iter()
                    .find(|p| p.concept_id == id)
                    .unwrap()
                    .bounds;
                lens.visual_boundaries
                    .iter()
                    .position(|boundary| boundary.bounds.contains_point(bounds.center()))
            };
            assert!(
                version.relationships.iter().any(|edge| {
                    let source = region(edge.source_id);
                    let target = region(edge.target_id);
                    source.is_some() && target.is_some() && source != target
                }),
                "{} needs a cross-boundary connection",
                example.title
            );
            let counts: Vec<_> = [0.22, 0.24, 0.26, 0.83, 0.85, 0.87]
                .into_iter()
                .map(|scale| {
                    document
                        .semantic_zoom_frame(&version.id, &lens.id, scale)
                        .unwrap()
                        .cards
                        .len()
                })
                .collect();
            assert!(
                counts.windows(2).all(|pair| pair[0] <= pair[1]),
                "{} loses context: {counts:?}",
                example.title
            );
        }
    }
}
