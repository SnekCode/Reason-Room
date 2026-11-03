use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    CanvasDataset, CanvasItemBounds, CanvasItemId, ConceptCard, ConceptKind, Point, Rect,
    Relationship, RelationshipOverlayGeometry, canvas_item_move_ids_with_contained_descendants,
    card_parent_ids, cards_are_in_containment_chain,
};

pub const WORKSPACE_SCHEMA_VERSION: u32 = 7;
/// The default new-card dimensions at 100% zoom. Automatic layer selection
/// compares Lens-local card bounds to this presentation baseline.
pub const NEW_CARD_BASELINE_SIZE_AT_100_PERCENT: Point = Point::new(240.0, 120.0);
pub const MINIMUM_ZOOM_PERCENT: f32 = 1.0;
pub const MAXIMUM_ZOOM_PERCENT: f32 = 409_600.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceDocument {
    pub schema_version: u32,
    pub workspace_id: String,
    pub title: String,
    #[serde(default)]
    pub custom_types: Vec<CustomType>,
    pub current_state: WorkspaceVersion,
    #[serde(default)]
    pub planning_scenarios: Vec<PlanningScenario>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomType {
    pub id: String,
    pub title: String,
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceVersion {
    pub id: String,
    pub title: String,
    pub concepts: Vec<WorkspaceConcept>,
    pub relationships: Vec<TypedRelationship>,
    pub lenses: Vec<WorkspaceLens>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanningScenario {
    pub based_on_version_id: String,
    #[serde(flatten)]
    pub version: WorkspaceVersion,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceConcept {
    /// Explicit shared-content identity; geometry and relationships remain local.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_reference: Option<String>,
    pub id: u32,
    pub type_id: String,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub details: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<ExecutionMode>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardPasteMode {
    Copy,
    Reference,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Manual,
    Automated,
    Hybrid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    Contains,
    Owns,
    Operates,
    DependsOn,
    ImplementedIn,
    Delivers,
    DeployedTo,
}

impl RelationshipKind {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::Owns => "owns",
            Self::Operates => "operates",
            Self::DependsOn => "depends-on",
            Self::ImplementedIn => "implemented-in",
            Self::Delivers => "delivers",
            Self::DeployedTo => "deployed-to",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::Owns => "owns",
            Self::Operates => "operates",
            Self::DependsOn => "depends on",
            Self::ImplementedIn => "implemented in",
            Self::Delivers => "delivers",
            Self::DeployedTo => "deployed to",
        }
    }

    /// Returns whether this relationship type can connect the supplied
    /// concept types. The rule is shared by authoring controls and workspace
    /// validation so invalid relationship types are never offered for a
    /// source-target pair.
    pub fn supports_endpoints(self, source_type_id: &str, target_type_id: &str) -> bool {
        match self {
            Self::Owns | Self::Operates => source_type_id == "team",
            Self::ImplementedIn => source_type_id == "process" && target_type_id == "repository",
            Self::Contains | Self::DependsOn | Self::Delivers | Self::DeployedTo => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedRelationship {
    pub id: String,
    pub kind: RelationshipKind,
    pub source_id: u32,
    pub target_id: u32,
    #[serde(default)]
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceLens {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<LensTemplate>,
    #[serde(default)]
    pub filter: LensFilter,
    pub placements: Vec<ConceptPlacement>,
    /// View-local connections. Missing in legacy files means the version connections.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationships: Option<Vec<TypedRelationship>>,
    /// Named spatial context in this Lens. A visual boundary does not create
    /// membership, responsibility, ownership, or a graph relationship.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visual_boundaries: Vec<VisualBoundary>,
    /// Ordered presentation rules that assign base levels in this Lens.
    /// An empty list preserves the complete Lens presentation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_zoom_layers: Vec<SemanticZoomLayer>,
    /// Per-card presentation overrides for this Lens. The default is automatic
    /// size-based layer assignment. These modes do not change graph facts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_zoom_card_modes: Vec<SemanticZoomCardLayerModeEntry>,
}

/// A named, Lens-local visual area used to give a part of a map context.
///
/// Boundaries intentionally retain no member IDs. A card may be arranged
/// inside, outside, or across a boundary without changing a workspace fact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualBoundary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<String>,
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub bounds: Rect,
}

/// An inclusive zoom interval, expressed as a canvas scale multiplier.
///
/// This remains available to deserialize schema version 4 layer controls. New
/// layer presentation persists user-facing percentage ranges instead.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoomRange {
    pub start: f32,
    pub end: f32,
}

/// An inclusive user-facing zoom interval. `100` means the canvas is at 100%.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoomPercentRange {
    pub start_percent: f32,
    pub end_percent: f32,
}

/// One Lens-local layer's active range and symmetric fade distance.
///
/// The layer is fully visible inside `active_zoom_range`. It fades in for
/// `fade_percent` before the start and fades out for the same distance after
/// the end. These controls are presentation data only.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticZoomLayerPercentSettings {
    pub active_zoom_range: ZoomPercentRange,
    pub fade_percent: f32,
}

/// Controls how one card participates in a Lens's zoom-layer presentation.
///
/// This never changes a concept, its type, or its graph relationships.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticZoomCardLayerMode {
    /// Select exactly one layer from the card's Lens-local size at 100% zoom.
    Automatic,
    /// Show the card on the explicitly selected Lens-local layers.
    Manual { layer_ids: Vec<String> },
    /// Legacy one-layer policy. Schema version 5 data migrates this to
    /// [`Self::Manual`] before it is saved again.
    Dedicated { layer_id: String },
    /// Legacy all-layer policy. Schema version 5 data migrates this to
    /// [`Self::Manual`] before it is saved again.
    AlwaysVisible,
}

/// The resolved Lens-local layer visibility for one card.
///
/// `base_layer_id` is the resolved automatic layer or the first selected
/// manual layer. It is absent only when the Lens has no configured layers.
/// `visible_layer_ids` contains exactly the layers where the card can render.
/// This is presentation information, not a graph fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticZoomCardLayerVisibility {
    pub mode: SemanticZoomCardLayerMode,
    pub base_layer_id: Option<String>,
    pub visible_layer_ids: Vec<String>,
}

/// An explicit card presentation override. Omitted cards use `Automatic`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticZoomCardLayerModeEntry {
    pub concept_id: u32,
    pub mode: SemanticZoomCardLayerMode,
}

/// A user-authored presentation rule inside one Lens. Relationships have one
/// authored layer. Automatic cards resolve one layer from size, while manual
/// cards keep an explicit list of visible layers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticZoomLayer {
    pub id: String,
    pub title: String,
    /// User-facing active percent range and symmetric fade. Schema version 6
    /// persists this field. The legacy ranges remain only for deterministic
    /// schema version 4 migration and temporary source compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent_settings: Option<SemanticZoomLayerPercentSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_in: Option<ZoomRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_out: Option<ZoomRange>,
    pub card_scale: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub concept_ids: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationship_ids: Vec<String>,
}

/// The presentation settings that can change without changing layer membership.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticZoomLayerSettings {
    pub title: String,
    pub fade_in: Option<ZoomRange>,
    pub fade_out: Option<ZoomRange>,
    pub card_scale: f32,
}

/// A visible card and its resolved presentation at one canvas scale.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticZoomCard {
    pub card: ConceptCard,
    pub opacity: f32,
    pub card_scale: f32,
}

/// A visible relationship and its resolved presentation at one canvas scale.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticZoomRelationship {
    pub id: String,
    pub kind: RelationshipKind,
    pub label: String,
    pub source_id: u32,
    pub target_id: u32,
    pub opacity: f32,
}

/// A resolved authored layer at one canvas scale.
///
/// This is presentation-only state. It does not change the layer definition
/// or card membership that the workspace stores.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticZoomLayerProjection {
    pub id: String,
    pub title: String,
    pub opacity: f32,
}

/// The pure semantic-zoom projection for one Lens at one scale.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticZoomFrame {
    pub cards: Vec<SemanticZoomCard>,
    pub relationships: Vec<SemanticZoomRelationship>,
    pub layers: Vec<SemanticZoomLayerProjection>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LensTemplate {
    Ownership,
    ProcessesAndDependencies,
    ServicesAndAccess,
    Deployment,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LensFilter {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub concept_type_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationship_kinds: Vec<RelationshipKind>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConceptPlacement {
    pub concept_id: u32,
    pub bounds: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedConceptRelationshipOverlay {
    pub selected_concept: SelectedConceptOverlaySource,
    pub relationships: Vec<SelectedConceptRelationshipOverlayEntry>,
}

impl SelectedConceptRelationshipOverlay {
    /// Returns the legacy center-to-center geometry for visible endpoint pairs.
    pub fn visible_lines(&self) -> Vec<SelectedConceptRelationshipOverlayLine> {
        let SelectedConceptOverlaySource::Visible {
            bounds: source_bounds,
        } = self.selected_concept
        else {
            return Vec::new();
        };

        self.relationships
            .iter()
            .filter_map(|relationship| {
                let RelationshipOverlayTarget::Visible {
                    bounds: target_bounds,
                } = relationship.target
                else {
                    return None;
                };
                let parent_by_child = card_parent_ids([
                    (relationship.source_id, source_bounds),
                    (relationship.target_id, target_bounds),
                ]);
                if cards_are_in_containment_chain(
                    &parent_by_child,
                    relationship.source_id,
                    relationship.target_id,
                ) {
                    return None;
                }

                Some(SelectedConceptRelationshipOverlayLine {
                    relationship_id: relationship.relationship_id.clone(),
                    relationship_kind: relationship.relationship_kind,
                    source_id: relationship.source_id,
                    target_id: relationship.target_id,
                    geometry: RelationshipOverlayGeometry::between(source_bounds, target_bounds),
                })
            })
            .collect()
    }

    fn visible_paths(
        &self,
        visible_card_bounds_by_concept_id: &BTreeMap<u32, Rect>,
    ) -> Vec<SelectedConceptRelationshipOverlayPath> {
        if !matches!(
            self.selected_concept,
            SelectedConceptOverlaySource::Visible { .. }
        ) {
            return Vec::new();
        }
        let parent_by_child = card_parent_ids(
            visible_card_bounds_by_concept_id
                .iter()
                .map(|(concept_id, bounds)| (*concept_id, *bounds)),
        );

        self.relationships
            .iter()
            .filter_map(|relationship| {
                if !matches!(
                    relationship.target,
                    RelationshipOverlayTarget::Visible { .. }
                ) {
                    return None;
                }
                if cards_are_in_containment_chain(
                    &parent_by_child,
                    relationship.source_id,
                    relationship.target_id,
                ) {
                    return None;
                }
                let source_bounds = visible_card_bounds_by_concept_id
                    .get(&relationship.source_id)
                    .copied()?;
                let target_bounds = visible_card_bounds_by_concept_id
                    .get(&relationship.target_id)
                    .copied()?;
                let obstacles: Vec<_> = visible_card_bounds_by_concept_id
                    .iter()
                    .filter_map(|(concept_id, bounds)| {
                        (*concept_id != relationship.source_id
                            && *concept_id != relationship.target_id)
                            .then_some(*bounds)
                    })
                    .collect();

                Some(SelectedConceptRelationshipOverlayPath {
                    relationship_id: relationship.relationship_id.clone(),
                    relationship_kind: relationship.relationship_kind,
                    source_id: relationship.source_id,
                    target_id: relationship.target_id,
                    points: relationship_overlay_path(source_bounds, target_bounds, &obstacles),
                })
            })
            .collect()
    }
}

/// Transient active-Lens placement bounds used only by relationship-overlay
/// queries. Invalid bounds leave the stored placement in effect.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelationshipOverlayPlacementOverrides {
    pub bounds_by_concept_id: BTreeMap<u32, Rect>,
}

impl RelationshipOverlayPlacementOverrides {
    pub fn with_bounds(concept_id: u32, bounds: Rect) -> Self {
        Self {
            bounds_by_concept_id: BTreeMap::from([(concept_id, bounds)]),
        }
    }

    pub fn insert(&mut self, concept_id: u32, bounds: Rect) {
        self.bounds_by_concept_id.insert(concept_id, bounds);
    }

    fn effective_bounds(&self, concept_id: u32, stored_bounds: Rect) -> Rect {
        self.bounds_by_concept_id
            .get(&concept_id)
            .copied()
            .filter(|bounds| has_valid_bounds(*bounds))
            .unwrap_or(stored_bounds)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SelectedConceptOverlaySource {
    Visible { bounds: Rect },
    Offscreen,
    Filtered,
    Unplaced,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedConceptRelationshipOverlayEntry {
    pub relationship_id: String,
    pub relationship_kind: RelationshipKind,
    pub source_id: u32,
    pub target_id: u32,
    pub related_concept_id: u32,
    pub target: RelationshipOverlayTarget,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RelationshipOverlayTarget {
    Visible { bounds: Rect },
    Offscreen,
    Filtered,
    Absent,
    Unplaced,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedConceptRelationshipOverlayLine {
    pub relationship_id: String,
    pub relationship_kind: RelationshipKind,
    pub source_id: u32,
    pub target_id: u32,
    pub geometry: RelationshipOverlayGeometry,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedConceptRelationshipOverlayPath {
    pub relationship_id: String,
    pub relationship_kind: RelationshipKind,
    pub source_id: u32,
    pub target_id: u32,
    /// The ordered world-space polyline points. The first and last points are
    /// on the source and target card edges, respectively.
    pub points: Vec<Point>,
}

struct SelectedConceptRelationshipOverlayQuery {
    overlay: SelectedConceptRelationshipOverlay,
    visible_card_bounds_by_concept_id: BTreeMap<u32, Rect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConceptDeletionPreview {
    pub concept_id: u32,
    pub title: String,
    pub incident_relationship_count: usize,
    pub placement_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeletedConcept {
    pub version_id: String,
    pub concept_index: usize,
    pub concept: WorkspaceConcept,
    pub relationships: Vec<DeletedTypedRelationship>,
    pub lens_relationships: Vec<(String, DeletedTypedRelationship)>,
    pub placements: Vec<DeletedConceptPlacement>,
    pub card_layer_modes: Vec<DeletedSemanticZoomCardLayerMode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeletedTypedRelationship {
    pub index: usize,
    pub relationship: TypedRelationship,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeletedConceptPlacement {
    pub lens_id: String,
    pub index: usize,
    pub placement: ConceptPlacement,
}

/// A Lens-local card presentation override removed with its concept.
///
/// Keeping this receipt makes direct domain restoration lossless and prevents
/// a deleted concept ID from remaining in semantic zoom validation records.
#[derive(Clone, Debug, PartialEq)]
pub struct DeletedSemanticZoomCardLayerMode {
    pub lens_id: String,
    pub index: usize,
    pub entry: SemanticZoomCardLayerModeEntry,
}

pub type WorkspaceView = WorkspaceLens;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceError {
    InvalidJson(String),
    InvalidSchemaVersion(u32),
    Validation(String),
    MissingVersion(String),
    MissingConcept {
        version_id: String,
        concept_id: u32,
    },
    MissingRelationship {
        version_id: String,
        relationship_id: String,
    },
    MissingLens {
        version_id: String,
        lens_id: String,
    },
    MissingSemanticZoomLayer {
        version_id: String,
        lens_id: String,
        layer_id: String,
    },
    MissingVisualBoundary {
        version_id: String,
        lens_id: String,
        boundary_id: String,
    },
    MissingPlacement {
        version_id: String,
        lens_id: String,
        concept_id: u32,
    },
    RestoreConceptAlreadyExists {
        version_id: String,
        concept_id: u32,
    },
    InvalidConceptDeletionReceipt {
        version_id: String,
        reason: String,
    },
    Serialization(String),
}

impl Display for WorkspaceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(formatter, "invalid workspace JSON: {error}"),
            Self::InvalidSchemaVersion(version) => {
                write!(formatter, "unsupported workspace schema version: {version}")
            }
            Self::Validation(message) => formatter.write_str(message),
            Self::MissingVersion(version_id) => {
                write!(formatter, "workspace version was not found: {version_id}")
            }
            Self::MissingConcept {
                version_id,
                concept_id,
            } => write!(
                formatter,
                "workspace version {version_id} has no concept with id {concept_id}"
            ),
            Self::MissingRelationship {
                version_id,
                relationship_id,
            } => write!(
                formatter,
                "workspace version {version_id} has no relationship named {relationship_id}"
            ),
            Self::MissingLens {
                version_id,
                lens_id,
            } => write!(
                formatter,
                "workspace version {version_id} has no lens named {lens_id}"
            ),
            Self::MissingSemanticZoomLayer {
                version_id,
                lens_id,
                layer_id,
            } => write!(
                formatter,
                "workspace version {version_id}, lens {lens_id} has no semantic zoom layer named {layer_id}"
            ),
            Self::MissingVisualBoundary {
                version_id,
                lens_id,
                boundary_id,
            } => write!(
                formatter,
                "workspace version {version_id}, lens {lens_id} has no visual boundary named {boundary_id}"
            ),
            Self::MissingPlacement {
                version_id,
                lens_id,
                concept_id,
            } => write!(
                formatter,
                "workspace version {version_id}, lens {lens_id} has no placement for concept {concept_id}"
            ),
            Self::RestoreConceptAlreadyExists {
                version_id,
                concept_id,
            } => write!(
                formatter,
                "workspace version {version_id} already has a concept with id {concept_id}"
            ),
            Self::InvalidConceptDeletionReceipt { version_id, reason } => write!(
                formatter,
                "concept deletion receipt for workspace version {version_id} is invalid: {reason}"
            ),
            Self::Serialization(error) => {
                write!(formatter, "workspace serialization failed: {error}")
            }
        }
    }
}

impl std::error::Error for WorkspaceError {}

impl WorkspaceDocument {
    /// Creates a valid empty workspace with one unconstrained Canvas Lens.
    ///
    /// A new workspace deliberately starts without concepts, relationships, or
    /// semantic zoom layers. Those facts are authored after creation, while
    /// the single Lens gives the session a valid canvas to open immediately.
    pub fn blank(workspace_id: String, title: String) -> Result<Self, WorkspaceError> {
        let document = Self {
            schema_version: WORKSPACE_SCHEMA_VERSION,
            workspace_id,
            title,
            custom_types: Vec::new(),
            current_state: WorkspaceVersion {
                id: "current".to_string(),
                title: "Current State".to_string(),
                concepts: Vec::new(),
                relationships: Vec::new(),
                lenses: vec![WorkspaceLens {
                    relationships: None,
                    id: "canvas".to_string(),
                    title: "Canvas".to_string(),
                    template: None,
                    filter: LensFilter::default(),
                    placements: Vec::new(),
                    visual_boundaries: Vec::new(),
                    semantic_zoom_layers: Vec::new(),
                    semantic_zoom_card_modes: Vec::new(),
                }],
            },
            planning_scenarios: Vec::new(),
        };
        document.validate()?;
        Ok(document)
    }

    pub fn parse_json(source: &str) -> Result<Self, WorkspaceError> {
        let value = serde_json::from_str::<serde_json::Value>(source)
            .map_err(|error| WorkspaceError::InvalidJson(error.to_string()))?;
        let schema_version = value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                WorkspaceError::InvalidJson("schema_version must be an integer".into())
            })?;

        let mut document = match schema_version {
            1 => LegacyWorkspaceDocument::from_value(value)?.into_workspace_document(),
            2..=6 => {
                let mut document: Self = serde_json::from_value(value)
                    .map_err(|error| WorkspaceError::InvalidJson(error.to_string()))?;
                document.schema_version = WORKSPACE_SCHEMA_VERSION;
                document
            }
            version if version == u64::from(WORKSPACE_SCHEMA_VERSION) => {
                serde_json::from_value(value)
                    .map_err(|error| WorkspaceError::InvalidJson(error.to_string()))?
            }
            version => return Err(WorkspaceError::InvalidSchemaVersion(version as u32)),
        };
        document.migrate_semantic_zoom_layer_presentation();
        document.validate()?;
        Ok(document)
    }

    pub fn to_pretty_json(&self) -> Result<String, WorkspaceError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| WorkspaceError::Serialization(error.to_string()))
    }

    fn migrate_semantic_zoom_layer_presentation(&mut self) {
        for version in std::iter::once(&mut self.current_state).chain(
            self.planning_scenarios
                .iter_mut()
                .map(|scenario| &mut scenario.version),
        ) {
            for lens in &mut version.lenses {
                for layer in &mut lens.semantic_zoom_layers {
                    if layer.percent_settings.is_none() {
                        layer.percent_settings = Some(percent_settings_from_legacy_fades(
                            layer.fade_in,
                            layer.fade_out,
                        ));
                    }
                    layer.fade_in = None;
                    layer.fade_out = None;
                }
                for entry in &mut lens.semantic_zoom_card_modes {
                    entry.mode = match &entry.mode {
                        SemanticZoomCardLayerMode::Automatic => {
                            SemanticZoomCardLayerMode::Automatic
                        }
                        SemanticZoomCardLayerMode::Manual { layer_ids } => {
                            SemanticZoomCardLayerMode::Manual {
                                layer_ids: layer_ids.clone(),
                            }
                        }
                        SemanticZoomCardLayerMode::Dedicated { layer_id } => {
                            SemanticZoomCardLayerMode::Manual {
                                layer_ids: vec![layer_id.clone()],
                            }
                        }
                        SemanticZoomCardLayerMode::AlwaysVisible => {
                            SemanticZoomCardLayerMode::Manual {
                                layer_ids: lens
                                    .semantic_zoom_layers
                                    .iter()
                                    .map(|layer| layer.id.clone())
                                    .collect(),
                            }
                        }
                    };
                }
            }
        }
    }

    pub fn validate(&self) -> Result<(), WorkspaceError> {
        if self.schema_version != WORKSPACE_SCHEMA_VERSION {
            return Err(WorkspaceError::InvalidSchemaVersion(self.schema_version));
        }
        validate_required("workspace_id", &self.workspace_id)?;
        validate_required("workspace title", &self.title)?;

        let mut custom_type_ids = BTreeSet::new();
        for custom_type in &self.custom_types {
            validate_required("custom type id", &custom_type.id)?;
            validate_required("custom type title", &custom_type.title)?;
            if ConceptKind::from_type_id(&custom_type.id).is_some() {
                return validation(format!(
                    "custom type id {} is reserved for a core concept type",
                    custom_type.id
                ));
            }
            if !custom_type_ids.insert(custom_type.id.as_str()) {
                return validation(format!("duplicate custom type id: {}", custom_type.id));
            }
            if parse_hex_color(&custom_type.color).is_none() {
                return validation(format!(
                    "custom type {} has invalid color {}; use #RRGGBB",
                    custom_type.id, custom_type.color
                ));
            }
            if custom_type
                .icon
                .as_ref()
                .is_some_and(|icon| icon.trim().is_empty())
            {
                return validation(format!("custom type {} has an empty icon", custom_type.id));
            }
        }

        let mut version_ids = BTreeSet::new();
        validate_version(&self.current_state, &custom_type_ids, "current_state")?;
        version_ids.insert(self.current_state.id.as_str());

        for scenario in &self.planning_scenarios {
            validate_version(&scenario.version, &custom_type_ids, "planning scenario")?;
            if !version_ids.insert(scenario.version.id.as_str()) {
                return validation(format!(
                    "duplicate workspace version id: {}",
                    scenario.version.id
                ));
            }
        }

        let mut shared_content = BTreeMap::new();
        for card in std::iter::once(&self.current_state)
            .chain(
                self.planning_scenarios
                    .iter()
                    .map(|scenario| &scenario.version),
            )
            .flat_map(|version| &version.concepts)
        {
            if let Some(token) = &card.content_reference {
                validate_required("card content reference", token)?;
                let mut content = card.clone();
                content.id = 0;
                if let Some(previous) = shared_content.insert(token, content.clone())
                    && previous != content
                {
                    return validation(format!("shared content {token} is inconsistent"));
                }
            }
        }

        let scenario_bases: BTreeMap<_, _> = self
            .planning_scenarios
            .iter()
            .map(|scenario| {
                (
                    scenario.version.id.as_str(),
                    scenario.based_on_version_id.as_str(),
                )
            })
            .collect();
        for scenario in &self.planning_scenarios {
            validate_required(
                "planning scenario base version id",
                &scenario.based_on_version_id,
            )?;
            if !version_ids.contains(scenario.based_on_version_id.as_str()) {
                return validation(format!(
                    "planning scenario {} is based on unknown version {}",
                    scenario.version.id, scenario.based_on_version_id
                ));
            }
            let mut visited = BTreeSet::new();
            let mut current = scenario.version.id.as_str();
            while let Some(parent) = scenario_bases.get(current) {
                if !visited.insert(current) {
                    return validation(format!(
                        "planning scenarios contain a base-version cycle at {current}"
                    ));
                }
                current = parent;
            }
        }

        Ok(())
    }

    pub fn current_state_id(&self) -> &str {
        &self.current_state.id
    }

    pub fn version(&self, version_id: &str) -> Result<&WorkspaceVersion, WorkspaceError> {
        if self.current_state.id == version_id {
            return Ok(&self.current_state);
        }
        self.planning_scenarios
            .iter()
            .find(|scenario| scenario.version.id == version_id)
            .map(|scenario| &scenario.version)
            .ok_or_else(|| WorkspaceError::MissingVersion(version_id.to_string()))
    }

    pub fn version_title(&self, version_id: &str) -> Result<&str, WorkspaceError> {
        Ok(&self.version(version_id)?.title)
    }

    pub fn concept(
        &self,
        version_id: &str,
        concept_id: u32,
    ) -> Result<&WorkspaceConcept, WorkspaceError> {
        self.version(version_id)?
            .concepts
            .iter()
            .find(|concept| concept.id == concept_id)
            .ok_or_else(|| WorkspaceError::MissingConcept {
                version_id: version_id.to_string(),
                concept_id,
            })
    }

    pub fn relationships_in_lens(
        &self,
        version_id: &str,
        lens_id: &str,
    ) -> Result<&[TypedRelationship], WorkspaceError> {
        let version = self.version(version_id)?;
        let lens = version
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        Ok(lens
            .relationships
            .as_deref()
            .unwrap_or(&version.relationships))
    }

    /// Run an existing graph mutation against one isolated view, then retain
    /// only that view's connections and presentation updates in the candidate.
    fn mutate_lens_relationships<T>(
        &mut self,
        version_id: &str,
        lens_id: &str,
        mutation: impl FnOnce(&mut Self) -> Result<T, WorkspaceError>,
    ) -> Result<T, WorkspaceError> {
        self.validate()?;
        let mut scoped = self.clone();
        let relationships = self.relationships_in_lens(version_id, lens_id)?.to_vec();
        let version = scoped.version_mut(version_id)?;
        let mut lens = version
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .unwrap()
            .clone();
        lens.relationships = None;
        version.relationships = relationships;
        version.lenses = vec![lens];
        let result = mutation(&mut scoped)?;
        let version = scoped.version(version_id)?;
        let mut lens = version.lenses[0].clone();
        lens.relationships = Some(version.relationships.clone());
        let mut candidate = self.clone();
        *candidate.lens_mut(version_id, lens_id)? = lens;
        candidate.validate()?;
        *self = candidate;
        Ok(result)
    }

    pub fn link_concepts_in_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        source_id: u32,
        target_id: u32,
        kind: RelationshipKind,
    ) -> Result<TypedRelationship, WorkspaceError> {
        self.mutate_lens_relationships(version_id, lens_id, |document| {
            document.link_concepts(version_id, source_id, target_id, kind)
        })
    }

    pub fn change_relationship_kind_in_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        relationship_id: &str,
        kind: RelationshipKind,
    ) -> Result<TypedRelationship, WorkspaceError> {
        self.mutate_lens_relationships(version_id, lens_id, |document| {
            document.change_relationship_kind(version_id, relationship_id, kind)
        })
    }

    pub fn remove_relationship_in_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        relationship_id: &str,
    ) -> Result<TypedRelationship, WorkspaceError> {
        self.mutate_lens_relationships(version_id, lens_id, |document| {
            document.remove_relationship(version_id, relationship_id)
        })
    }

    pub fn relationships_for_concept(
        &self,
        version_id: &str,
        concept_id: u32,
    ) -> Result<Vec<&TypedRelationship>, WorkspaceError> {
        let version = self.version(version_id)?;
        if !version
            .concepts
            .iter()
            .any(|concept| concept.id == concept_id)
        {
            return Err(WorkspaceError::MissingConcept {
                version_id: version_id.to_string(),
                concept_id,
            });
        }

        Ok(version
            .relationships
            .iter()
            .filter(|relationship| {
                relationship.source_id == concept_id || relationship.target_id == concept_id
            })
            .collect())
    }

    pub fn preview_concept_deletion(
        &self,
        version_id: &str,
        concept_id: u32,
    ) -> Result<ConceptDeletionPreview, WorkspaceError> {
        self.validate()?;
        let version = self.version(version_id)?;
        let concept = version
            .concepts
            .iter()
            .find(|concept| concept.id == concept_id)
            .ok_or_else(|| WorkspaceError::MissingConcept {
                version_id: version_id.to_string(),
                concept_id,
            })?;

        Ok(ConceptDeletionPreview {
            concept_id,
            title: concept.title.clone(),
            incident_relationship_count: version
                .relationships
                .iter()
                .filter(|relationship| {
                    relationship.source_id == concept_id || relationship.target_id == concept_id
                })
                .count(),
            placement_count: version
                .lenses
                .iter()
                .flat_map(|lens| &lens.placements)
                .filter(|placement| placement.concept_id == concept_id)
                .count(),
        })
    }

    pub fn version_ids(&self) -> Vec<&str> {
        std::iter::once(self.current_state.id.as_str())
            .chain(
                self.planning_scenarios
                    .iter()
                    .map(|scenario| scenario.version.id.as_str()),
            )
            .collect()
    }

    pub fn default_lens_id(&self, version_id: &str) -> Result<&str, WorkspaceError> {
        self.version(version_id)?
            .lenses
            .first()
            .map(|lens| lens.id.as_str())
            .ok_or_else(|| {
                WorkspaceError::Validation(format!("workspace version {version_id} has no lenses"))
            })
    }

    /// Creates a planning version from an existing version. The new version is
    /// a complete snapshot of its source, including its concepts,
    /// relationships, Lenses, and Lens-local presentation data.
    pub fn create_version_from(
        &mut self,
        source_version_id: &str,
        title: String,
    ) -> Result<String, WorkspaceError> {
        self.validate()?;
        validate_required("workspace version title", &title)?;

        let mut candidate = self.clone();
        let mut version = candidate.version(source_version_id)?.clone();
        let version_id = next_workspace_version_id(&title, &candidate);
        for concept in &mut version.concepts {
            concept.content_reference = None;
        }
        version.id = version_id.clone();
        version.title = title;
        candidate.planning_scenarios.push(PlanningScenario {
            based_on_version_id: source_version_id.to_string(),
            version,
        });
        candidate.validate()?;
        *self = candidate;
        Ok(version_id)
    }

    /// Renames a workspace version without changing its graph or its Lens
    /// presentation records. The current version remains the permanent root
    /// version, but its human-readable title can be edited.
    pub fn rename_version(
        &mut self,
        version_id: &str,
        title: String,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        validate_required("workspace version title", &title)?;

        let mut candidate = self.clone();
        candidate.version_mut(version_id)?.title = title;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Removes a planning version. The permanent current version cannot be
    /// removed, and a version that is the source of another planning version
    /// must remain until its dependents are removed.
    pub fn delete_version(&mut self, version_id: &str) -> Result<(), WorkspaceError> {
        self.validate()?;
        if self.current_state.id == version_id {
            return validation("the current workspace version cannot be deleted".to_string());
        }
        if self
            .planning_scenarios
            .iter()
            .any(|scenario| scenario.based_on_version_id == version_id)
        {
            return validation(format!(
                "workspace version {version_id} is the source for another version and cannot be deleted"
            ));
        }

        let mut candidate = self.clone();
        let scenario_index = candidate
            .planning_scenarios
            .iter()
            .position(|scenario| scenario.version.id == version_id)
            .ok_or_else(|| WorkspaceError::MissingVersion(version_id.to_string()))?;
        candidate.planning_scenarios.remove(scenario_index);
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Creates a new Lens by copying an existing Lens in the same version.
    /// A Lens is a saved view, so this retains the source view's filter,
    /// placement, boundaries, and zoom-layer presentation instead of making a
    /// generic empty canvas.
    pub fn create_lens_from(
        &mut self,
        version_id: &str,
        source_lens_id: &str,
        title: String,
    ) -> Result<String, WorkspaceError> {
        self.validate()?;
        validate_required("lens title", &title)?;

        let mut candidate = self.clone();
        let source_lens = candidate
            .version(version_id)?
            .lenses
            .iter()
            .find(|lens| lens.id == source_lens_id)
            .cloned()
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: source_lens_id.to_string(),
            })?;
        let relationships = candidate
            .relationships_in_lens(version_id, source_lens_id)?
            .to_vec();
        let version = candidate.version_mut(version_id)?;
        let lens_id = next_lens_id(&title, &version.lenses);
        let mut lens = source_lens;
        lens.relationships = Some(relationships);
        lens.id = lens_id.clone();
        lens.title = title;
        version.lenses.push(lens);
        candidate.validate()?;
        *self = candidate;
        Ok(lens_id)
    }

    /// Creates a blank Lens in the requested version.
    ///
    /// A Lens is an independently arranged view. A blank Lens deliberately
    /// starts without placements, boundaries, filters, or semantic layers so
    /// the author can compose a new map without inheriting the current one.
    pub fn create_empty_lens(
        &mut self,
        version_id: &str,
        title: String,
    ) -> Result<String, WorkspaceError> {
        self.validate()?;
        validate_required("lens title", &title)?;

        let mut candidate = self.clone();
        let version = candidate.version_mut(version_id)?;
        let lens_id = next_lens_id(&title, &version.lenses);
        version.lenses.push(WorkspaceLens {
            relationships: Some(Vec::new()),
            id: lens_id.clone(),
            title,
            template: None,
            filter: LensFilter::default(),
            placements: Vec::new(),
            visual_boundaries: Vec::new(),
            semantic_zoom_layers: Vec::new(),
            semantic_zoom_card_modes: Vec::new(),
        });
        candidate.validate()?;
        *self = candidate;
        Ok(lens_id)
    }

    /// Renames a Lens without changing its local visual arrangement or the
    /// version's graph facts.
    pub fn rename_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        title: String,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        validate_required("lens title", &title)?;

        let mut candidate = self.clone();
        candidate.lens_mut(version_id, lens_id)?.title = title;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Removes one Lens. Every workspace version keeps at least one Lens so
    /// it always has a valid canvas to open.
    pub fn delete_lens(&mut self, version_id: &str, lens_id: &str) -> Result<(), WorkspaceError> {
        self.validate()?;
        let version = self.version(version_id)?;
        if version.lenses.len() == 1 {
            return validation("a workspace version must keep at least one Lens".to_string());
        }

        let mut candidate = self.clone();
        let version = candidate.version_mut(version_id)?;
        let lens_index = version
            .lenses
            .iter()
            .position(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        version.lenses.remove(lens_index);
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Copies existing cards into one Lens. The copies receive fresh concept
    /// IDs and copied Lens-local placement and display policy. Typed graph
    /// relationships are intentionally never copied: spatial containment is
    /// still represented only by the copied geometry.
    pub fn duplicate_concepts_in_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        concept_ids: &[u32],
        world_delta: Point,
    ) -> Result<Vec<u32>, WorkspaceError> {
        let source = self.clone();
        self.copy_concepts_from_lens(
            &source,
            version_id,
            lens_id,
            version_id,
            lens_id,
            concept_ids,
            world_delta,
        )
    }

    /// Legacy paste default: share content across Views in the same Version,
    /// otherwise import independent captured copies. The app uses an explicit mode.
    #[allow(clippy::too_many_arguments)]
    pub fn copy_concepts_from_lens(
        &mut self,
        source: &Self,
        source_version_id: &str,
        source_lens_id: &str,
        version_id: &str,
        lens_id: &str,
        concept_ids: &[u32],
        world_delta: Point,
    ) -> Result<Vec<u32>, WorkspaceError> {
        let mode = if source.workspace_id == self.workspace_id
            && source_version_id == version_id
            && source_lens_id != lens_id
        {
            CardPasteMode::Reference
        } else {
            CardPasteMode::Copy
        };
        self.paste_concepts_from_lens(
            source,
            source_version_id,
            source_lens_id,
            version_id,
            lens_id,
            concept_ids,
            world_delta,
            mode,
        )
    }

    /// Pastes independent captured copies or explicit live shared-content references.
    #[allow(clippy::too_many_arguments)]
    pub fn paste_concepts_from_lens(
        &mut self,
        source: &Self,
        source_version_id: &str,
        source_lens_id: &str,
        version_id: &str,
        lens_id: &str,
        concept_ids: &[u32],
        world_delta: Point,
        mode: CardPasteMode,
    ) -> Result<Vec<u32>, WorkspaceError> {
        self.validate()?;
        if concept_ids.is_empty() {
            return validation("select at least one card to copy".to_string());
        }
        if !world_delta.x.is_finite() || !world_delta.y.is_finite() {
            return validation("card copy offset must be finite".to_string());
        }

        let requested_ids = concept_ids.iter().copied().collect::<BTreeSet<_>>();
        if requested_ids.len() != concept_ids.len() {
            return validation("a card can be copied only once per action".to_string());
        }

        let mut candidate = self.clone();
        let source_version = source.version(source_version_id)?;
        let source_concepts = concept_ids
            .iter()
            .map(|concept_id| {
                source_version
                    .concepts
                    .iter()
                    .find(|concept| concept.id == *concept_id)
                    .cloned()
                    .ok_or_else(|| WorkspaceError::MissingConcept {
                        version_id: version_id.to_string(),
                        concept_id: *concept_id,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let source_lens = source_version
            .lenses
            .iter()
            .find(|lens| lens.id == source_lens_id)
            .cloned()
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        let source_placements = concept_ids
            .iter()
            .map(|concept_id| {
                source_lens
                    .placements
                    .iter()
                    .find(|placement| placement.concept_id == *concept_id)
                    .cloned()
                    .ok_or_else(|| WorkspaceError::MissingPlacement {
                        version_id: version_id.to_string(),
                        lens_id: lens_id.to_string(),
                        concept_id: *concept_id,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let reference = mode == CardPasteMode::Reference;
        if reference && source.workspace_id != self.workspace_id {
            return validation("References must belong to this workspace.".into());
        }
        let mut next_reference = 1_u64;
        let mut prepared = Vec::new();
        for captured in source_concepts {
            let mut concept = if reference {
                candidate.concept(source_version_id, captured.id)?.clone()
            } else {
                captured
            };
            if reference && concept.content_reference.is_none() {
                let token = loop {
                    let token = format!("content-{next_reference}");
                    next_reference += 1;
                    if !std::iter::once(&candidate.current_state)
                        .chain(
                            candidate
                                .planning_scenarios
                                .iter()
                                .map(|scenario| &scenario.version),
                        )
                        .flat_map(|version| &version.concepts)
                        .any(|card| card.content_reference.as_deref() == Some(&token))
                    {
                        break token;
                    }
                };
                candidate
                    .version_mut(source_version_id)?
                    .concepts
                    .iter_mut()
                    .find(|card| card.id == concept.id)
                    .unwrap()
                    .content_reference = Some(token.clone());
                concept.content_reference = Some(token);
            } else if !reference {
                concept.content_reference = None;
            }
            prepared.push(concept);
        }
        let version = candidate.version_mut(version_id)?;
        let mut next_concept_id = version
            .concepts
            .iter()
            .map(|card| card.id)
            .max()
            .unwrap_or_default()
            .saturating_add(1);
        let mut copied_concept_ids = Vec::new();
        for mut concept in prepared {
            let existing = reference
                .then(|| {
                    version
                        .concepts
                        .iter()
                        .find(|existing| existing.content_reference == concept.content_reference)
                        .map(|card| card.id)
                })
                .flatten();
            if let Some(id) = existing {
                if version
                    .lenses
                    .iter()
                    .find(|lens| lens.id == lens_id)
                    .is_some_and(|lens| {
                        lens.placements
                            .iter()
                            .any(|placement| placement.concept_id == id)
                    })
                {
                    return validation(
                        "This View already contains that reference. Choose Copy for another card."
                            .into(),
                    );
                }
                copied_concept_ids.push(id);
            } else {
                concept.id = next_concept_id;
                next_concept_id = next_concept_id.saturating_add(1);
                copied_concept_ids.push(concept.id);
                version.concepts.push(concept);
            }
        }

        let lens = version
            .lenses
            .iter_mut()
            .find(|lens| lens.id == lens_id)
            .expect("the copied source Lens exists in the candidate version");
        for (source_placement, copied_concept_id) in
            source_placements.iter().zip(&copied_concept_ids)
        {
            lens.placements.push(ConceptPlacement {
                concept_id: *copied_concept_id,
                bounds: Rect::new(
                    source_placement.bounds.origin + world_delta,
                    source_placement.bounds.size,
                ),
            });
        }
        for (source_concept_id, copied_concept_id) in concept_ids.iter().zip(&copied_concept_ids) {
            for layer in &mut lens.semantic_zoom_layers {
                if source_lens.semantic_zoom_layers.iter().any(|source_layer| {
                    source_layer.id == layer.id
                        && source_layer.concept_ids.contains(source_concept_id)
                }) {
                    layer.concept_ids.push(*copied_concept_id);
                }
            }
            if let Some(mode) = source_lens
                .semantic_zoom_card_modes
                .iter()
                .find(|entry| entry.concept_id == *source_concept_id)
                .map(|entry| entry.mode.clone())
                .filter(|mode| match mode {
                    SemanticZoomCardLayerMode::Manual { layer_ids } => layer_ids.iter().all(|id| {
                        lens.semantic_zoom_layers
                            .iter()
                            .any(|layer| &layer.id == id)
                    }),
                    SemanticZoomCardLayerMode::Dedicated { layer_id } => lens
                        .semantic_zoom_layers
                        .iter()
                        .any(|layer| &layer.id == layer_id),
                    _ => true,
                })
            {
                lens.semantic_zoom_card_modes
                    .push(SemanticZoomCardLayerModeEntry {
                        concept_id: *copied_concept_id,
                        mode,
                    });
            }
        }

        candidate.validate()?;
        *self = candidate;
        Ok(copied_concept_ids)
    }

    pub fn lens_title(&self, version_id: &str, lens_id: &str) -> Result<&str, WorkspaceError> {
        self.version(version_id)?
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .map(|lens| lens.title.as_str())
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })
    }

    pub fn lens_ids(&self, version_id: &str) -> Result<Vec<&str>, WorkspaceError> {
        Ok(self
            .version(version_id)?
            .lenses
            .iter()
            .map(|lens| lens.id.as_str())
            .collect())
    }

    pub fn canvas_dataset(
        &self,
        version_id: &str,
        lens_id: &str,
    ) -> Result<CanvasDataset, WorkspaceError> {
        self.validate()?;
        let version = self.version(version_id)?;
        let lens = version
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        let concepts_by_id: BTreeMap<_, _> = version
            .concepts
            .iter()
            .map(|concept| (concept.id, concept))
            .collect();
        let visible_concept_ids: BTreeSet<_> = lens
            .placements
            .iter()
            .filter_map(|placement| {
                let concept = concepts_by_id.get(&placement.concept_id)?;
                lens.filter
                    .includes_concept_type(&concept.type_id)
                    .then_some(placement.concept_id)
            })
            .collect();
        let cards = lens
            .placements
            .iter()
            .filter_map(|placement| {
                let concept = concepts_by_id.get(&placement.concept_id)?;
                lens.filter
                    .includes_concept_type(&concept.type_id)
                    .then(|| ConceptCard {
                        id: concept.id,
                        title: concept.title.clone(),
                        summary: concept.summary.clone(),
                        kind: ConceptKind::from_type_id(&concept.type_id)
                            .unwrap_or(ConceptKind::Project),
                        accent_color: self.custom_type_color(&concept.type_id),
                        bounds: placement.bounds,
                    })
            })
            .collect();
        let relationships = lens
            .relationships
            .as_deref()
            .unwrap_or(&version.relationships)
            .iter()
            .filter(|relationship| {
                visible_concept_ids.contains(&relationship.source_id)
                    && visible_concept_ids.contains(&relationship.target_id)
                    && lens.filter.includes_relationship_kind(relationship.kind)
            })
            .map(|relationship| Relationship {
                source_id: relationship.source_id,
                target_id: relationship.target_id,
            })
            .collect();

        Ok(CanvasDataset::from_records(cards, relationships))
    }

    /// Resolves the active Lens into cards and relationships at one canvas scale.
    ///
    /// Layers do not duplicate concepts or relationships. When no layers are
    /// configured, this returns the complete filtered Lens at its stored card
    /// sizes. A card's assigned layer is its base level: the card stays visible
    /// whenever that layer or a deeper layer is active and keeps its base
    /// layer's card scale.
    pub fn semantic_zoom_frame(
        &self,
        version_id: &str,
        lens_id: &str,
        scale: f32,
    ) -> Result<SemanticZoomFrame, WorkspaceError> {
        self.semantic_zoom_frame_with_relationship_rollup(version_id, lens_id, scale, false)
    }

    pub fn semantic_zoom_frame_with_relationship_rollup(
        &self,
        version_id: &str,
        lens_id: &str,
        scale: f32,
        roll_up_relationships: bool,
    ) -> Result<SemanticZoomFrame, WorkspaceError> {
        self.validate()?;
        let version = self.version(version_id)?;
        let lens = version
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        let concepts_by_id: BTreeMap<_, _> = version
            .concepts
            .iter()
            .map(|concept| (concept.id, concept))
            .collect();
        let placements_by_concept_id: BTreeMap<_, _> = lens
            .placements
            .iter()
            .map(|placement| (placement.concept_id, placement.bounds))
            .collect();

        let candidates: BTreeMap<_, _> = placements_by_concept_id
            .iter()
            .filter_map(|(concept_id, bounds)| {
                let concept = concepts_by_id.get(concept_id)?;
                lens.filter
                    .includes_concept_type(&concept.type_id)
                    .then_some((*concept_id, (*concept, *bounds)))
            })
            .collect();

        let layers = lens
            .semantic_zoom_layers
            .iter()
            .map(|layer| SemanticZoomLayerProjection {
                id: layer.id.clone(),
                title: layer.title.clone(),
                opacity: layer_opacity_at_scale(layer, scale),
            })
            .collect::<Vec<_>>();

        let mut presentation_by_concept_id: BTreeMap<u32, (f32, f32)> = BTreeMap::new();
        if lens.semantic_zoom_layers.is_empty() {
            for concept_id in candidates.keys() {
                presentation_by_concept_id.insert(*concept_id, (1.0, 1.0));
            }
        } else {
            for (concept_id, (_, bounds)) in &candidates {
                if let Some((opacity, card_scale)) =
                    card_layer_presentation(lens, *concept_id, *bounds, &layers)
                {
                    presentation_by_concept_id.insert(*concept_id, (opacity, card_scale));
                }
            }
        }

        // A card that spatially contains a visible card remains visual context
        // at deeper semantic layers. This is presentation-only containment: it
        // does not create a relationship or change either card's layer policy.
        let parent_by_child = card_parent_ids(
            candidates
                .iter()
                .map(|(concept_id, (_, bounds))| (*concept_id, *bounds)),
        );
        let visible_card_ids = presentation_by_concept_id
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for visible_card_id in visible_card_ids {
            let Some(child_opacity) = presentation_by_concept_id
                .get(&visible_card_id)
                .map(|(opacity, _)| *opacity)
            else {
                continue;
            };
            let mut child_id = visible_card_id;
            while let Some(parent_id) = parent_by_child.get(&child_id).copied() {
                let parent_bounds = candidates[&parent_id].1;
                let parent_scale = assigned_card_scale(lens, parent_id, parent_bounds, &layers);
                let parent_presentation = presentation_by_concept_id
                    .entry(parent_id)
                    .or_insert((child_opacity, parent_scale));
                parent_presentation.0 = parent_presentation.0.max(child_opacity);
                child_id = parent_id;
            }
        }

        let parent_by_child =
            card_parent_ids(candidates.iter().map(|(id, (_, bounds))| (*id, *bounds)));
        let mut relationship_endpoints_by_id = BTreeMap::new();
        let mut relationship_opacity_by_id = BTreeMap::new();
        for relationship in lens
            .relationships
            .as_deref()
            .unwrap_or(&version.relationships)
        {
            if !lens.filter.includes_relationship_kind(relationship.kind)
                || !candidates.contains_key(&relationship.source_id)
                || !candidates.contains_key(&relationship.target_id)
            {
                continue;
            }
            let layer_opacity = if lens.semantic_zoom_layers.is_empty() {
                1.0
            } else {
                lens.semantic_zoom_layers
                    .iter()
                    .position(|layer| layer.relationship_ids.contains(&relationship.id))
                    .map(|layer_index| context_opacity_for_layer(&layers, layer_index))
                    .unwrap_or_default()
            };
            let endpoint = |mut id| {
                if roll_up_relationships {
                    while presentation_by_concept_id
                        .get(&id)
                        .is_none_or(|(opacity, _)| *opacity <= 0.0)
                    {
                        let parent = parent_by_child.get(&id)?;
                        id = *parent;
                    }
                }
                presentation_by_concept_id
                    .get(&id)
                    .is_some_and(|(opacity, _)| *opacity > 0.0)
                    .then_some(id)
            };
            let Some(source_id) = endpoint(relationship.source_id) else {
                continue;
            };
            let Some(target_id) = endpoint(relationship.target_id) else {
                continue;
            };
            if source_id == target_id {
                continue;
            }
            relationship_endpoints_by_id.insert(relationship.id.as_str(), (source_id, target_id));
            let source_opacity = presentation_by_concept_id
                .get(&source_id)
                .map(|(opacity, _)| *opacity)
                .unwrap_or_default();
            let target_opacity = presentation_by_concept_id
                .get(&target_id)
                .map(|(opacity, _)| *opacity)
                .unwrap_or_default();
            let relationship_opacity = layer_opacity.max(source_opacity).max(target_opacity);
            if relationship_opacity <= 0.0 {
                continue;
            }
            relationship_opacity_by_id.insert(relationship.id.as_str(), relationship_opacity);
        }

        let cards = candidates
            .into_iter()
            .filter_map(|(concept_id, (concept, bounds))| {
                let (opacity, card_scale) = presentation_by_concept_id.get(&concept_id).copied()?;
                Some(SemanticZoomCard {
                    card: ConceptCard {
                        id: concept.id,
                        title: concept.title.clone(),
                        summary: concept.summary.clone(),
                        kind: ConceptKind::from_type_id(&concept.type_id)
                            .unwrap_or(ConceptKind::Project),
                        accent_color: self.custom_type_color(&concept.type_id),
                        bounds: scale_bounds_from_center(bounds, card_scale),
                    },
                    opacity,
                    card_scale,
                })
            })
            .collect::<Vec<_>>();
        let relationships = lens
            .relationships
            .as_deref()
            .unwrap_or(&version.relationships)
            .iter()
            .filter_map(|relationship| {
                let opacity = relationship_opacity_by_id.get(relationship.id.as_str())?;
                Some(SemanticZoomRelationship {
                    id: relationship.id.clone(),
                    kind: relationship.kind,
                    label: relationship.label.clone(),
                    source_id: relationship_endpoints_by_id[relationship.id.as_str()].0,
                    target_id: relationship_endpoints_by_id[relationship.id.as_str()].1,
                    opacity: *opacity,
                })
            })
            .collect();

        Ok(SemanticZoomFrame {
            cards,
            relationships,
            layers,
        })
    }

    /// Creates one empty semantic zoom layer in the requested Lens.
    pub fn create_semantic_zoom_layer(
        &mut self,
        version_id: &str,
        lens_id: &str,
        title: String,
    ) -> Result<String, WorkspaceError> {
        self.validate()?;
        validate_required("semantic zoom layer title", &title)?;
        let mut candidate = self.clone();
        let id = {
            let lens = candidate.lens_mut(version_id, lens_id)?;
            let id = next_semantic_zoom_layer_id(&title, &lens.semantic_zoom_layers);
            let percent_settings = default_percent_settings(lens.semantic_zoom_layers.len());
            lens.semantic_zoom_layers.push(SemanticZoomLayer {
                id: id.clone(),
                title,
                percent_settings: Some(percent_settings),
                fade_in: None,
                fade_out: None,
                card_scale: 1.0,
                concept_ids: Vec::new(),
                relationship_ids: Vec::new(),
            });
            id
        };
        candidate.validate()?;
        *self = candidate;
        Ok(id)
    }

    /// Inserts a new layer at an authored position, retaining the new range
    /// and shifting later ranges only when required to make room.
    pub fn insert_semantic_zoom_layer(
        &mut self,
        version_id: &str,
        lens_id: &str,
        layer_id: &str,
        index: usize,
    ) -> Result<(), WorkspaceError> {
        let mut candidate = self.clone();
        let layers = &mut candidate
            .lens_mut(version_id, lens_id)?
            .semantic_zoom_layers;
        let source = layers
            .iter()
            .position(|layer| layer.id == layer_id)
            .ok_or_else(|| WorkspaceError::MissingSemanticZoomLayer {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                layer_id: layer_id.to_string(),
            })?;
        let layer = layers.remove(source);
        if index > layers.len() {
            return validation("Layer insertion index is outside the layer list".to_string());
        }
        let mut next_start = effective_percent_settings(&layer)
            .active_zoom_range
            .end_percent;
        layers.insert(index, layer);
        for layer in layers.iter_mut().skip(index + 1) {
            let mut settings = layer.percent_settings.unwrap_or_else(|| {
                percent_settings_from_legacy_fades(layer.fade_in, layer.fade_out)
            });
            let width =
                settings.active_zoom_range.end_percent - settings.active_zoom_range.start_percent;
            if settings.active_zoom_range.start_percent < next_start {
                settings.active_zoom_range = ZoomPercentRange {
                    start_percent: next_start,
                    end_percent: next_start + width,
                };
                layer.percent_settings = Some(settings);
                layer.fade_in = None;
                layer.fade_out = None;
            }
            next_start = settings.active_zoom_range.end_percent;
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Creates a named visual boundary in one Lens without changing the graph.
    pub fn create_visual_boundary(
        &mut self,
        version_id: &str,
        lens_id: &str,
        title: String,
        summary: String,
        bounds: Rect,
    ) -> Result<String, WorkspaceError> {
        self.validate()?;
        validate_required("visual boundary title", &title)?;
        if !has_valid_bounds(bounds) {
            return validation("visual boundary has invalid bounds".to_string());
        }
        let mut candidate = self.clone();
        let id = {
            let lens = candidate.lens_mut(version_id, lens_id)?;
            let id = next_visual_boundary_id(&title, &lens.visual_boundaries);
            lens.visual_boundaries.push(VisualBoundary {
                layer_id: None,
                id: id.clone(),
                title,
                summary,
                bounds,
            });
            id
        };
        candidate.validate()?;
        *self = candidate;
        Ok(id)
    }

    /// Assigns a visual boundary to a Lens-local layer; None keeps it as master context.
    pub fn set_visual_boundary_layer(
        &mut self,
        version_id: &str,
        lens_id: &str,
        boundary_id: &str,
        layer_id: Option<String>,
    ) -> Result<(), WorkspaceError> {
        let mut candidate = self.clone();
        let boundary = candidate
            .lens_mut(version_id, lens_id)?
            .visual_boundaries
            .iter_mut()
            .find(|boundary| boundary.id == boundary_id)
            .ok_or_else(|| WorkspaceError::MissingVisualBoundary {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                boundary_id: boundary_id.to_string(),
            })?;
        boundary.layer_id = layer_id;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Updates the readable label and optional context for one visual boundary.
    /// This changes only Lens-local presentation data.
    pub fn update_visual_boundary(
        &mut self,
        version_id: &str,
        lens_id: &str,
        boundary_id: &str,
        title: String,
        summary: String,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        validate_required("visual boundary title", &title)?;
        let mut candidate = self.clone();
        let boundary = candidate
            .lens_mut(version_id, lens_id)?
            .visual_boundaries
            .iter_mut()
            .find(|boundary| boundary.id == boundary_id)
            .ok_or_else(|| WorkspaceError::MissingVisualBoundary {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                boundary_id: boundary_id.to_string(),
            })?;
        boundary.title = title;
        boundary.summary = summary;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Moves one visual boundary and every fully enclosed card and boundary.
    /// The move stays Lens-local and does not change graph facts.
    pub fn move_visual_boundary(
        &mut self,
        version_id: &str,
        lens_id: &str,
        boundary_id: &str,
        world_delta: Point,
    ) -> Result<(), WorkspaceError> {
        self.move_visual_boundary_with_contained_concepts(
            version_id,
            lens_id,
            boundary_id,
            world_delta,
        )
        .map(|_| ())
    }

    /// Moves a visual boundary and returns the card IDs that traveled with it.
    ///
    /// Membership is resolved from the boundary's bounds before the move. A
    /// card or nested boundary must be fully enclosed, so an item merely
    /// crossing a boundary edge remains independent.
    pub fn move_visual_boundary_with_contained_concepts(
        &mut self,
        version_id: &str,
        lens_id: &str,
        boundary_id: &str,
        world_delta: Point,
    ) -> Result<Vec<u32>, WorkspaceError> {
        self.move_canvas_items_in_lens(
            version_id,
            lens_id,
            &[CanvasItemId::Boundary(boundary_id.to_string())],
            world_delta,
        )
        .map(|item_ids| {
            item_ids
                .into_iter()
                .filter_map(|item_id| match item_id {
                    CanvasItemId::Card(concept_id) => Some(concept_id),
                    CanvasItemId::Boundary(_) => None,
                })
                .collect()
        })
    }

    /// Sets one visual boundary's rectangle without changing card placement or
    /// graph facts.
    pub fn resize_visual_boundary(
        &mut self,
        version_id: &str,
        lens_id: &str,
        boundary_id: &str,
        bounds: Rect,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        if !has_valid_bounds(bounds) {
            return validation("visual boundary has invalid bounds".to_string());
        }
        let mut candidate = self.clone();
        let boundary = candidate
            .lens_mut(version_id, lens_id)?
            .visual_boundaries
            .iter_mut()
            .find(|boundary| boundary.id == boundary_id)
            .ok_or_else(|| WorkspaceError::MissingVisualBoundary {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                boundary_id: boundary_id.to_string(),
            })?;
        boundary.bounds = bounds;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Removes one Lens-local visual boundary without changing graph facts.
    pub fn remove_visual_boundary(
        &mut self,
        version_id: &str,
        lens_id: &str,
        boundary_id: &str,
    ) -> Result<VisualBoundary, WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let boundaries = &mut candidate.lens_mut(version_id, lens_id)?.visual_boundaries;
        let index = boundaries
            .iter()
            .position(|boundary| boundary.id == boundary_id)
            .ok_or_else(|| WorkspaceError::MissingVisualBoundary {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                boundary_id: boundary_id.to_string(),
            })?;
        let removed = boundaries.remove(index);
        candidate.validate()?;
        *self = candidate;
        Ok(removed)
    }

    /// Changes the presentation controls for one semantic zoom layer.
    pub fn update_semantic_zoom_layer(
        &mut self,
        version_id: &str,
        lens_id: &str,
        layer_id: &str,
        settings: SemanticZoomLayerSettings,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let layer = candidate
            .lens_mut(version_id, lens_id)?
            .semantic_zoom_layers
            .iter_mut()
            .find(|layer| layer.id == layer_id)
            .ok_or_else(|| WorkspaceError::MissingSemanticZoomLayer {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                layer_id: layer_id.to_string(),
            })?;
        layer.title = settings.title;
        if settings.fade_in.is_some() || settings.fade_out.is_some() {
            layer.percent_settings = Some(percent_settings_from_legacy_fades(
                settings.fade_in,
                settings.fade_out,
            ));
        }
        layer.fade_in = None;
        layer.fade_out = None;
        layer.card_scale = settings.card_scale;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Changes a layer's user-facing active zoom range and symmetric fade.
    pub fn update_semantic_zoom_layer_percent_settings(
        &mut self,
        version_id: &str,
        lens_id: &str,
        layer_id: &str,
        title: String,
        percent_settings: SemanticZoomLayerPercentSettings,
        card_scale: f32,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let layer = candidate
            .lens_mut(version_id, lens_id)?
            .semantic_zoom_layers
            .iter_mut()
            .find(|layer| layer.id == layer_id)
            .ok_or_else(|| WorkspaceError::MissingSemanticZoomLayer {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                layer_id: layer_id.to_string(),
            })?;
        layer.title = title;
        layer.percent_settings = Some(percent_settings);
        layer.fade_in = None;
        layer.fade_out = None;
        layer.card_scale = card_scale;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Moves one saved zoom layer to a new position in its Lens presentation
    /// order. Existing zoom ranges stay in their ordinal slots; layer IDs,
    /// membership, fade and card scale move with the layer.
    pub fn reorder_semantic_zoom_layer(
        &mut self,
        version_id: &str,
        lens_id: &str,
        layer_id: &str,
        destination_index: usize,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let layers = &mut candidate
            .lens_mut(version_id, lens_id)?
            .semantic_zoom_layers;
        let source_index = layers
            .iter()
            .position(|layer| layer.id == layer_id)
            .ok_or_else(|| WorkspaceError::MissingSemanticZoomLayer {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                layer_id: layer_id.to_string(),
            })?;
        if destination_index >= layers.len() {
            return validation(format!(
                "semantic zoom layer {} in lens {} has an invalid destination index {}",
                layer_id, lens_id, destination_index
            ));
        }
        if source_index != destination_index {
            // Reordering changes which layer occupies each range, rather than
            // trying to fit the dragged range outside the target. This also
            // handles the first slot, unequal widths, gaps and overlapping ranges.
            let ranges: Vec<_> = layers
                .iter()
                .map(|layer| {
                    layer
                        .percent_settings
                        .unwrap_or_else(|| {
                            percent_settings_from_legacy_fades(layer.fade_in, layer.fade_out)
                        })
                        .active_zoom_range
                })
                .collect();
            let layer = layers.remove(source_index);
            layers.insert(destination_index, layer);
            for (layer, range) in layers.iter_mut().zip(ranges) {
                let mut settings = layer.percent_settings.unwrap_or_else(|| {
                    percent_settings_from_legacy_fades(layer.fade_in, layer.fade_out)
                });
                settings.active_zoom_range = range;
                layer.percent_settings = Some(settings);
                layer.fade_in = None;
                layer.fade_out = None;
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Returns the persisted percent settings for a named Lens layer.
    pub fn semantic_zoom_layer_percent_settings(
        &self,
        version_id: &str,
        lens_id: &str,
        layer_id: &str,
    ) -> Result<SemanticZoomLayerPercentSettings, WorkspaceError> {
        let layer = self
            .version(version_id)?
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?
            .semantic_zoom_layers
            .iter()
            .find(|layer| layer.id == layer_id)
            .ok_or_else(|| WorkspaceError::MissingSemanticZoomLayer {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                layer_id: layer_id.to_string(),
            })?;
        Ok(layer
            .percent_settings
            .unwrap_or_else(|| percent_settings_from_legacy_fades(layer.fade_in, layer.fade_out)))
    }

    /// Adds or removes concepts from one semantic zoom layer.
    pub fn set_semantic_zoom_layer_concepts(
        &mut self,
        version_id: &str,
        lens_id: &str,
        layer_id: &str,
        concept_ids: &[u32],
        included: bool,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let version = candidate.version(version_id)?;
        let known_concept_ids: BTreeSet<_> =
            version.concepts.iter().map(|concept| concept.id).collect();
        if concept_ids
            .iter()
            .any(|concept_id| !known_concept_ids.contains(concept_id))
        {
            return validation("semantic zoom layer references an unknown concept".to_string());
        }
        let lens = candidate.lens_mut(version_id, lens_id)?;
        if included {
            for other_layer in &mut lens.semantic_zoom_layers {
                if other_layer.id != layer_id {
                    other_layer
                        .concept_ids
                        .retain(|member_id| !concept_ids.contains(member_id));
                }
            }
            lens.semantic_zoom_card_modes
                .retain(|entry| !concept_ids.contains(&entry.concept_id));
        }
        let layer = lens
            .semantic_zoom_layers
            .iter_mut()
            .find(|layer| layer.id == layer_id)
            .ok_or_else(|| WorkspaceError::MissingSemanticZoomLayer {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                layer_id: layer_id.to_string(),
            })?;
        set_membership(&mut layer.concept_ids, concept_ids, included);
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Sets one card's Lens-local layer presentation policy.
    ///
    /// `Automatic` clears any explicit layer selection. `Manual` saves one or
    /// more selected layers. Both policies clear the historical base-layer
    /// membership because the policy is the single source of card display.
    pub fn set_semantic_zoom_card_layer_mode(
        &mut self,
        version_id: &str,
        lens_id: &str,
        concept_id: u32,
        mode: SemanticZoomCardLayerMode,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let version = candidate.version(version_id)?;
        if !version
            .concepts
            .iter()
            .any(|concept| concept.id == concept_id)
        {
            return Err(WorkspaceError::MissingConcept {
                version_id: version_id.to_string(),
                concept_id,
            });
        }
        let lens = candidate.lens_mut(version_id, lens_id)?;
        if !lens
            .placements
            .iter()
            .any(|placement| placement.concept_id == concept_id)
        {
            return Err(WorkspaceError::MissingPlacement {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                concept_id,
            });
        }
        let mode = normalized_card_layer_mode(mode, &lens.semantic_zoom_layers)?;
        for layer in &mut lens.semantic_zoom_layers {
            layer
                .concept_ids
                .retain(|member_id| *member_id != concept_id);
        }
        lens.semantic_zoom_card_modes
            .retain(|entry| entry.concept_id != concept_id);
        if !matches!(mode, SemanticZoomCardLayerMode::Automatic) {
            lens.semantic_zoom_card_modes
                .push(SemanticZoomCardLayerModeEntry { concept_id, mode });
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Returns the effective saved card policy. Cards without an entry are
    /// automatic and use their Lens-local dimensions to choose a layer.
    pub fn semantic_zoom_card_layer_mode(
        &self,
        version_id: &str,
        lens_id: &str,
        concept_id: u32,
    ) -> Result<SemanticZoomCardLayerMode, WorkspaceError> {
        let lens = self
            .version(version_id)?
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        if !lens
            .placements
            .iter()
            .any(|placement| placement.concept_id == concept_id)
        {
            return Err(WorkspaceError::MissingPlacement {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                concept_id,
            });
        }
        Ok(card_layer_mode(lens, concept_id))
    }

    /// Resolves the saved layer visibility for a card at its stored Lens-local
    /// bounds. The result states both the card policy and the authored layers
    /// in which the card remains available as context.
    pub fn semantic_zoom_card_layer_visibility(
        &self,
        version_id: &str,
        lens_id: &str,
        concept_id: u32,
    ) -> Result<SemanticZoomCardLayerVisibility, WorkspaceError> {
        let lens = self
            .version(version_id)?
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        let bounds = lens
            .placements
            .iter()
            .find(|placement| placement.concept_id == concept_id)
            .map(|placement| placement.bounds)
            .ok_or_else(|| WorkspaceError::MissingPlacement {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                concept_id,
            })?;
        Ok(card_layer_visibility_at_bounds(lens, concept_id, bounds))
    }

    /// Resolves a card's layer visibility for proposed Lens-local bounds
    /// without changing the workspace. The application uses this to preview a
    /// size-driven layer change before a resize is saved.
    pub fn semantic_zoom_card_layer_visibility_at_bounds(
        &self,
        version_id: &str,
        lens_id: &str,
        concept_id: u32,
        bounds: Rect,
    ) -> Result<SemanticZoomCardLayerVisibility, WorkspaceError> {
        let lens = self
            .version(version_id)?
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        if !lens
            .placements
            .iter()
            .any(|placement| placement.concept_id == concept_id)
        {
            return Err(WorkspaceError::MissingPlacement {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                concept_id,
            });
        }
        Ok(card_layer_visibility_at_bounds(lens, concept_id, bounds))
    }

    /// Adds or removes relationships from one semantic zoom layer.
    pub fn set_semantic_zoom_layer_relationships(
        &mut self,
        version_id: &str,
        lens_id: &str,
        layer_id: &str,
        relationship_ids: &[String],
        included: bool,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let known_relationship_ids: BTreeSet<_> = candidate
            .relationships_in_lens(version_id, lens_id)?
            .iter()
            .map(|relationship| relationship.id.as_str())
            .collect();
        if relationship_ids
            .iter()
            .any(|relationship_id| !known_relationship_ids.contains(relationship_id.as_str()))
        {
            return validation(
                "semantic zoom layer references an unknown relationship".to_string(),
            );
        }
        let layers = &mut candidate
            .lens_mut(version_id, lens_id)?
            .semantic_zoom_layers;
        if included {
            for other_layer in layers.iter_mut() {
                if other_layer.id != layer_id {
                    other_layer
                        .relationship_ids
                        .retain(|member_id| !relationship_ids.contains(member_id));
                }
            }
        }
        let layer = layers
            .iter_mut()
            .find(|layer| layer.id == layer_id)
            .ok_or_else(|| WorkspaceError::MissingSemanticZoomLayer {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                layer_id: layer_id.to_string(),
            })?;
        set_membership(&mut layer.relationship_ids, relationship_ids, included);
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Removes one semantic zoom layer without changing the shared graph.
    pub fn delete_semantic_zoom_layer(
        &mut self,
        version_id: &str,
        lens_id: &str,
        layer_id: &str,
    ) -> Result<SemanticZoomLayer, WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let layers = &mut candidate
            .lens_mut(version_id, lens_id)?
            .semantic_zoom_layers;
        let index = layers
            .iter()
            .position(|layer| layer.id == layer_id)
            .ok_or_else(|| WorkspaceError::MissingSemanticZoomLayer {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                layer_id: layer_id.to_string(),
            })?;
        let removed = layers.remove(index);
        if let Some(reassignment_layer) = layers.get_mut(index.saturating_sub(1)) {
            reassignment_layer
                .concept_ids
                .extend(removed.concept_ids.iter().copied());
            reassignment_layer
                .relationship_ids
                .extend(removed.relationship_ids.iter().cloned());
        }
        let lens = candidate.lens_mut(version_id, lens_id)?;
        for boundary in &mut lens.visual_boundaries {
            if boundary.layer_id.as_deref() == Some(layer_id) {
                boundary.layer_id = None;
            }
        }
        for entry in &mut lens.semantic_zoom_card_modes {
            if let SemanticZoomCardLayerMode::Manual { layer_ids } = &mut entry.mode {
                layer_ids.retain(|selected_layer_id| selected_layer_id != layer_id);
                if layer_ids.is_empty() {
                    entry.mode = SemanticZoomCardLayerMode::Automatic;
                }
            }
        }
        lens.semantic_zoom_card_modes
            .retain(|entry| !matches!(entry.mode, SemanticZoomCardLayerMode::Automatic));
        candidate.validate()?;
        *self = candidate;
        Ok(removed)
    }

    /// Returns only stored relationships incident to the selected concept.
    ///
    /// The query keeps unavailable endpoints explicit instead of fabricating
    /// card bounds. Use [`Self::selected_concept_relationship_paths`] for
    /// edge-to-edge geometry for the visible source-target pairs only.
    pub fn selected_concept_relationship_overlay(
        &self,
        version_id: &str,
        lens_id: &str,
        selected_concept_id: u32,
        viewport: Rect,
    ) -> Result<SelectedConceptRelationshipOverlay, WorkspaceError> {
        self.selected_concept_relationship_overlay_with_placement_overrides(
            version_id,
            lens_id,
            selected_concept_id,
            viewport,
            &RelationshipOverlayPlacementOverrides::default(),
        )
    }

    /// Returns a selected-concept overlay using non-persistent placement
    /// bounds, such as an active drag preview.
    pub fn selected_concept_relationship_overlay_with_placement_overrides(
        &self,
        version_id: &str,
        lens_id: &str,
        selected_concept_id: u32,
        viewport: Rect,
        placement_overrides: &RelationshipOverlayPlacementOverrides,
    ) -> Result<SelectedConceptRelationshipOverlay, WorkspaceError> {
        Ok(self
            .selected_concept_relationship_overlay_query(
                version_id,
                lens_id,
                selected_concept_id,
                viewport,
                placement_overrides,
            )?
            .overlay)
    }

    /// Returns edge-to-edge, collision-avoiding paths for visible selected
    /// concept relationships.
    pub fn selected_concept_relationship_paths(
        &self,
        version_id: &str,
        lens_id: &str,
        selected_concept_id: u32,
        viewport: Rect,
    ) -> Result<Vec<SelectedConceptRelationshipOverlayPath>, WorkspaceError> {
        self.selected_concept_relationship_paths_with_placement_overrides(
            version_id,
            lens_id,
            selected_concept_id,
            viewport,
            &RelationshipOverlayPlacementOverrides::default(),
        )
    }

    /// Returns edge-to-edge relationship paths using non-persistent placement
    /// bounds, such as an active drag preview.
    pub fn selected_concept_relationship_paths_with_placement_overrides(
        &self,
        version_id: &str,
        lens_id: &str,
        selected_concept_id: u32,
        viewport: Rect,
        placement_overrides: &RelationshipOverlayPlacementOverrides,
    ) -> Result<Vec<SelectedConceptRelationshipOverlayPath>, WorkspaceError> {
        let query = self.selected_concept_relationship_overlay_query(
            version_id,
            lens_id,
            selected_concept_id,
            viewport,
            placement_overrides,
        )?;

        Ok(query
            .overlay
            .visible_paths(&query.visible_card_bounds_by_concept_id))
    }

    /// Returns selected-concept paths using the final bounds of displayed
    /// cards. Each supplied bound must use one coordinate system. Endpoints
    /// absent from the displayed bounds do not produce a path.
    pub fn selected_concept_relationship_paths_with_displayed_bounds(
        &self,
        version_id: &str,
        lens_id: &str,
        selected_concept_id: u32,
        viewport: Rect,
        placement_overrides: &RelationshipOverlayPlacementOverrides,
        displayed_card_bounds_by_concept_id: &BTreeMap<u32, Rect>,
    ) -> Result<Vec<SelectedConceptRelationshipOverlayPath>, WorkspaceError> {
        let query = self.selected_concept_relationship_overlay_query(
            version_id,
            lens_id,
            selected_concept_id,
            viewport,
            placement_overrides,
        )?;

        Ok(query
            .overlay
            .visible_paths(displayed_card_bounds_by_concept_id))
    }

    fn selected_concept_relationship_overlay_query(
        &self,
        version_id: &str,
        lens_id: &str,
        selected_concept_id: u32,
        viewport: Rect,
        placement_overrides: &RelationshipOverlayPlacementOverrides,
    ) -> Result<SelectedConceptRelationshipOverlayQuery, WorkspaceError> {
        let version = self.version(version_id)?;
        let lens = version
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        let concepts_by_id: BTreeMap<_, _> = version
            .concepts
            .iter()
            .map(|concept| (concept.id, concept))
            .collect();
        if !concepts_by_id.contains_key(&selected_concept_id) {
            return Err(WorkspaceError::MissingConcept {
                version_id: version_id.to_string(),
                concept_id: selected_concept_id,
            });
        }
        let placements_by_concept_id: BTreeMap<_, _> = lens
            .placements
            .iter()
            .map(|placement| {
                (
                    placement.concept_id,
                    placement_overrides.effective_bounds(placement.concept_id, placement.bounds),
                )
            })
            .collect();
        let viewport = viewport.normalized();
        let selected_concept = overlay_source_state(
            concepts_by_id[&selected_concept_id],
            placements_by_concept_id.get(&selected_concept_id).copied(),
            &lens.filter,
            viewport,
        );

        let mut relationships: Vec<_> = lens
            .relationships
            .as_deref()
            .unwrap_or(&version.relationships)
            .iter()
            .filter(|relationship| {
                relationship.source_id == selected_concept_id
                    || relationship.target_id == selected_concept_id
            })
            .map(|relationship| {
                let related_concept_id = if relationship.source_id == selected_concept_id {
                    relationship.target_id
                } else {
                    relationship.source_id
                };
                let target = if lens.filter.includes_relationship_kind(relationship.kind) {
                    overlay_target_state(
                        concepts_by_id.get(&related_concept_id).copied(),
                        placements_by_concept_id.get(&related_concept_id).copied(),
                        &lens.filter,
                        viewport,
                    )
                } else {
                    RelationshipOverlayTarget::Filtered
                };

                SelectedConceptRelationshipOverlayEntry {
                    relationship_id: relationship.id.clone(),
                    relationship_kind: relationship.kind,
                    source_id: relationship.source_id,
                    target_id: relationship.target_id,
                    related_concept_id,
                    target,
                }
            })
            .collect();
        relationships.sort_by(|left, right| left.relationship_id.cmp(&right.relationship_id));

        let visible_card_bounds_by_concept_id = concepts_by_id
            .iter()
            .filter_map(|(concept_id, concept)| {
                if !lens.filter.includes_concept_type(&concept.type_id) {
                    return None;
                }

                placements_by_concept_id
                    .get(concept_id)
                    .copied()
                    .filter(|bounds| bounds.intersects(viewport))
                    .map(Rect::normalized)
                    .map(|bounds| (*concept_id, bounds))
            })
            .collect();

        Ok(SelectedConceptRelationshipOverlayQuery {
            overlay: SelectedConceptRelationshipOverlay {
                selected_concept,
                relationships,
            },
            visible_card_bounds_by_concept_id,
        })
    }

    pub fn move_concept_in_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        concept_id: u32,
        world_delta: Point,
    ) -> Result<(), WorkspaceError> {
        self.move_concepts_in_lens(version_id, lens_id, &[concept_id], world_delta)
    }

    /// Sets one card's Lens-local rectangle without changing the shared
    /// concept or any graph relationship.
    pub fn resize_concept_in_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        concept_id: u32,
        bounds: Rect,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        if !has_valid_bounds(bounds) {
            return validation("concept placement has invalid bounds".to_string());
        }
        let mut candidate = self.clone();
        let placement = candidate
            .lens_mut(version_id, lens_id)?
            .placements
            .iter_mut()
            .find(|placement| placement.concept_id == concept_id)
            .ok_or_else(|| WorkspaceError::MissingPlacement {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                concept_id,
            })?;
        placement.bounds = bounds;
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Moves a card to an adjacent authored layer and resizes it to that
    /// layer's natural world size while retaining aspect ratio and center.
    pub fn shift_concept_layer(
        &mut self,
        version_id: &str,
        lens_id: &str,
        concept_id: u32,
        deeper: bool,
    ) -> Result<(), WorkspaceError> {
        let visibility =
            self.semantic_zoom_card_layer_visibility(version_id, lens_id, concept_id)?;
        let lens = self
            .version(version_id)?
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })?;
        let current = lens
            .semantic_zoom_layers
            .iter()
            .position(|layer| Some(&layer.id) == visibility.base_layer_id.as_ref())
            .ok_or_else(|| WorkspaceError::Validation("Card has no zoom layer".to_string()))?;
        let target_index = if deeper {
            current.checked_add(1)
        } else {
            current.checked_sub(1)
        }
        .filter(|index| *index < lens.semantic_zoom_layers.len())
        .ok_or_else(|| WorkspaceError::Validation("No adjacent zoom layer".to_string()))?;
        let target = &lens.semantic_zoom_layers[target_index];
        let range = effective_percent_settings(target).active_zoom_range;
        let target_percent = (range.start_percent + range.end_percent) * 0.5;
        let target_id = target.id.clone();
        let bounds = lens
            .placements
            .iter()
            .find(|placement| placement.concept_id == concept_id)
            .ok_or_else(|| WorkspaceError::MissingPlacement {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                concept_id,
            })?
            .bounds;
        let size_ratio = (bounds.size.x / NEW_CARD_BASELINE_SIZE_AT_100_PERCENT.x)
            .max(bounds.size.y / NEW_CARD_BASELINE_SIZE_AT_100_PERCENT.y);
        let size = bounds.size * (100.0 / target_percent / size_ratio);
        let mut candidate = self.clone();
        candidate.resize_concept_in_lens(
            version_id,
            lens_id,
            concept_id,
            Rect::new(bounds.origin + (bounds.size - size) * 0.5, size),
        )?;
        candidate.set_semantic_zoom_card_layer_mode(
            version_id,
            lens_id,
            concept_id,
            SemanticZoomCardLayerMode::Manual {
                layer_ids: vec![target_id],
            },
        )?;
        *self = candidate;
        Ok(())
    }

    /// Moves every requested card placement by one delta in one Lens.
    ///
    /// Fully enclosed cards and boundary frames travel with a selected card.
    /// The request is atomic. It does not change this document unless every
    /// selected card has a placement in the requested Lens and the moved
    /// document is valid.
    pub fn move_concepts_in_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        concept_ids: &[u32],
        world_delta: Point,
    ) -> Result<(), WorkspaceError> {
        if concept_ids.is_empty() {
            return validation(
                "at least one concept ID is required to move placements".to_string(),
            );
        }
        if !world_delta.x.is_finite() || !world_delta.y.is_finite() {
            return validation("placement movement delta must be finite".to_string());
        }

        let requested_concept_ids: BTreeSet<_> = concept_ids.iter().copied().collect();
        if requested_concept_ids.len() != concept_ids.len() {
            return validation("concept IDs to move must be unique".to_string());
        }

        self.move_canvas_items_in_lens(
            version_id,
            lens_id,
            &requested_concept_ids
                .into_iter()
                .map(CanvasItemId::Card)
                .collect::<Vec<_>>(),
            world_delta,
        )
        .map(|_| ())
    }

    /// Moves cards and Lens-local boundary frames through one containment
    /// resolver. Both item kinds use the same fully-enclosed descendant rule.
    pub fn move_canvas_items_in_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        item_ids: &[CanvasItemId],
        world_delta: Point,
    ) -> Result<Vec<CanvasItemId>, WorkspaceError> {
        if item_ids.is_empty() {
            return validation("at least one canvas item is required to move".to_string());
        }
        if !world_delta.x.is_finite() || !world_delta.y.is_finite() {
            return validation("canvas item movement delta must be finite".to_string());
        }

        let requested_item_ids = item_ids.iter().cloned().collect::<BTreeSet<_>>();
        if requested_item_ids.len() != item_ids.len() {
            return validation("canvas item IDs to move must be unique".to_string());
        }

        let mut candidate = self.clone();
        let moved_item_ids = {
            let version = candidate.version_mut(version_id)?;
            let lens = version
                .lenses
                .iter_mut()
                .find(|lens| lens.id == lens_id)
                .ok_or_else(|| WorkspaceError::MissingLens {
                    version_id: version_id.to_string(),
                    lens_id: lens_id.to_string(),
                })?;

            for item_id in &requested_item_ids {
                match item_id {
                    CanvasItemId::Card(concept_id)
                        if !lens
                            .placements
                            .iter()
                            .any(|placement| placement.concept_id == *concept_id) =>
                    {
                        return Err(WorkspaceError::MissingPlacement {
                            version_id: version_id.to_string(),
                            lens_id: lens_id.to_string(),
                            concept_id: *concept_id,
                        });
                    }
                    CanvasItemId::Boundary(boundary_id)
                        if !lens
                            .visual_boundaries
                            .iter()
                            .any(|boundary| boundary.id == *boundary_id) =>
                    {
                        return Err(WorkspaceError::MissingVisualBoundary {
                            version_id: version_id.to_string(),
                            lens_id: lens_id.to_string(),
                            boundary_id: boundary_id.clone(),
                        });
                    }
                    _ => {}
                }
            }

            let layouts = lens
                .placements
                .iter()
                .map(|placement| CanvasItemBounds {
                    id: CanvasItemId::Card(placement.concept_id),
                    bounds: placement.bounds,
                })
                .chain(
                    lens.visual_boundaries
                        .iter()
                        .map(|boundary| CanvasItemBounds {
                            id: CanvasItemId::Boundary(boundary.id.clone()),
                            bounds: boundary.bounds,
                        }),
                )
                .collect::<Vec<_>>();
            let moved_item_ids =
                canvas_item_move_ids_with_contained_descendants(layouts, requested_item_ids)
                    .into_iter()
                    .collect::<BTreeSet<_>>();

            for placement in &mut lens.placements {
                if moved_item_ids.contains(&CanvasItemId::Card(placement.concept_id)) {
                    placement.bounds.origin += world_delta;
                }
            }
            for boundary in &mut lens.visual_boundaries {
                if moved_item_ids.contains(&CanvasItemId::Boundary(boundary.id.clone())) {
                    boundary.bounds.origin += world_delta;
                }
            }
            moved_item_ids.into_iter().collect::<Vec<_>>()
        };
        candidate.validate()?;
        *self = candidate;
        Ok(moved_item_ids)
    }

    pub fn link_concepts(
        &mut self,
        version_id: &str,
        source_id: u32,
        target_id: u32,
        kind: RelationshipKind,
    ) -> Result<TypedRelationship, WorkspaceError> {
        let relationship = {
            let version = self.version_mut(version_id)?;
            validate_link_endpoints(version, source_id, target_id, kind)?;
            let base_id = format!("rel-{source_id}-{target_id}-{}", kind.identifier());
            let relationship_ids: BTreeSet<_> = version
                .relationships
                .iter()
                .map(|relationship| relationship.id.as_str())
                .collect();
            let mut relationship_id = base_id.clone();
            let mut duplicate_index = 2;
            while relationship_ids.contains(relationship_id.as_str()) {
                relationship_id = format!("{base_id}-{duplicate_index}");
                duplicate_index += 1;
            }
            let relationship = TypedRelationship {
                id: relationship_id,
                kind,
                source_id,
                target_id,
                label: kind.label().to_string(),
            };
            version.relationships.push(relationship.clone());
            for lens in &mut version.lenses {
                let source_base_layer = lens
                    .semantic_zoom_layers
                    .iter()
                    .position(|layer| layer.concept_ids.contains(&source_id));
                let target_base_layer = lens
                    .semantic_zoom_layers
                    .iter()
                    .position(|layer| layer.concept_ids.contains(&target_id));
                if let Some(base_layer_index) = source_base_layer
                    .zip(target_base_layer)
                    .map(|(source_index, target_index)| source_index.max(target_index))
                {
                    lens.semantic_zoom_layers[base_layer_index]
                        .relationship_ids
                        .push(relationship.id.clone());
                }
            }
            relationship
        };
        self.validate()?;
        Ok(relationship)
    }

    pub fn remove_relationship(
        &mut self,
        version_id: &str,
        relationship_id: &str,
    ) -> Result<TypedRelationship, WorkspaceError> {
        self.validate()?;
        let relationship = {
            let version = self.version_mut(version_id)?;
            let relationship_index = version
                .relationships
                .iter()
                .position(|relationship| relationship.id == relationship_id)
                .ok_or_else(|| WorkspaceError::MissingRelationship {
                    version_id: version_id.to_string(),
                    relationship_id: relationship_id.to_string(),
                })?;
            let relationship = version.relationships.remove(relationship_index);
            for lens in &mut version.lenses {
                for layer in &mut lens.semantic_zoom_layers {
                    layer
                        .relationship_ids
                        .retain(|member_id| member_id != relationship_id);
                }
            }
            relationship
        };
        self.validate()?;
        Ok(relationship)
    }

    pub fn change_relationship_kind(
        &mut self,
        version_id: &str,
        relationship_id: &str,
        relationship_kind: RelationshipKind,
    ) -> Result<TypedRelationship, WorkspaceError> {
        self.validate()?;

        let mut candidate = self.clone();
        let changed_relationship = {
            let version = candidate.version_mut(version_id)?;
            let relationship = version
                .relationships
                .iter_mut()
                .find(|relationship| relationship.id == relationship_id)
                .ok_or_else(|| WorkspaceError::MissingRelationship {
                    version_id: version_id.to_string(),
                    relationship_id: relationship_id.to_string(),
                })?;
            let label_tracks_kind = relationship.label.trim().is_empty()
                || relationship.label == relationship.kind.label();
            relationship.kind = relationship_kind;
            if label_tracks_kind {
                relationship.label = relationship_kind.label().to_string();
            }
            relationship.clone()
        };

        candidate.validate()?;
        *self = candidate;
        Ok(changed_relationship)
    }

    pub fn edit_concept(
        &mut self,
        version_id: &str,
        concept_id: u32,
        type_id: &str,
        title: String,
    ) -> Result<WorkspaceConcept, WorkspaceError> {
        self.validate()?;

        let mut candidate = self.clone();
        let edited_concept = {
            let version = candidate.version_mut(version_id)?;
            let concept = version
                .concepts
                .iter_mut()
                .find(|concept| concept.id == concept_id)
                .ok_or_else(|| WorkspaceError::MissingConcept {
                    version_id: version_id.to_string(),
                    concept_id,
                })?;
            concept.type_id = type_id.to_string();
            concept.title = title;
            concept.clone()
        };

        candidate.synchronize_concept_content(version_id, concept_id)?;
        *self = candidate;
        Ok(edited_concept)
    }

    /// Propagates all content fields of an explicitly linked card atomically.
    pub fn synchronize_concept_content(
        &mut self,
        version_id: &str,
        concept_id: u32,
    ) -> Result<(), WorkspaceError> {
        let source = self.concept(version_id, concept_id)?.clone();
        let mut candidate = self.clone();
        if let Some(token) = &source.content_reference {
            for version in std::iter::once(&mut candidate.current_state).chain(
                candidate
                    .planning_scenarios
                    .iter_mut()
                    .map(|scenario| &mut scenario.version),
            ) {
                for card in &mut version.concepts {
                    if card.content_reference.as_ref() == Some(token) {
                        let id = card.id;
                        *card = source.clone();
                        card.id = id;
                    }
                }
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Removes one view reference while retaining the shared concept and its
    /// relationships for other views in the version.
    pub fn remove_concept_from_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        concept_id: u32,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        let mut candidate = self.clone();
        let relationships = candidate
            .relationships_in_lens(version_id, lens_id)?
            .to_vec();
        let incident_relationship_ids = relationships
            .iter()
            .filter(|relationship| {
                relationship.source_id == concept_id || relationship.target_id == concept_id
            })
            .map(|relationship| relationship.id.clone())
            .collect::<BTreeSet<_>>();
        let lens = candidate.lens_mut(version_id, lens_id)?;
        if !lens
            .placements
            .iter()
            .any(|placement| placement.concept_id == concept_id)
        {
            return Err(WorkspaceError::MissingPlacement {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
                concept_id,
            });
        }
        lens.relationships = Some(
            relationships
                .into_iter()
                .filter(|relationship| !incident_relationship_ids.contains(&relationship.id))
                .collect(),
        );
        lens.placements
            .retain(|placement| placement.concept_id != concept_id);
        lens.semantic_zoom_card_modes
            .retain(|entry| entry.concept_id != concept_id);
        for layer in &mut lens.semantic_zoom_layers {
            layer.concept_ids.retain(|id| *id != concept_id);
            layer
                .relationship_ids
                .retain(|id| !incident_relationship_ids.contains(id));
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    pub fn delete_concept(
        &mut self,
        version_id: &str,
        concept_id: u32,
    ) -> Result<DeletedConcept, WorkspaceError> {
        self.validate()?;
        let original = self.clone();
        let deletion = (|| {
            let version = self.version_mut(version_id)?;
            let concept_index = version
                .concepts
                .iter()
                .position(|concept| concept.id == concept_id)
                .ok_or_else(|| WorkspaceError::MissingConcept {
                    version_id: version_id.to_string(),
                    concept_id,
                })?;
            let concept = version.concepts.remove(concept_index);
            let relationships: Vec<_> = version
                .relationships
                .iter()
                .enumerate()
                .filter(|(_, relationship)| {
                    relationship.source_id == concept_id || relationship.target_id == concept_id
                })
                .map(|(index, relationship)| DeletedTypedRelationship {
                    index,
                    relationship: relationship.clone(),
                })
                .collect();
            version.relationships.retain(|relationship| {
                relationship.source_id != concept_id && relationship.target_id != concept_id
            });

            let mut placements = Vec::new();
            let mut card_layer_modes = Vec::new();
            let mut lens_relationships = Vec::new();
            for lens in &mut version.lenses {
                if let Some(local) = &mut lens.relationships {
                    let removed = local
                        .iter()
                        .enumerate()
                        .filter(|(_, relationship)| {
                            relationship.source_id == concept_id
                                || relationship.target_id == concept_id
                        })
                        .map(|(index, relationship)| {
                            (
                                lens.id.clone(),
                                DeletedTypedRelationship {
                                    index,
                                    relationship: relationship.clone(),
                                },
                            )
                        })
                        .collect::<Vec<_>>();
                    let ids: BTreeSet<_> = removed
                        .iter()
                        .map(|(_, deleted)| deleted.relationship.id.clone())
                        .collect();
                    local.retain(|relationship| !ids.contains(&relationship.id));
                    for layer in &mut lens.semantic_zoom_layers {
                        layer.relationship_ids.retain(|id| !ids.contains(id));
                    }
                    lens_relationships.extend(removed);
                }
                placements.extend(
                    lens.placements
                        .iter()
                        .enumerate()
                        .filter(|(_, placement)| placement.concept_id == concept_id)
                        .map(|(index, placement)| DeletedConceptPlacement {
                            lens_id: lens.id.clone(),
                            index,
                            placement: placement.clone(),
                        }),
                );
                lens.placements
                    .retain(|placement| placement.concept_id != concept_id);
                card_layer_modes.extend(
                    lens.semantic_zoom_card_modes
                        .iter()
                        .enumerate()
                        .filter(|(_, entry)| entry.concept_id == concept_id)
                        .map(|(index, entry)| DeletedSemanticZoomCardLayerMode {
                            lens_id: lens.id.clone(),
                            index,
                            entry: entry.clone(),
                        }),
                );
                lens.semantic_zoom_card_modes
                    .retain(|entry| entry.concept_id != concept_id);
                for layer in &mut lens.semantic_zoom_layers {
                    layer
                        .concept_ids
                        .retain(|member_id| *member_id != concept_id);
                }
            }
            let removed_relationship_ids: BTreeSet<_> = relationships
                .iter()
                .map(|deleted| deleted.relationship.id.as_str())
                .collect();
            for lens in &mut version.lenses {
                for layer in &mut lens.semantic_zoom_layers {
                    layer.relationship_ids.retain(|relationship_id| {
                        !removed_relationship_ids.contains(relationship_id.as_str())
                    });
                }
            }

            Ok(DeletedConcept {
                version_id: version_id.to_string(),
                concept_index,
                concept,
                relationships,
                lens_relationships,
                placements,
                card_layer_modes,
            })
        })();

        match deletion.and_then(|deletion| self.validate().map(|()| deletion)) {
            Ok(deletion) => Ok(deletion),
            Err(error) => {
                *self = original;
                Err(error)
            }
        }
    }

    pub fn restore_deleted_concept(
        &mut self,
        deletion: &DeletedConcept,
    ) -> Result<(), WorkspaceError> {
        self.validate()?;
        let original = self.clone();
        let result = (|| {
            validate_deleted_records(deletion)?;
            {
                let version = self.version_mut(&deletion.version_id)?;
                if version
                    .concepts
                    .iter()
                    .any(|concept| concept.id == deletion.concept.id)
                {
                    return Err(WorkspaceError::RestoreConceptAlreadyExists {
                        version_id: deletion.version_id.clone(),
                        concept_id: deletion.concept.id,
                    });
                }
                if deletion.concept_index > version.concepts.len() {
                    return Err(invalid_deletion_receipt(
                        deletion,
                        "concept insertion index is out of bounds",
                    ));
                }

                version
                    .concepts
                    .insert(deletion.concept_index, deletion.concept.clone());

                let mut relationships = deletion.relationships.clone();
                relationships.sort_by_key(|deleted| deleted.index);
                for deleted in relationships {
                    if deleted.index > version.relationships.len() {
                        return Err(invalid_deletion_receipt(
                            deletion,
                            "relationship insertion index is out of bounds",
                        ));
                    }
                    version
                        .relationships
                        .insert(deleted.index, deleted.relationship);
                }

                let mut local_relationships = deletion.lens_relationships.clone();
                local_relationships.sort_by(|left, right| {
                    left.0.cmp(&right.0).then(left.1.index.cmp(&right.1.index))
                });
                for (lens_id, deleted) in local_relationships {
                    let lens = version
                        .lenses
                        .iter_mut()
                        .find(|lens| lens.id == lens_id)
                        .ok_or_else(|| WorkspaceError::MissingLens {
                            version_id: deletion.version_id.clone(),
                            lens_id,
                        })?;
                    let relationships = lens.relationships.as_mut().ok_or_else(|| {
                        invalid_deletion_receipt(deletion, "view-local relationship set is missing")
                    })?;
                    if deleted.index > relationships.len()
                        || (deleted.relationship.source_id != deletion.concept.id
                            && deleted.relationship.target_id != deletion.concept.id)
                    {
                        return Err(invalid_deletion_receipt(
                            deletion,
                            "invalid view-local relationship receipt",
                        ));
                    }
                    relationships.insert(deleted.index, deleted.relationship);
                }
                let mut placements = deletion.placements.clone();
                placements.sort_by(|first, second| {
                    first
                        .lens_id
                        .cmp(&second.lens_id)
                        .then(first.index.cmp(&second.index))
                });
                for deleted in placements {
                    let lens = version
                        .lenses
                        .iter_mut()
                        .find(|lens| lens.id == deleted.lens_id)
                        .ok_or_else(|| WorkspaceError::MissingLens {
                            version_id: deletion.version_id.clone(),
                            lens_id: deleted.lens_id.clone(),
                        })?;
                    if deleted.index > lens.placements.len() {
                        return Err(invalid_deletion_receipt(
                            deletion,
                            "placement insertion index is out of bounds",
                        ));
                    }
                    lens.placements.insert(deleted.index, deleted.placement);
                }

                let mut card_layer_modes = deletion.card_layer_modes.clone();
                card_layer_modes.sort_by(|first, second| {
                    first
                        .lens_id
                        .cmp(&second.lens_id)
                        .then(first.index.cmp(&second.index))
                });
                for deleted in card_layer_modes {
                    let lens = version
                        .lenses
                        .iter_mut()
                        .find(|lens| lens.id == deleted.lens_id)
                        .ok_or_else(|| WorkspaceError::MissingLens {
                            version_id: deletion.version_id.clone(),
                            lens_id: deleted.lens_id.clone(),
                        })?;
                    if deleted.index > lens.semantic_zoom_card_modes.len() {
                        return Err(invalid_deletion_receipt(
                            deletion,
                            "card layer mode insertion index is out of bounds",
                        ));
                    }
                    lens.semantic_zoom_card_modes
                        .insert(deleted.index, deleted.entry);
                }
            }

            self.validate()
        })();

        if result.is_err() {
            *self = original;
        }
        result
    }

    pub fn add_concept_to_lens(
        &mut self,
        version_id: &str,
        lens_id: &str,
        type_id: &str,
        title: String,
        summary: String,
        bounds: Rect,
    ) -> Result<u32, WorkspaceError> {
        let concept_id = {
            let version = self.version_mut(version_id)?;
            let concept_id = version
                .concepts
                .iter()
                .map(|concept| concept.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            version.concepts.push(WorkspaceConcept {
                content_reference: None,
                id: concept_id,
                type_id: type_id.to_string(),
                title,
                summary,
                details: String::new(),
                execution_mode: None,
                attributes: BTreeMap::new(),
            });
            let lens = version
                .lenses
                .iter_mut()
                .find(|lens| lens.id == lens_id)
                .ok_or_else(|| WorkspaceError::MissingLens {
                    version_id: version_id.to_string(),
                    lens_id: lens_id.to_string(),
                })?;
            lens.placements
                .push(ConceptPlacement { concept_id, bounds });
            concept_id
        };
        self.validate()?;
        Ok(concept_id)
    }

    pub fn create_planning_scenario(
        &mut self,
        scenario_id: String,
        title: String,
        based_on_version_id: &str,
    ) -> Result<(), WorkspaceError> {
        let mut version = self.version(based_on_version_id)?.clone();
        version.id = scenario_id;
        version.title = title;
        self.planning_scenarios.push(PlanningScenario {
            based_on_version_id: based_on_version_id.to_string(),
            version,
        });
        self.validate()
    }

    pub fn sample() -> Self {
        let concepts = vec![
            concept(1, "organization", "Acme", "The organization"),
            concept(2, "team", "Platform Team", "Owns platform capabilities"),
            concept(3, "team", "Product Team", "Owns product delivery"),
            concept(4, "project", "Customer Portal", "A customer-facing project"),
            concept(5, "service", "Portal API", "The portal backend service"),
            concept(6, "environment", "Production", "The production environment"),
            concept(
                7,
                "process",
                "Release Pipeline",
                "Automated software release process",
            ),
            concept(
                8,
                "repository",
                "portal-api",
                "Repository for the Portal API",
            ),
            concept(
                9,
                "access",
                "Grafana read access",
                "Read access for release checks",
            ),
        ];
        let relationships = vec![
            relationship("rel-product-portal", RelationshipKind::Owns, 3, 4),
            relationship("rel-platform-api", RelationshipKind::Owns, 2, 5),
            relationship("rel-platform-production", RelationshipKind::Operates, 2, 6),
            relationship("rel-portal-api", RelationshipKind::DependsOn, 4, 5),
            relationship("rel-api-production", RelationshipKind::DeployedTo, 5, 6),
            relationship("rel-release-api", RelationshipKind::DependsOn, 7, 5),
            relationship(
                "rel-release-repository",
                RelationshipKind::ImplementedIn,
                7,
                8,
            ),
            relationship("rel-release-access", RelationshipKind::DependsOn, 7, 9),
        ];
        let all_placements = vec![
            placement(1, 120.0, 120.0),
            placement(2, 440.0, 80.0),
            placement(3, 440.0, 280.0),
            placement(4, 760.0, 280.0),
            placement(5, 1080.0, 280.0),
            placement(6, 1400.0, 280.0),
            placement(7, 760.0, 520.0),
            placement(8, 1080.0, 520.0),
            placement(9, 1080.0, 680.0),
        ];
        let delivery_placements = vec![
            placement(3, 160.0, 280.0),
            placement(4, 500.0, 280.0),
            placement(5, 840.0, 280.0),
            placement(6, 1180.0, 280.0),
            placement(2, 1180.0, 80.0),
            placement(7, 500.0, 520.0),
            placement(8, 840.0, 520.0),
            placement(9, 840.0, 680.0),
        ];
        Self {
            schema_version: WORKSPACE_SCHEMA_VERSION,
            workspace_id: "acme-platform".to_string(),
            title: "Acme Platform Landscape".to_string(),
            custom_types: vec![CustomType {
                id: "capability".to_string(),
                title: "Capability".to_string(),
                color: "#8B5CF6".to_string(),
                icon: None,
            }],
            current_state: WorkspaceVersion {
                id: "current".to_string(),
                title: "Current State".to_string(),
                concepts,
                relationships,
                lenses: vec![
                    WorkspaceLens {
                        relationships: None,
                        id: "organization-landscape".to_string(),
                        title: "Organization Landscape".to_string(),
                        template: Some(LensTemplate::Ownership),
                        filter: LensFilter::default(),
                        placements: all_placements,
                        visual_boundaries: Vec::new(),
                        semantic_zoom_layers: Vec::new(),
                        semantic_zoom_card_modes: Vec::new(),
                    },
                    WorkspaceLens {
                        relationships: None,
                        id: "delivery-and-production".to_string(),
                        title: "Delivery and Production".to_string(),
                        template: Some(LensTemplate::Deployment),
                        filter: LensFilter {
                            concept_type_ids: vec![
                                "team".to_string(),
                                "project".to_string(),
                                "process".to_string(),
                                "repository".to_string(),
                                "service".to_string(),
                                "environment".to_string(),
                                "access".to_string(),
                            ],
                            relationship_kinds: Vec::new(),
                        },
                        placements: delivery_placements,
                        visual_boundaries: Vec::new(),
                        semantic_zoom_layers: Vec::new(),
                        semantic_zoom_card_modes: Vec::new(),
                    },
                ],
            },
            planning_scenarios: Vec::new(),
        }
    }

    fn version_mut(&mut self, version_id: &str) -> Result<&mut WorkspaceVersion, WorkspaceError> {
        if self.current_state.id == version_id {
            return Ok(&mut self.current_state);
        }
        self.planning_scenarios
            .iter_mut()
            .find(|scenario| scenario.version.id == version_id)
            .map(|scenario| &mut scenario.version)
            .ok_or_else(|| WorkspaceError::MissingVersion(version_id.to_string()))
    }

    fn lens_mut(
        &mut self,
        version_id: &str,
        lens_id: &str,
    ) -> Result<&mut WorkspaceLens, WorkspaceError> {
        let version = self.version_mut(version_id)?;
        version
            .lenses
            .iter_mut()
            .find(|lens| lens.id == lens_id)
            .ok_or_else(|| WorkspaceError::MissingLens {
                version_id: version_id.to_string(),
                lens_id: lens_id.to_string(),
            })
    }

    fn custom_type_color(&self, type_id: &str) -> Option<[f32; 4]> {
        self.custom_types
            .iter()
            .find(|custom_type| custom_type.id == type_id)
            .and_then(|custom_type| parse_hex_color(&custom_type.color))
    }
}

impl LensFilter {
    fn includes_concept_type(&self, type_id: &str) -> bool {
        self.concept_type_ids.is_empty()
            || self
                .concept_type_ids
                .iter()
                .any(|included_type_id| included_type_id == type_id)
    }

    fn includes_relationship_kind(&self, kind: RelationshipKind) -> bool {
        self.relationship_kinds.is_empty() || self.relationship_kinds.contains(&kind)
    }
}

fn overlay_source_state(
    concept: &WorkspaceConcept,
    placement: Option<Rect>,
    filter: &LensFilter,
    viewport: Rect,
) -> SelectedConceptOverlaySource {
    if !filter.includes_concept_type(&concept.type_id) {
        return SelectedConceptOverlaySource::Filtered;
    }
    let Some(bounds) = placement else {
        return SelectedConceptOverlaySource::Unplaced;
    };
    if bounds.intersects(viewport) {
        SelectedConceptOverlaySource::Visible { bounds }
    } else {
        SelectedConceptOverlaySource::Offscreen
    }
}

fn overlay_target_state(
    concept: Option<&WorkspaceConcept>,
    placement: Option<Rect>,
    filter: &LensFilter,
    viewport: Rect,
) -> RelationshipOverlayTarget {
    let Some(concept) = concept else {
        return RelationshipOverlayTarget::Absent;
    };
    if !filter.includes_concept_type(&concept.type_id) {
        return RelationshipOverlayTarget::Filtered;
    }
    let Some(bounds) = placement else {
        return RelationshipOverlayTarget::Unplaced;
    };
    if bounds.intersects(viewport) {
        RelationshipOverlayTarget::Visible { bounds }
    } else {
        RelationshipOverlayTarget::Offscreen
    }
}

const RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE: f32 = 16.0;

fn relationship_overlay_path(
    source_bounds: Rect,
    target_bounds: Rect,
    visible_unrelated_bounds: &[Rect],
) -> Vec<Point> {
    relationship_path_around_obstacles(source_bounds, target_bounds, visible_unrelated_bounds)
}

/// Routes a relationship between two card edges without crossing the interior
/// of an unrelated card. The returned route is deterministic and uses the
/// shortest visible route around the supplied obstacles when a direct segment
/// is blocked.
pub fn relationship_path_around_obstacles(
    source_bounds: Rect,
    target_bounds: Rect,
    visible_unrelated_bounds: &[Rect],
) -> Vec<Point> {
    let source_bounds = source_bounds.normalized();
    let target_bounds = target_bounds.normalized();
    let source_center = source_bounds.center();
    let target_center = target_bounds.center();
    let obstacles: Vec<_> = visible_unrelated_bounds
        .iter()
        .copied()
        .map(Rect::normalized)
        .filter(|bounds| !bounds.is_empty())
        .collect();
    let source = card_edge_toward(source_bounds, target_center);
    let target = card_edge_toward(target_bounds, source_center);

    if !segment_crosses_any_rect_interior(source, target, &obstacles) {
        return vec![source, target];
    }

    let expanded_obstacles: Vec<_> = obstacles
        .iter()
        .copied()
        .map(expand_relationship_route_obstacle)
        .collect();
    let mut route = shortest_visible_route(source_center, target_center, &expanded_obstacles)
        .unwrap_or_else(|| {
            fallback_relationship_route(source_center, target_center, &expanded_obstacles)
        });

    if route.len() < 2 {
        return vec![source, target];
    }

    route[0] = card_edge_toward(source_bounds, route[1]);
    let route_end = route.len() - 1;
    route[route_end] = card_edge_toward(target_bounds, route[route_end - 1]);
    route
}

fn card_edge_toward(bounds: Rect, toward: Point) -> Point {
    let bounds = bounds.normalized();
    let center = bounds.center();
    let direction = toward - center;
    let direction = if direction.x == 0.0 && direction.y == 0.0 {
        Point::new(1.0, 0.0)
    } else {
        direction
    };
    let horizontal_distance = bounds.size.x * 0.5;
    let vertical_distance = bounds.size.y * 0.5;
    let horizontal_scale = if direction.x == 0.0 {
        f32::INFINITY
    } else {
        horizontal_distance / direction.x.abs()
    };
    let vertical_scale = if direction.y == 0.0 {
        f32::INFINITY
    } else {
        vertical_distance / direction.y.abs()
    };

    center + direction * horizontal_scale.min(vertical_scale)
}

fn expand_relationship_route_obstacle(bounds: Rect) -> Rect {
    let bounds = bounds.normalized();
    Rect::new(
        Point::new(
            bounds.origin.x - RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE,
            bounds.origin.y - RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE,
        ),
        Point::new(
            bounds.size.x + RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE * 2.0,
            bounds.size.y + RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE * 2.0,
        ),
    )
}

fn shortest_visible_route(source: Point, target: Point, obstacles: &[Rect]) -> Option<Vec<Point>> {
    let mut nodes = vec![source, target];
    for obstacle in obstacles {
        let obstacle = obstacle.normalized();
        nodes.extend([
            obstacle.origin,
            Point::new(obstacle.max_x(), obstacle.origin.y),
            Point::new(obstacle.origin.x, obstacle.max_y()),
            Point::new(obstacle.max_x(), obstacle.max_y()),
        ]);
    }

    let node_count = nodes.len();
    let mut distances = vec![f32::INFINITY; node_count];
    let mut previous = vec![None; node_count];
    let mut visited = vec![false; node_count];
    distances[0] = 0.0;

    for _ in 0..node_count {
        let current = (0..node_count)
            .filter(|index| !visited[*index])
            .min_by(|left, right| {
                distances[*left]
                    .total_cmp(&distances[*right])
                    .then_with(|| left.cmp(right))
            })?;
        if !distances[current].is_finite() || current == 1 {
            break;
        }
        visited[current] = true;

        for next in 0..node_count {
            if visited[next]
                || segment_crosses_any_rect_interior(nodes[current], nodes[next], obstacles)
            {
                continue;
            }

            let candidate_distance =
                distances[current] + point_distance(nodes[current], nodes[next]);
            if candidate_distance < distances[next] {
                distances[next] = candidate_distance;
                previous[next] = Some(current);
            }
        }
    }

    distances[1].is_finite().then(|| {
        let mut indices = vec![1];
        while let Some(index) = previous[*indices.last().expect("route has a target")] {
            indices.push(index);
        }
        indices.reverse();
        indices.into_iter().map(|index| nodes[index]).collect()
    })
}

fn fallback_relationship_route(source: Point, target: Point, obstacles: &[Rect]) -> Vec<Point> {
    let first = obstacles
        .first()
        .copied()
        .expect("a fallback route needs an obstacle");
    let outer_bounds = obstacles.iter().copied().fold(first, |bounds, obstacle| {
        Rect::from_points(
            bounds.origin,
            Point::new(
                bounds.max_x().max(obstacle.max_x()),
                bounds.max_y().max(obstacle.max_y()),
            ),
        )
        .normalized()
    });
    let top = outer_bounds.origin.y - RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE;
    let bottom = outer_bounds.max_y() + RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE;
    let left = outer_bounds.origin.x - RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE;
    let right = outer_bounds.max_x() + RELATIONSHIP_OVERLAY_ROUTE_CLEARANCE;
    let candidates = [
        vec![
            source,
            Point::new(source.x, top),
            Point::new(target.x, top),
            target,
        ],
        vec![
            source,
            Point::new(source.x, bottom),
            Point::new(target.x, bottom),
            target,
        ],
        vec![
            source,
            Point::new(left, source.y),
            Point::new(left, target.y),
            target,
        ],
        vec![
            source,
            Point::new(right, source.y),
            Point::new(right, target.y),
            target,
        ],
    ];

    candidates
        .into_iter()
        .filter(|candidate| {
            candidate.windows(2).all(|segment| {
                !segment_crosses_any_rect_interior(segment[0], segment[1], obstacles)
            })
        })
        .min_by(|left, right| {
            route_length(left)
                .total_cmp(&route_length(right))
                .then_with(|| compare_routes(left, right))
        })
        .unwrap_or_else(|| vec![source, target])
}

fn segment_crosses_any_rect_interior(start: Point, end: Point, rectangles: &[Rect]) -> bool {
    rectangles
        .iter()
        .copied()
        .any(|rectangle| segment_crosses_rect_interior(start, end, rectangle))
}

fn segment_crosses_rect_interior(start: Point, end: Point, rectangle: Rect) -> bool {
    let rectangle = rectangle.normalized();
    if rectangle.is_empty() {
        return false;
    }

    let delta = end - start;
    let Some((x_start, x_end)) =
        open_axis_interval(start.x, delta.x, rectangle.origin.x, rectangle.max_x())
    else {
        return false;
    };
    let Some((y_start, y_end)) =
        open_axis_interval(start.y, delta.y, rectangle.origin.y, rectangle.max_y())
    else {
        return false;
    };

    x_start.max(y_start).max(0.0) < x_end.min(y_end).min(1.0)
}

fn open_axis_interval(start: f32, delta: f32, minimum: f32, maximum: f32) -> Option<(f32, f32)> {
    if delta == 0.0 {
        return (minimum < start && start < maximum).then_some((f32::NEG_INFINITY, f32::INFINITY));
    }

    let first = (minimum - start) / delta;
    let second = (maximum - start) / delta;
    Some((first.min(second), first.max(second)))
}

fn point_distance(first: Point, second: Point) -> f32 {
    let delta = second - first;
    (delta.x * delta.x + delta.y * delta.y).sqrt()
}

fn route_length(points: &[Point]) -> f32 {
    points
        .windows(2)
        .map(|segment| point_distance(segment[0], segment[1]))
        .sum()
}

fn compare_routes(left: &[Point], right: &[Point]) -> std::cmp::Ordering {
    left.iter()
        .zip(right)
        .find_map(|(left, right)| {
            let ordering = left
                .x
                .total_cmp(&right.x)
                .then_with(|| left.y.total_cmp(&right.y));
            (ordering != std::cmp::Ordering::Equal).then_some(ordering)
        })
        .unwrap_or_else(|| left.len().cmp(&right.len()))
}

fn validate_version(
    version: &WorkspaceVersion,
    custom_type_ids: &BTreeSet<&str>,
    context: &str,
) -> Result<(), WorkspaceError> {
    validate_required(&format!("{context} id"), &version.id)?;
    validate_required(&format!("{context} title"), &version.title)?;

    let mut concept_ids = BTreeSet::new();
    for concept in &version.concepts {
        validate_required(&format!("concept {} type_id", concept.id), &concept.type_id)?;
        validate_required(&format!("concept {} title", concept.id), &concept.title)?;
        if ConceptKind::from_type_id(&concept.type_id).is_none()
            && !custom_type_ids.contains(concept.type_id.as_str())
        {
            return validation(format!(
                "concept {} in version {} uses unknown type {}",
                concept.id, version.id, concept.type_id
            ));
        }
        if concept.execution_mode.is_some() && concept.type_id != "process" {
            return validation(format!(
                "concept {} in version {} has execution_mode but is not a process",
                concept.id, version.id
            ));
        }
        if !concept_ids.insert(concept.id) {
            return validation(format!(
                "duplicate concept id {} in version {}",
                concept.id, version.id
            ));
        }
    }

    let mut relationship_ids = BTreeSet::new();
    let mut typed_relationships = BTreeSet::new();
    let concepts_by_id: BTreeMap<_, _> = version
        .concepts
        .iter()
        .map(|concept| (concept.id, concept))
        .collect();
    for relationship in &version.relationships {
        validate_required("relationship id", &relationship.id)?;
        if !relationship_ids.insert(relationship.id.as_str()) {
            return validation(format!(
                "duplicate relationship id {} in version {}",
                relationship.id, version.id
            ));
        }
        for concept_id in [relationship.source_id, relationship.target_id] {
            if !concept_ids.contains(&concept_id) {
                return validation(format!(
                    "relationship {} in version {} references unknown concept {}",
                    relationship.id, version.id, concept_id
                ));
            }
        }
        if relationship.source_id == relationship.target_id {
            return validation(format!(
                "relationship {} in version {} links concept {} to itself",
                relationship.id, version.id, relationship.source_id
            ));
        }
        if !typed_relationships.insert((
            relationship.kind,
            relationship.source_id,
            relationship.target_id,
        )) {
            return validation(format!(
                "duplicate {} relationship from {} to {} in version {}",
                relationship.kind.identifier(),
                relationship.source_id,
                relationship.target_id,
                version.id
            ));
        }
        let source = concepts_by_id[&relationship.source_id];
        let target = concepts_by_id[&relationship.target_id];
        if !relationship
            .kind
            .supports_endpoints(&source.type_id, &target.type_id)
        {
            let error = match relationship.kind {
                RelationshipKind::Owns | RelationshipKind::Operates => format!(
                    "{} relationship {} in version {} must have a team source",
                    relationship.kind.identifier(),
                    relationship.id,
                    version.id
                ),
                RelationshipKind::ImplementedIn => format!(
                    "implemented_in relationship {} in version {} must run from a process to a repository",
                    relationship.id, version.id
                ),
                RelationshipKind::Contains
                | RelationshipKind::DependsOn
                | RelationshipKind::Delivers
                | RelationshipKind::DeployedTo => {
                    unreachable!("unrestricted relationship kinds support all endpoints")
                }
            };
            return validation(error);
        }
    }

    let mut lens_ids = BTreeSet::new();
    for lens in &version.lenses {
        let relationship_ids: BTreeSet<_> = if let Some(relationships) = &lens.relationships {
            let mut local = version.clone();
            local.relationships = relationships.clone();
            let mut inherited = lens.clone();
            inherited.relationships = None;
            local.lenses = vec![inherited];
            validate_version(&local, custom_type_ids, context)?;
            relationships
                .iter()
                .map(|relationship| relationship.id.as_str())
                .collect()
        } else {
            relationship_ids.clone()
        };
        validate_required("lens id", &lens.id)?;
        validate_required("lens title", &lens.title)?;
        if !lens_ids.insert(lens.id.as_str()) {
            return validation(format!(
                "duplicate lens id {} in version {}",
                lens.id, version.id
            ));
        }
        for type_id in &lens.filter.concept_type_ids {
            if ConceptKind::from_type_id(type_id).is_none()
                && !custom_type_ids.contains(type_id.as_str())
            {
                return validation(format!(
                    "lens {} in version {} filters unknown concept type {}",
                    lens.id, version.id, type_id
                ));
            }
        }
        let mut placement_ids = BTreeSet::new();
        for placement in &lens.placements {
            if !concept_ids.contains(&placement.concept_id) {
                return validation(format!(
                    "lens {} in version {} places unknown concept {}",
                    lens.id, version.id, placement.concept_id
                ));
            }
            if !placement_ids.insert(placement.concept_id) {
                return validation(format!(
                    "lens {} in version {} has duplicate placement for concept {}",
                    lens.id, version.id, placement.concept_id
                ));
            }
            if !has_valid_bounds(placement.bounds) {
                return validation(format!(
                    "lens {} in version {} has invalid bounds for concept {}",
                    lens.id, version.id, placement.concept_id
                ));
            }
        }

        let mut visual_boundary_ids = BTreeSet::new();
        for boundary in &lens.visual_boundaries {
            validate_required("visual boundary id", &boundary.id)?;
            validate_required("visual boundary title", &boundary.title)?;
            if let Some(layer_id) = &boundary.layer_id
                && !lens
                    .semantic_zoom_layers
                    .iter()
                    .any(|layer| &layer.id == layer_id)
            {
                return validation(format!(
                    "boundary {} references unknown zoom layer {}",
                    boundary.id, layer_id
                ));
            }
            if !visual_boundary_ids.insert(boundary.id.as_str()) {
                return validation(format!(
                    "lens {} in version {} has duplicate visual boundary id {}",
                    lens.id, version.id, boundary.id
                ));
            }
            if !has_valid_bounds(boundary.bounds) {
                return validation(format!(
                    "lens {} in version {} has invalid bounds for visual boundary {}",
                    lens.id, version.id, boundary.id
                ));
            }
        }

        let mut semantic_zoom_layer_ids = BTreeSet::new();
        let mut semantic_zoom_base_concept_ids = BTreeSet::new();
        let mut semantic_zoom_base_relationship_ids = BTreeSet::new();
        for layer in &lens.semantic_zoom_layers {
            validate_required("semantic zoom layer id", &layer.id)?;
            validate_required("semantic zoom layer title", &layer.title)?;
            if !semantic_zoom_layer_ids.insert(layer.id.as_str()) {
                return validation(format!(
                    "lens {} in version {} has duplicate semantic zoom layer id {}",
                    lens.id, version.id, layer.id
                ));
            }
            validate_percent_settings(&layer.percent_settings, layer, lens, version)?;
            validate_zoom_range(&layer.fade_in, "fade-in", layer, lens, version)?;
            validate_zoom_range(&layer.fade_out, "fade-out", layer, lens, version)?;
            if !layer.card_scale.is_finite() || layer.card_scale <= 0.0 {
                return validation(format!(
                    "semantic zoom layer {} in lens {} in version {} has an invalid card scale",
                    layer.id, lens.id, version.id
                ));
            }
            let mut layer_concept_ids = BTreeSet::new();
            for concept_id in &layer.concept_ids {
                if !concept_ids.contains(concept_id) {
                    return validation(format!(
                        "semantic zoom layer {} in lens {} in version {} references unknown concept {}",
                        layer.id, lens.id, version.id, concept_id
                    ));
                }
                if !layer_concept_ids.insert(*concept_id) {
                    return validation(format!(
                        "semantic zoom layer {} in lens {} in version {} repeats concept {}",
                        layer.id, lens.id, version.id, concept_id
                    ));
                }
                if !semantic_zoom_base_concept_ids.insert(*concept_id) {
                    return validation(format!(
                        "concept {} has more than one semantic zoom base layer in lens {} in version {}",
                        concept_id, lens.id, version.id
                    ));
                }
            }
            let mut layer_relationship_ids = BTreeSet::new();
            for relationship_id in &layer.relationship_ids {
                if !relationship_ids.contains(relationship_id.as_str()) {
                    return validation(format!(
                        "semantic zoom layer {} in lens {} in version {} references unknown relationship {}",
                        layer.id, lens.id, version.id, relationship_id
                    ));
                }
                if !layer_relationship_ids.insert(relationship_id.as_str()) {
                    return validation(format!(
                        "semantic zoom layer {} in lens {} in version {} repeats relationship {}",
                        layer.id, lens.id, version.id, relationship_id
                    ));
                }
                if !semantic_zoom_base_relationship_ids.insert(relationship_id.as_str()) {
                    return validation(format!(
                        "relationship {} has more than one semantic zoom base layer in lens {} in version {}",
                        relationship_id, lens.id, version.id
                    ));
                }
            }
        }

        let mut semantic_zoom_card_mode_ids = BTreeSet::new();
        for entry in &lens.semantic_zoom_card_modes {
            if !concept_ids.contains(&entry.concept_id) {
                return validation(format!(
                    "semantic zoom card mode in lens {} in version {} references unknown concept {}",
                    lens.id, version.id, entry.concept_id
                ));
            }
            if !placement_ids.contains(&entry.concept_id) {
                return validation(format!(
                    "semantic zoom card mode in lens {} in version {} references unplaced concept {}",
                    lens.id, version.id, entry.concept_id
                ));
            }
            if !semantic_zoom_card_mode_ids.insert(entry.concept_id) {
                return validation(format!(
                    "semantic zoom card mode in lens {} in version {} repeats concept {}",
                    lens.id, version.id, entry.concept_id
                ));
            }
            if let SemanticZoomCardLayerMode::Manual { layer_ids } = &entry.mode {
                if layer_ids.is_empty() {
                    return validation(format!(
                        "manual semantic zoom card mode for concept {} in lens {} in version {} has no selected layers",
                        entry.concept_id, lens.id, version.id
                    ));
                }
                let mut selected_layer_ids = BTreeSet::new();
                for layer_id in layer_ids {
                    if !semantic_zoom_layer_ids.contains(layer_id.as_str()) {
                        return validation(format!(
                            "manual semantic zoom card mode for concept {} in lens {} in version {} references unknown layer {}",
                            entry.concept_id, lens.id, version.id, layer_id
                        ));
                    }
                    if !selected_layer_ids.insert(layer_id.as_str()) {
                        return validation(format!(
                            "manual semantic zoom card mode for concept {} in lens {} in version {} repeats layer {}",
                            entry.concept_id, lens.id, version.id, layer_id
                        ));
                    }
                }
            }
            if matches!(
                entry.mode,
                SemanticZoomCardLayerMode::Dedicated { .. }
                    | SemanticZoomCardLayerMode::AlwaysVisible
            ) {
                return validation(format!(
                    "semantic zoom card mode for concept {} in lens {} in version {} must migrate to manual",
                    entry.concept_id, lens.id, version.id
                ));
            }
            if matches!(entry.mode, SemanticZoomCardLayerMode::Automatic) {
                return validation(format!(
                    "semantic zoom card mode for concept {} in lens {} in version {} must omit automatic defaults",
                    entry.concept_id, lens.id, version.id
                ));
            }
        }
    }
    if version.lenses.is_empty() {
        return validation(format!("workspace version {} has no lenses", version.id));
    }

    Ok(())
}

fn validate_zoom_range(
    range: &Option<ZoomRange>,
    description: &str,
    layer: &SemanticZoomLayer,
    lens: &WorkspaceLens,
    version: &WorkspaceVersion,
) -> Result<(), WorkspaceError> {
    let Some(range) = range else {
        return Ok(());
    };
    if !range.start.is_finite()
        || !range.end.is_finite()
        || range.start <= 0.0
        || range.end <= range.start
    {
        return validation(format!(
            "semantic zoom layer {} in lens {} in version {} has an invalid {description} range",
            layer.id, lens.id, version.id
        ));
    }
    Ok(())
}

fn validate_percent_settings(
    settings: &Option<SemanticZoomLayerPercentSettings>,
    layer: &SemanticZoomLayer,
    lens: &WorkspaceLens,
    version: &WorkspaceVersion,
) -> Result<(), WorkspaceError> {
    let Some(settings) = settings else {
        return Ok(());
    };
    let range = settings.active_zoom_range;
    if !range.start_percent.is_finite()
        || !range.end_percent.is_finite()
        || range.start_percent < MINIMUM_ZOOM_PERCENT
        || range.end_percent > MAXIMUM_ZOOM_PERCENT
        || range.end_percent < range.start_percent
        || !settings.fade_percent.is_finite()
        || settings.fade_percent < 0.0
    {
        return validation(format!(
            "semantic zoom layer {} in lens {} in version {} has invalid percent zoom settings",
            layer.id, lens.id, version.id
        ));
    }
    Ok(())
}

fn layer_opacity_at_scale(layer: &SemanticZoomLayer, scale: f32) -> f32 {
    if let Some(percent_settings) = layer.percent_settings {
        return layer_opacity_at_percent(percent_settings, scale * 100.0);
    }
    if !scale.is_finite() || scale <= 0.0 {
        return 0.0;
    }
    let fade_in_opacity = layer
        .fade_in
        .map(|range| ((scale - range.start) / (range.end - range.start)).clamp(0.0, 1.0))
        .unwrap_or(1.0);
    let fade_out_opacity = layer
        .fade_out
        .map(|range| ((range.end - scale) / (range.end - range.start)).clamp(0.0, 1.0))
        .unwrap_or(1.0);
    fade_in_opacity.min(fade_out_opacity)
}

fn layer_opacity_at_percent(settings: SemanticZoomLayerPercentSettings, percent: f32) -> f32 {
    if !percent.is_finite() || percent <= 0.0 {
        return 0.0;
    }
    let range = settings.active_zoom_range;
    if percent >= range.start_percent && percent <= range.end_percent {
        return 1.0;
    }
    if settings.fade_percent <= 0.0 {
        return 0.0;
    }
    if percent < range.start_percent {
        return ((percent - (range.start_percent - settings.fade_percent)) / settings.fade_percent)
            .clamp(0.0, 1.0);
    }
    ((range.end_percent + settings.fade_percent - percent) / settings.fade_percent).clamp(0.0, 1.0)
}

fn context_opacity_for_layer(layers: &[SemanticZoomLayerProjection], layer_index: usize) -> f32 {
    layers[layer_index..]
        .iter()
        .map(|layer| layer.opacity)
        .fold(0.0_f32, f32::max)
}

fn automatic_layer_index(layers: &[SemanticZoomLayer], bounds: Rect) -> Option<usize> {
    let normalized = bounds.normalized();
    let size_ratio = (normalized.size.x / NEW_CARD_BASELINE_SIZE_AT_100_PERCENT.x)
        .max(normalized.size.y / NEW_CARD_BASELINE_SIZE_AT_100_PERCENT.y);
    if !size_ratio.is_finite() || size_ratio <= 0.0 {
        return None;
    }
    let target_percent = (100.0 / size_ratio).clamp(MINIMUM_ZOOM_PERCENT, MAXIMUM_ZOOM_PERCENT);
    layers
        .iter()
        .enumerate()
        .min_by(|(left_index, left), (right_index, right)| {
            percent_distance_to_range(
                target_percent,
                effective_percent_settings(left).active_zoom_range,
            )
            .total_cmp(&percent_distance_to_range(
                target_percent,
                effective_percent_settings(right).active_zoom_range,
            ))
            .then(left_index.cmp(right_index))
        })
        .map(|(index, _)| index)
}

fn card_layer_mode(lens: &WorkspaceLens, concept_id: u32) -> SemanticZoomCardLayerMode {
    lens.semantic_zoom_card_modes
        .iter()
        .find(|entry| entry.concept_id == concept_id)
        .map(|entry| entry.mode.clone())
        .unwrap_or(SemanticZoomCardLayerMode::Automatic)
}

fn normalized_card_layer_mode(
    mode: SemanticZoomCardLayerMode,
    layers: &[SemanticZoomLayer],
) -> Result<SemanticZoomCardLayerMode, WorkspaceError> {
    let requested_layer_ids = match mode {
        SemanticZoomCardLayerMode::Automatic => return Ok(SemanticZoomCardLayerMode::Automatic),
        SemanticZoomCardLayerMode::Manual { layer_ids } => layer_ids,
        SemanticZoomCardLayerMode::Dedicated { layer_id } => vec![layer_id],
        SemanticZoomCardLayerMode::AlwaysVisible => {
            if layers.is_empty() {
                return Ok(SemanticZoomCardLayerMode::Automatic);
            }
            layers.iter().map(|layer| layer.id.clone()).collect()
        }
    };
    if requested_layer_ids.is_empty() {
        return validation("manual card display requires at least one layer".to_string());
    }
    let requested_layer_id_set = requested_layer_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if requested_layer_id_set.len() != requested_layer_ids.len() {
        return validation(
            "manual card display cannot select one layer more than once".to_string(),
        );
    }
    let selected_layer_ids = layers
        .iter()
        .filter(|layer| requested_layer_id_set.contains(layer.id.as_str()))
        .map(|layer| layer.id.clone())
        .collect::<Vec<_>>();
    if selected_layer_ids.len() != requested_layer_ids.len() {
        let unknown_layer_id = requested_layer_ids
            .iter()
            .find(|layer_id| !layers.iter().any(|layer| layer.id == **layer_id))
            .cloned()
            .unwrap_or_else(|| "unknown layer".to_string());
        return validation(format!(
            "manual card display references unknown layer {unknown_layer_id}"
        ));
    }
    Ok(SemanticZoomCardLayerMode::Manual {
        layer_ids: selected_layer_ids,
    })
}

fn card_base_layer_index(lens: &WorkspaceLens, concept_id: u32, bounds: Rect) -> Option<usize> {
    card_layer_indices(lens, concept_id, bounds)
        .into_iter()
        .next()
}

fn card_layer_indices(lens: &WorkspaceLens, concept_id: u32, bounds: Rect) -> Vec<usize> {
    match card_layer_mode(lens, concept_id) {
        // Layer membership records preserve historical composition, but must
        // not pin an automatic card. Automatic is explicitly size-derived.
        SemanticZoomCardLayerMode::Automatic => {
            automatic_layer_index(&lens.semantic_zoom_layers, bounds)
                .into_iter()
                .collect()
        }
        SemanticZoomCardLayerMode::Manual { layer_ids } => lens
            .semantic_zoom_layers
            .iter()
            .enumerate()
            .filter_map(|(index, layer)| layer_ids.contains(&layer.id).then_some(index))
            .collect(),
        // These variants are migrated on load. Keeping their display behavior
        // here preserves source compatibility for in-memory callers.
        SemanticZoomCardLayerMode::Dedicated { layer_id } => lens
            .semantic_zoom_layers
            .iter()
            .position(|layer| layer.id == layer_id)
            .into_iter()
            .collect(),
        SemanticZoomCardLayerMode::AlwaysVisible => (0..lens.semantic_zoom_layers.len()).collect(),
    }
}

fn card_layer_visibility_at_bounds(
    lens: &WorkspaceLens,
    concept_id: u32,
    bounds: Rect,
) -> SemanticZoomCardLayerVisibility {
    let mode = card_layer_mode(lens, concept_id);
    let layer_indices = card_layer_indices(lens, concept_id, bounds);
    let visible_layer_ids = layer_indices
        .iter()
        .filter_map(|index| lens.semantic_zoom_layers.get(*index))
        .map(|layer| layer.id.clone())
        .collect();

    SemanticZoomCardLayerVisibility {
        mode,
        base_layer_id: layer_indices
            .first()
            .copied()
            .and_then(|index| lens.semantic_zoom_layers.get(index))
            .map(|layer| layer.id.clone()),
        visible_layer_ids,
    }
}

fn card_layer_presentation(
    lens: &WorkspaceLens,
    concept_id: u32,
    bounds: Rect,
    layer_projections: &[SemanticZoomLayerProjection],
) -> Option<(f32, f32)> {
    let selected_layer_index = card_layer_indices(lens, concept_id, bounds)
        .into_iter()
        .filter_map(|index| {
            layer_projections
                .get(index)
                .map(|projection| (index, projection.opacity))
        })
        .filter(|(_, opacity)| *opacity > 0.0)
        .max_by(|(left_index, left_opacity), (right_index, right_opacity)| {
            left_opacity
                .total_cmp(right_opacity)
                .then(left_index.cmp(right_index))
        })
        .map(|(index, _)| index)?;
    Some((
        layer_projections[selected_layer_index].opacity,
        lens.semantic_zoom_layers[selected_layer_index].card_scale,
    ))
}

fn assigned_card_scale(
    lens: &WorkspaceLens,
    concept_id: u32,
    bounds: Rect,
    layer_projections: &[SemanticZoomLayerProjection],
) -> f32 {
    card_layer_presentation(lens, concept_id, bounds, layer_projections)
        .map(|(_, card_scale)| card_scale)
        .or_else(|| {
            card_base_layer_index(lens, concept_id, bounds)
                .and_then(|layer_index| lens.semantic_zoom_layers.get(layer_index))
                .map(|layer| layer.card_scale)
        })
        .unwrap_or(1.0)
}

fn effective_percent_settings(layer: &SemanticZoomLayer) -> SemanticZoomLayerPercentSettings {
    layer
        .percent_settings
        .unwrap_or_else(|| percent_settings_from_legacy_fades(layer.fade_in, layer.fade_out))
}

fn percent_settings_from_legacy_fades(
    fade_in: Option<ZoomRange>,
    fade_out: Option<ZoomRange>,
) -> SemanticZoomLayerPercentSettings {
    let start_percent = fade_in
        .map(|range| range.end * 100.0)
        .unwrap_or(MINIMUM_ZOOM_PERCENT)
        .clamp(MINIMUM_ZOOM_PERCENT, MAXIMUM_ZOOM_PERCENT);
    let end_percent = fade_out
        .map(|range| range.start * 100.0)
        .unwrap_or(MAXIMUM_ZOOM_PERCENT)
        .clamp(start_percent, MAXIMUM_ZOOM_PERCENT);
    let fade_percent = [fade_in, fade_out]
        .into_iter()
        .flatten()
        .map(|range| (range.end - range.start).abs() * 100.0)
        .fold(0.0_f32, f32::max);
    SemanticZoomLayerPercentSettings {
        active_zoom_range: ZoomPercentRange {
            start_percent,
            end_percent,
        },
        fade_percent,
    }
}

fn default_percent_settings(layer_index: usize) -> SemanticZoomLayerPercentSettings {
    let active_zoom_range = match layer_index {
        0 => ZoomPercentRange {
            start_percent: MINIMUM_ZOOM_PERCENT,
            end_percent: 65.0,
        },
        1 => ZoomPercentRange {
            start_percent: 66.0,
            end_percent: 150.0,
        },
        _ => ZoomPercentRange {
            start_percent: 151.0,
            end_percent: MAXIMUM_ZOOM_PERCENT,
        },
    };
    SemanticZoomLayerPercentSettings {
        active_zoom_range,
        fade_percent: 10.0,
    }
}

fn percent_distance_to_range(percent: f32, range: ZoomPercentRange) -> f32 {
    if percent < range.start_percent {
        range.start_percent - percent
    } else if percent > range.end_percent {
        percent - range.end_percent
    } else {
        0.0
    }
}

fn scale_bounds_from_center(bounds: Rect, scale: f32) -> Rect {
    let center = bounds.center();
    let size = bounds.size * scale;
    Rect::new(center - size * 0.5, size)
}

fn set_membership<T: Ord + Clone>(members: &mut Vec<T>, requested: &[T], included: bool) {
    let requested: BTreeSet<_> = requested.iter().cloned().collect();
    let mut combined: BTreeSet<_> = members.iter().cloned().collect();
    if included {
        combined.extend(requested);
    } else {
        for member in requested {
            combined.remove(&member);
        }
    }
    *members = combined.into_iter().collect();
}

fn next_semantic_zoom_layer_id(title: &str, layers: &[SemanticZoomLayer]) -> String {
    let slug = identifier_slug(title);
    let stem = if slug.is_empty() { "layer" } else { &slug };
    let base = format!("layer-{stem}");
    let existing_ids: BTreeSet<_> = layers.iter().map(|layer| layer.id.as_str()).collect();
    let mut identifier = base.clone();
    let mut suffix = 2;
    while existing_ids.contains(identifier.as_str()) {
        identifier = format!("{base}-{suffix}");
        suffix += 1;
    }
    identifier
}

fn next_visual_boundary_id(title: &str, boundaries: &[VisualBoundary]) -> String {
    let slug = identifier_slug(title);
    let stem = if slug.is_empty() { "boundary" } else { &slug };
    let base = format!("boundary-{stem}");
    let existing_ids: BTreeSet<_> = boundaries
        .iter()
        .map(|boundary| boundary.id.as_str())
        .collect();
    next_available_identifier(&base, &existing_ids)
}

fn next_lens_id(title: &str, lenses: &[WorkspaceLens]) -> String {
    let slug = identifier_slug(title);
    let stem = if slug.is_empty() { "view" } else { &slug };
    let base = format!("lens-{stem}");
    let existing_ids: BTreeSet<_> = lenses.iter().map(|lens| lens.id.as_str()).collect();
    next_available_identifier(&base, &existing_ids)
}

fn next_workspace_version_id(title: &str, document: &WorkspaceDocument) -> String {
    let slug = identifier_slug(title);
    let stem = if slug.is_empty() { "version" } else { &slug };
    let base = format!("version-{stem}");
    let existing_ids: BTreeSet<_> = document.version_ids().into_iter().collect();
    next_available_identifier(&base, &existing_ids)
}

fn identifier_slug(title: &str) -> String {
    let mut slug = title
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

fn next_available_identifier(base: &str, existing_ids: &BTreeSet<&str>) -> String {
    let mut identifier = base.to_string();
    let mut suffix = 2;
    while existing_ids.contains(identifier.as_str()) {
        identifier = format!("{base}-{suffix}");
        suffix += 1;
    }
    identifier
}

fn validate_link_endpoints(
    version: &WorkspaceVersion,
    source_id: u32,
    target_id: u32,
    kind: RelationshipKind,
) -> Result<(), WorkspaceError> {
    let concept_ids: BTreeSet<_> = version.concepts.iter().map(|concept| concept.id).collect();
    for concept_id in [source_id, target_id] {
        if !concept_ids.contains(&concept_id) {
            return validation(format!(
                "cannot create {} relationship: concept {} does not exist in version {}",
                kind.identifier(),
                concept_id,
                version.id
            ));
        }
    }
    if source_id == target_id {
        return validation(format!("a concept cannot link to itself: {source_id}"));
    }
    if let Some(lens) = version.lenses.iter().find(|lens| {
        let parent_by_child = card_parent_ids(
            lens.placements
                .iter()
                .map(|placement| (placement.concept_id, placement.bounds)),
        );
        cards_are_in_containment_chain(&parent_by_child, source_id, target_id)
    }) {
        return validation(format!(
            "cannot create {} relationship from concept {source_id} to concept {target_id}: cards share a containment chain in lens {}",
            kind.identifier(),
            lens.id,
        ));
    }
    if version.relationships.iter().any(|relationship| {
        relationship.kind == kind
            && relationship.source_id == source_id
            && relationship.target_id == target_id
    }) {
        return validation(format!(
            "duplicate {} relationship from {source_id} to {target_id}",
            kind.identifier()
        ));
    }
    Ok(())
}

fn invalid_deletion_receipt(deletion: &DeletedConcept, reason: &str) -> WorkspaceError {
    WorkspaceError::InvalidConceptDeletionReceipt {
        version_id: deletion.version_id.clone(),
        reason: reason.to_string(),
    }
}

fn validate_deleted_records(deletion: &DeletedConcept) -> Result<(), WorkspaceError> {
    let concept_id = deletion.concept.id;
    let mut relationship_indices = BTreeSet::new();
    for deleted in &deletion.relationships {
        if !relationship_indices.insert(deleted.index) {
            return Err(invalid_deletion_receipt(
                deletion,
                "relationship insertion indices must be unique",
            ));
        }
        if deleted.relationship.source_id != concept_id
            && deleted.relationship.target_id != concept_id
        {
            return Err(invalid_deletion_receipt(
                deletion,
                "relationship does not reference the deleted concept",
            ));
        }
    }

    let mut placement_indices = BTreeSet::new();
    for deleted in &deletion.placements {
        if !placement_indices.insert((&deleted.lens_id, deleted.index)) {
            return Err(invalid_deletion_receipt(
                deletion,
                "placement insertion indices must be unique within a lens",
            ));
        }
        if deleted.placement.concept_id != concept_id {
            return Err(invalid_deletion_receipt(
                deletion,
                "placement does not reference the deleted concept",
            ));
        }
    }

    let mut card_layer_mode_indices = BTreeSet::new();
    for deleted in &deletion.card_layer_modes {
        if !card_layer_mode_indices.insert((&deleted.lens_id, deleted.index)) {
            return Err(invalid_deletion_receipt(
                deletion,
                "card layer mode insertion indices must be unique within a lens",
            ));
        }
        if deleted.entry.concept_id != concept_id {
            return Err(invalid_deletion_receipt(
                deletion,
                "card layer mode does not reference the deleted concept",
            ));
        }
    }

    Ok(())
}

fn validate_required(name: &str, value: &str) -> Result<(), WorkspaceError> {
    if value.trim().is_empty() {
        return validation(format!("{name} must not be empty"));
    }
    Ok(())
}

fn validation<T>(message: String) -> Result<T, WorkspaceError> {
    Err(WorkspaceError::Validation(message))
}

fn has_valid_bounds(bounds: Rect) -> bool {
    bounds.origin.x.is_finite()
        && bounds.origin.y.is_finite()
        && bounds.size.x.is_finite()
        && bounds.size.y.is_finite()
        && !bounds.is_empty()
}

fn parse_hex_color(color: &str) -> Option<[f32; 4]> {
    let value = color.strip_prefix('#')?;
    if value.len() != 6 {
        return None;
    }
    let red = u8::from_str_radix(&value[0..2], 16).ok()?;
    let green = u8::from_str_radix(&value[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&value[4..6], 16).ok()?;
    Some([
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        1.0,
    ])
}

fn concept(id: u32, type_id: &str, title: &str, summary: &str) -> WorkspaceConcept {
    WorkspaceConcept {
        content_reference: None,
        id,
        type_id: type_id.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        details: String::new(),
        execution_mode: (type_id == "process").then_some(ExecutionMode::Automated),
        attributes: BTreeMap::new(),
    }
}

fn relationship(
    id: &str,
    kind: RelationshipKind,
    source_id: u32,
    target_id: u32,
) -> TypedRelationship {
    TypedRelationship {
        id: id.to_string(),
        kind,
        source_id,
        target_id,
        label: kind.label().to_string(),
    }
}

fn placement(concept_id: u32, x: f32, y: f32) -> ConceptPlacement {
    ConceptPlacement {
        concept_id,
        bounds: Rect::new(Point::new(x, y), Point::new(240.0, 132.0)),
    }
}

#[derive(Clone, Debug, Deserialize)]
struct LegacyWorkspaceDocument {
    workspace_id: String,
    title: String,
    concepts: Vec<LegacyWorkspaceConcept>,
    relationships: Vec<LegacyTypedRelationship>,
    views: Vec<LegacyWorkspaceView>,
}

#[derive(Clone, Debug, Deserialize)]
struct LegacyWorkspaceConcept {
    id: u32,
    kind: ConceptKind,
    title: String,
    summary: String,
    #[serde(default)]
    details: String,
    #[serde(default)]
    attributes: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacyRelationshipKind {
    Owns,
    ManagedBy,
    Delivers,
    DeployedTo,
    OperatedBy,
    DependsOn,
}

impl LegacyRelationshipKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Owns => "owns",
            Self::ManagedBy => "managed by",
            Self::Delivers => "delivers",
            Self::DeployedTo => "deployed to",
            Self::OperatedBy => "operated by",
            Self::DependsOn => "depends on",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct LegacyTypedRelationship {
    id: String,
    kind: LegacyRelationshipKind,
    source_id: u32,
    target_id: u32,
    #[serde(default)]
    label: String,
}

#[derive(Clone, Debug, Deserialize)]
struct LegacyWorkspaceView {
    id: String,
    title: String,
    placements: Vec<ConceptPlacement>,
}

impl LegacyWorkspaceDocument {
    fn from_value(value: serde_json::Value) -> Result<Self, WorkspaceError> {
        serde_json::from_value(value)
            .map_err(|error| WorkspaceError::InvalidJson(error.to_string()))
    }

    fn into_workspace_document(self) -> WorkspaceDocument {
        let legacy_concept_types: BTreeMap<_, _> = self
            .concepts
            .iter()
            .map(|concept| (concept.id, concept.kind))
            .collect();
        let concepts = self
            .concepts
            .into_iter()
            .map(|concept| WorkspaceConcept {
                content_reference: None,
                id: concept.id,
                type_id: concept.kind.type_id().to_string(),
                title: concept.title,
                summary: concept.summary,
                details: concept.details,
                execution_mode: None,
                attributes: concept.attributes,
            })
            .collect();
        let relationships = self
            .relationships
            .into_iter()
            .map(|relationship| {
                let legacy_label = relationship.kind.label();
                let (kind, source_id, target_id) = match relationship.kind {
                    LegacyRelationshipKind::Owns => (
                        if legacy_concept_types.get(&relationship.source_id)
                            == Some(&ConceptKind::Team)
                        {
                            RelationshipKind::Owns
                        } else {
                            RelationshipKind::Contains
                        },
                        relationship.source_id,
                        relationship.target_id,
                    ),
                    LegacyRelationshipKind::ManagedBy => (
                        RelationshipKind::Operates,
                        relationship.target_id,
                        relationship.source_id,
                    ),
                    LegacyRelationshipKind::Delivers => (
                        RelationshipKind::Delivers,
                        relationship.source_id,
                        relationship.target_id,
                    ),
                    LegacyRelationshipKind::DeployedTo => (
                        RelationshipKind::DeployedTo,
                        relationship.source_id,
                        relationship.target_id,
                    ),
                    LegacyRelationshipKind::OperatedBy => (
                        RelationshipKind::Operates,
                        relationship.target_id,
                        relationship.source_id,
                    ),
                    LegacyRelationshipKind::DependsOn => (
                        RelationshipKind::DependsOn,
                        relationship.source_id,
                        relationship.target_id,
                    ),
                };
                TypedRelationship {
                    id: relationship.id,
                    kind,
                    source_id,
                    target_id,
                    label: if relationship.label.trim().is_empty()
                        || relationship.label == legacy_label
                    {
                        kind.label().to_string()
                    } else {
                        relationship.label
                    },
                }
            })
            .collect();
        let lenses = self
            .views
            .into_iter()
            .map(|view| WorkspaceLens {
                relationships: None,
                id: view.id,
                title: view.title,
                template: None,
                filter: LensFilter::default(),
                placements: view.placements,
                visual_boundaries: Vec::new(),
                semantic_zoom_layers: Vec::new(),
                semantic_zoom_card_modes: Vec::new(),
            })
            .collect();

        WorkspaceDocument {
            schema_version: WORKSPACE_SCHEMA_VERSION,
            workspace_id: self.workspace_id,
            title: self.title,
            custom_types: Vec::new(),
            current_state: WorkspaceVersion {
                id: "current".to_string(),
                title: "Current State".to_string(),
                concepts,
                relationships,
                lenses,
            },
            planning_scenarios: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        ConceptPlacement, CustomType, LensFilter, NEW_CARD_BASELINE_SIZE_AT_100_PERCENT,
        RelationshipKind, RelationshipOverlayPlacementOverrides, RelationshipOverlayTarget,
        SelectedConceptOverlaySource, SemanticZoomCardLayerMode, SemanticZoomCardLayerModeEntry,
        SemanticZoomCardLayerVisibility, SemanticZoomLayerPercentSettings,
        SemanticZoomLayerSettings, VisualBoundary, WORKSPACE_SCHEMA_VERSION, WorkspaceDocument,
        WorkspaceError, ZoomPercentRange, ZoomRange, concept, layer_opacity_at_scale, placement,
        relationship,
    };
    use crate::{Point, Rect};

    #[test]
    fn sample_workspace_is_valid_and_deterministic() {
        let document = WorkspaceDocument::sample();

        let first = document.to_pretty_json().unwrap();
        let second = WorkspaceDocument::parse_json(&first)
            .unwrap()
            .to_pretty_json()
            .unwrap();

        assert_eq!(first, second);
        assert_eq!(document.current_state_id(), "current");
        assert_eq!(document.lens_ids("current").unwrap().len(), 2);
    }

    #[test]
    fn blank_workspace_is_valid_and_deterministic() {
        let document = WorkspaceDocument::blank(
            "finance-reporting".to_string(),
            "Finance reporting".to_string(),
        )
        .unwrap();

        let source = document.to_pretty_json().unwrap();
        let restored = WorkspaceDocument::parse_json(&source).unwrap();

        assert_eq!(restored, document);
        assert_eq!(document.current_state_id(), "current");
        assert_eq!(document.default_lens_id("current").unwrap(), "canvas");
        assert!(document.current_state.concepts.is_empty());
        assert!(document.current_state.relationships.is_empty());
    }

    #[test]
    fn new_version_is_a_complete_copy_of_its_selected_source() {
        let mut document = WorkspaceDocument::sample();
        let source = document.version("current").unwrap().clone();

        let version_id = document
            .create_version_from("current", "Proposed platform".to_string())
            .unwrap();

        let copied = document.version(&version_id).unwrap();
        assert_eq!(version_id, "version-proposed-platform");
        assert_eq!(copied.title, "Proposed platform");
        assert_eq!(copied.concepts, source.concepts);
        assert_eq!(copied.relationships, source.relationships);
        assert_eq!(copied.lenses, source.lenses);
        assert_eq!(
            document.planning_scenarios[0].based_on_version_id,
            "current"
        );
        document.validate().unwrap();
    }

    #[test]
    fn view_relationship_creation_kind_and_removal_are_isolated_after_reload() {
        let mut document = WorkspaceDocument::sample();
        let original = document.clone();
        let source = "organization-landscape";
        let clone = document
            .create_lens_from("current", source, "Independent connections".to_string())
            .unwrap();
        let created = document
            .link_concepts_in_lens("current", source, 2, 4, RelationshipKind::Owns)
            .unwrap();
        assert!(
            document
                .relationships_in_lens("current", source)
                .unwrap()
                .contains(&created)
        );
        assert!(
            !document
                .relationships_in_lens("current", &clone)
                .unwrap()
                .iter()
                .any(|relationship| relationship.id == created.id)
        );
        document
            .change_relationship_kind_in_lens(
                "current",
                &clone,
                "rel-portal-api",
                RelationshipKind::Delivers,
            )
            .unwrap();
        assert_eq!(
            document
                .relationships_in_lens("current", source)
                .unwrap()
                .iter()
                .find(|relationship| relationship.id == "rel-portal-api")
                .unwrap()
                .kind,
            RelationshipKind::DependsOn
        );
        document
            .remove_relationship_in_lens("current", source, "rel-release-api")
            .unwrap();
        assert!(
            document
                .relationships_in_lens("current", &clone)
                .unwrap()
                .iter()
                .any(|relationship| relationship.id == "rel-release-api")
        );
        document
            .edit_concept("current", 5, "service", "Shared title".to_string())
            .unwrap();
        let restored = WorkspaceDocument::parse_json(&document.to_pretty_json().unwrap()).unwrap();
        assert_eq!(restored, document);
        for lens in [source, clone.as_str()] {
            assert_eq!(
                restored
                    .canvas_dataset("current", lens)
                    .unwrap()
                    .card_by_id(5)
                    .unwrap()
                    .title,
                "Shared title"
            );
            let projected = restored.canvas_dataset("current", lens).unwrap();
            let semantic = restored.semantic_zoom_frame("current", lens, 1.0).unwrap();
            assert_eq!(projected.relationship_count(), semantic.relationships.len());
        }
        assert_eq!(
            restored.current_state.relationships,
            original.current_state.relationships
        );
        let deleted = document.delete_concept("current", 4).unwrap();
        document.restore_deleted_concept(&deleted).unwrap();
        assert_eq!(document, restored);
    }

    #[test]
    fn version_and_lens_crud_preserve_a_valid_document() {
        let mut document = WorkspaceDocument::sample();
        let created_version = document
            .create_version_from("current", "Proposed platform".to_string())
            .unwrap();
        document
            .rename_version(&created_version, "Approved platform".to_string())
            .unwrap();
        assert_eq!(
            document.version_title(&created_version).unwrap(),
            "Approved platform"
        );

        let source_lens = document
            .version("current")
            .unwrap()
            .lenses
            .first()
            .unwrap()
            .clone();
        let copied_lens = document
            .create_lens_from("current", &source_lens.id, "Delivery focus".to_string())
            .unwrap();
        let mut copied_lens_record = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == copied_lens)
            .unwrap()
            .clone();
        copied_lens_record.id = source_lens.id.clone();
        copied_lens_record.title = source_lens.title.clone();
        copied_lens_record.relationships = source_lens.relationships.clone();
        assert_eq!(copied_lens_record, source_lens);
        document
            .rename_lens("current", &copied_lens, "Delivery overview".to_string())
            .unwrap();
        document.delete_lens("current", &copied_lens).unwrap();
        document.delete_version(&created_version).unwrap();
        document.validate().unwrap();
    }

    #[test]
    fn empty_lens_keeps_workspace_facts_but_starts_without_presentation_records() {
        let mut document = WorkspaceDocument::sample();
        let source = document.version("current").unwrap().clone();

        let lens_id = document
            .create_empty_lens("current", "Blank delivery view".to_string())
            .unwrap();

        let version = document.version("current").unwrap();
        let lens = version
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .unwrap();
        assert_eq!(version.concepts, source.concepts);
        assert_eq!(version.relationships, source.relationships);
        assert_eq!(lens.title, "Blank delivery view");
        assert_eq!(lens.filter, LensFilter::default());
        assert!(lens.placements.is_empty());
        assert!(lens.visual_boundaries.is_empty());
        assert!(lens.semantic_zoom_layers.is_empty());
        assert!(lens.semantic_zoom_card_modes.is_empty());
        document.validate().unwrap();
    }

    #[test]
    fn captured_cards_paste_into_other_views_and_versions_without_changing_source() {
        let mut document = WorkspaceDocument::sample();
        let source = document.clone();
        let source_title = source.concept("current", 5).unwrap().title.clone();
        let view = document
            .create_empty_lens("current", "Paste destination".to_string())
            .unwrap();
        let copied = document
            .copy_concepts_from_lens(
                &source,
                "current",
                "organization-landscape",
                "current",
                &view,
                &[5],
                Point::new(100.0, 200.0),
            )
            .unwrap();
        assert_eq!(copied[0], 5);
        assert_eq!(
            document.concept("current", copied[0]).unwrap().title,
            source_title
        );
        assert_eq!(
            document.version("current").unwrap().lenses[0],
            source.version("current").unwrap().lenses[0]
        );
        let version = document
            .create_version_from("current", "Paste version".to_string())
            .unwrap();
        document
            .edit_concept("current", 5, "service", "Changed after copy".to_string())
            .unwrap();
        let lens = document.default_lens_id(&version).unwrap().to_string();
        let copied = document
            .copy_concepts_from_lens(
                &source,
                "current",
                "organization-landscape",
                &version,
                &lens,
                &[5],
                Point::new(300.0, 400.0),
            )
            .unwrap();
        assert_eq!(
            document.concept(&version, copied[0]).unwrap().title,
            source_title
        );
        document.validate().unwrap();
    }

    #[test]
    fn copied_cards_keep_lens_placement_without_copying_relationships() {
        let mut document = WorkspaceDocument::sample();
        let lens_id = "organization-landscape";
        let original_relationships = document.version("current").unwrap().relationships.clone();
        let source_placement = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .unwrap()
            .placements
            .iter()
            .find(|placement| placement.concept_id == 5)
            .unwrap()
            .clone();

        let copied_ids = document
            .duplicate_concepts_in_lens("current", lens_id, &[5], Point::new(48.0, 48.0))
            .unwrap();

        assert_eq!(copied_ids.len(), 1);
        let copied_id = copied_ids[0];
        assert_ne!(copied_id, 5);
        assert_eq!(
            document.concept("current", copied_id).unwrap().title,
            document.concept("current", 5).unwrap().title
        );
        assert_eq!(
            document.version("current").unwrap().relationships,
            original_relationships
        );
        let copied_placement = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == lens_id)
            .unwrap()
            .placements
            .iter()
            .find(|placement| placement.concept_id == copied_id)
            .unwrap();
        assert_eq!(
            copied_placement.bounds,
            Rect::new(
                source_placement.bounds.origin + Point::new(48.0, 48.0),
                source_placement.bounds.size
            )
        );
        document.validate().unwrap();
    }

    #[test]
    fn semantic_zoom_keeps_manually_selected_cards_in_each_selected_layer() {
        let mut document = WorkspaceDocument::sample();
        let overview = document
            .create_semantic_zoom_layer(
                "current",
                "organization-landscape",
                "Organization overview".to_string(),
            )
            .unwrap();
        let delivery = document
            .create_semantic_zoom_layer(
                "current",
                "organization-landscape",
                "Delivery path".to_string(),
            )
            .unwrap();
        let detail = document
            .create_semantic_zoom_layer(
                "current",
                "organization-landscape",
                "Delivery detail".to_string(),
            )
            .unwrap();
        document
            .update_semantic_zoom_layer(
                "current",
                "organization-landscape",
                &overview,
                SemanticZoomLayerSettings {
                    title: "Organization overview".to_string(),
                    fade_in: None,
                    fade_out: Some(ZoomRange {
                        start: 0.75,
                        end: 0.85,
                    }),
                    card_scale: 0.5,
                },
            )
            .unwrap();
        document
            .update_semantic_zoom_layer(
                "current",
                "organization-landscape",
                &delivery,
                SemanticZoomLayerSettings {
                    title: "Delivery path".to_string(),
                    fade_in: Some(ZoomRange {
                        start: 0.55,
                        end: 0.65,
                    }),
                    fade_out: Some(ZoomRange {
                        start: 1.6,
                        end: 1.7,
                    }),
                    card_scale: 0.7,
                },
            )
            .unwrap();
        document
            .update_semantic_zoom_layer(
                "current",
                "organization-landscape",
                &detail,
                SemanticZoomLayerSettings {
                    title: "Delivery detail".to_string(),
                    fade_in: Some(ZoomRange {
                        start: 1.4,
                        end: 1.5,
                    }),
                    fade_out: None,
                    card_scale: 1.0,
                },
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_concepts(
                "current",
                "organization-landscape",
                &overview,
                &[1, 2, 3],
                true,
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_concepts(
                "current",
                "organization-landscape",
                &delivery,
                &[4, 5, 6],
                true,
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_concepts(
                "current",
                "organization-landscape",
                &detail,
                &[7, 8, 9],
                true,
            )
            .unwrap();
        document
            .set_semantic_zoom_card_layer_mode(
                "current",
                "organization-landscape",
                1,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![overview.clone(), delivery.clone()],
                },
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_relationships(
                "current",
                "organization-landscape",
                &overview,
                &["rel-product-portal".to_string()],
                true,
            )
            .unwrap();

        let overview_frame = document
            .semantic_zoom_frame("current", "organization-landscape", 0.6)
            .unwrap();
        assert_eq!(
            overview_frame
                .layers
                .iter()
                .map(|layer| layer.id.as_str())
                .collect::<Vec<_>>(),
            vec![overview.as_str(), delivery.as_str(), detail.as_str()]
        );
        assert_eq!(overview_frame.layers[0].opacity, 1.0);
        assert!((overview_frame.layers[1].opacity - 0.5).abs() < 0.000_01);
        assert_eq!(overview_frame.layers[2].opacity, 0.0);
        assert!(overview_frame.cards.iter().any(|card| card.card.id == 1));
        assert!(overview_frame.cards.iter().any(|card| card.card.id == 7));
        assert!(
            overview_frame
                .relationships
                .iter()
                .any(|relationship| relationship.id == "rel-release-api")
        );

        let detail_frame = document
            .semantic_zoom_frame("current", "organization-landscape", 1.6)
            .unwrap();
        assert_eq!(
            detail_frame
                .layers
                .iter()
                .map(|layer| (layer.id.as_str(), layer.opacity))
                .collect::<Vec<_>>(),
            vec![
                (overview.as_str(), 0.0),
                (delivery.as_str(), 1.0),
                (detail.as_str(), 1.0),
            ]
        );
        let organization_card = detail_frame
            .cards
            .iter()
            .find(|card| card.card.id == 1)
            .unwrap();
        assert_eq!(organization_card.opacity, 1.0);
        assert_eq!(organization_card.card_scale, 0.7);
        assert!(detail_frame.cards.iter().any(|card| card.card.id == 7));
        assert!(
            detail_frame
                .relationships
                .iter()
                .any(|relationship| relationship.id == "rel-product-portal")
        );
    }

    #[test]
    fn percent_layer_settings_use_one_symmetric_fade_distance() {
        let mut document = WorkspaceDocument::sample();
        let layer_id = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Working".to_string())
            .unwrap();
        let settings = SemanticZoomLayerPercentSettings {
            active_zoom_range: ZoomPercentRange {
                start_percent: 50.0,
                end_percent: 100.0,
            },
            fade_percent: 11.0,
        };
        document
            .update_semantic_zoom_layer_percent_settings(
                "current",
                "organization-landscape",
                &layer_id,
                "Working".to_string(),
                settings,
                0.8,
            )
            .unwrap();

        assert_eq!(
            document
                .semantic_zoom_layer_percent_settings(
                    "current",
                    "organization-landscape",
                    &layer_id,
                )
                .unwrap(),
            settings
        );
        let layer = &document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers[0];
        assert_eq!(layer_opacity_at_scale(layer, 0.39), 0.0);
        assert!((layer_opacity_at_scale(layer, 0.445) - 0.5).abs() < 0.000_01);
        assert_eq!(layer_opacity_at_scale(layer, 0.50), 1.0);
        assert_eq!(layer_opacity_at_scale(layer, 1.0), 1.0);
        assert!((layer_opacity_at_scale(layer, 1.055) - 0.5).abs() < 0.000_01);
        assert_eq!(layer_opacity_at_scale(layer, 1.11), 0.0);
    }

    #[test]
    fn schema_four_fade_ranges_migrate_to_one_percent_range_and_fade() {
        let mut document = WorkspaceDocument::sample();
        let layer_id = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Working".to_string())
            .unwrap();
        let layer = document
            .current_state
            .lenses
            .iter_mut()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers
            .iter_mut()
            .find(|layer| layer.id == layer_id)
            .unwrap();
        layer.percent_settings = None;
        layer.fade_in = Some(ZoomRange {
            start: 0.55,
            end: 0.65,
        });
        layer.fade_out = Some(ZoomRange {
            start: 1.6,
            end: 1.7,
        });
        document.schema_version = 4;
        let source = serde_json::to_string(&document).unwrap();

        let migrated = WorkspaceDocument::parse_json(&source).unwrap();

        assert_eq!(migrated.schema_version, WORKSPACE_SCHEMA_VERSION);
        let settings = migrated
            .semantic_zoom_layer_percent_settings("current", "organization-landscape", &layer_id)
            .unwrap();
        assert_eq!(
            settings.active_zoom_range,
            ZoomPercentRange {
                start_percent: 65.0,
                end_percent: 160.0,
            }
        );
        assert!((settings.fade_percent - 10.0).abs() < 0.000_01);
    }

    #[test]
    fn schema_five_card_modes_migrate_to_manual_layer_selections() {
        let mut document = WorkspaceDocument::sample();
        let overview = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Overview".to_string())
            .unwrap();
        let detail = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Detail".to_string())
            .unwrap();
        let lens = document
            .version_mut("current")
            .unwrap()
            .lenses
            .iter_mut()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        lens.semantic_zoom_card_modes = vec![
            SemanticZoomCardLayerModeEntry {
                concept_id: 1,
                mode: SemanticZoomCardLayerMode::Dedicated {
                    layer_id: overview.clone(),
                },
            },
            SemanticZoomCardLayerModeEntry {
                concept_id: 2,
                mode: SemanticZoomCardLayerMode::AlwaysVisible,
            },
        ];
        document.schema_version = 5;

        let migrated =
            WorkspaceDocument::parse_json(&serde_json::to_string(&document).unwrap()).unwrap();
        let migrated_lens = migrated
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();

        assert_eq!(migrated.schema_version, WORKSPACE_SCHEMA_VERSION);
        assert_eq!(
            migrated_lens.semantic_zoom_card_modes,
            vec![
                SemanticZoomCardLayerModeEntry {
                    concept_id: 1,
                    mode: SemanticZoomCardLayerMode::Manual {
                        layer_ids: vec![overview.clone()],
                    },
                },
                SemanticZoomCardLayerModeEntry {
                    concept_id: 2,
                    mode: SemanticZoomCardLayerMode::Manual {
                        layer_ids: vec![overview, detail],
                    },
                },
            ]
        );
    }

    #[test]
    fn automatic_card_layer_recomputes_after_a_lens_local_resize() {
        let mut document = WorkspaceDocument::sample();
        let overview = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Overview".to_string())
            .unwrap();
        let working = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Working".to_string())
            .unwrap();
        document
            .update_semantic_zoom_layer_percent_settings(
                "current",
                "organization-landscape",
                &overview,
                "Overview".to_string(),
                SemanticZoomLayerPercentSettings {
                    active_zoom_range: ZoomPercentRange {
                        start_percent: 1.0,
                        end_percent: 65.0,
                    },
                    fade_percent: 0.0,
                },
                0.5,
            )
            .unwrap();
        document
            .update_semantic_zoom_layer_percent_settings(
                "current",
                "organization-landscape",
                &working,
                "Working".to_string(),
                SemanticZoomLayerPercentSettings {
                    active_zoom_range: ZoomPercentRange {
                        start_percent: 66.0,
                        end_percent: 150.0,
                    },
                    fade_percent: 0.0,
                },
                0.8,
            )
            .unwrap();
        let card_id = document
            .add_concept_to_lens(
                "current",
                "organization-landscape",
                "project",
                "Automatic card".to_string(),
                String::new(),
                Rect::new(Point::new(1_800.0, 120.0), Point::new(240.0, 120.0)),
            )
            .unwrap();
        // Existing workspaces can retain first-layer membership from the
        // initial base-layer setup. Automatic cards must still follow size.
        document
            .set_semantic_zoom_layer_concepts(
                "current",
                "organization-landscape",
                &overview,
                &[card_id],
                true,
            )
            .unwrap();

        let working_frame = document
            .semantic_zoom_frame("current", "organization-landscape", 1.0)
            .unwrap();
        assert_eq!(
            working_frame
                .cards
                .iter()
                .find(|card| card.card.id == card_id)
                .unwrap()
                .card_scale,
            0.8
        );
        assert_eq!(
            document
                .semantic_zoom_card_layer_visibility("current", "organization-landscape", card_id,)
                .unwrap(),
            SemanticZoomCardLayerVisibility {
                mode: SemanticZoomCardLayerMode::Automatic,
                base_layer_id: Some(working.clone()),
                visible_layer_ids: vec![working.clone()],
            }
        );
        assert_eq!(
            document
                .semantic_zoom_card_layer_visibility_at_bounds(
                    "current",
                    "organization-landscape",
                    card_id,
                    Rect::new(Point::new(1_800.0, 120.0), Point::new(480.0, 240.0)),
                )
                .unwrap(),
            SemanticZoomCardLayerVisibility {
                mode: SemanticZoomCardLayerMode::Automatic,
                base_layer_id: Some(overview.clone()),
                visible_layer_ids: vec![overview.clone()],
            }
        );

        document
            .resize_concept_in_lens(
                "current",
                "organization-landscape",
                card_id,
                Rect::new(Point::new(1_800.0, 120.0), Point::new(480.0, 240.0)),
            )
            .unwrap();
        let overview_frame = document
            .semantic_zoom_frame("current", "organization-landscape", 0.5)
            .unwrap();
        assert_eq!(
            overview_frame
                .cards
                .iter()
                .find(|card| card.card.id == card_id)
                .unwrap()
                .card_scale,
            0.5
        );
    }

    #[test]
    fn manual_card_mode_can_select_multiple_exact_layers() {
        let mut document = WorkspaceDocument::sample();
        let overview = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Overview".to_string())
            .unwrap();
        let working = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Working".to_string())
            .unwrap();
        for (layer_id, range) in [
            (
                &overview,
                ZoomPercentRange {
                    start_percent: 1.0,
                    end_percent: 65.0,
                },
            ),
            (
                &working,
                ZoomPercentRange {
                    start_percent: 66.0,
                    end_percent: 150.0,
                },
            ),
        ] {
            document
                .update_semantic_zoom_layer_percent_settings(
                    "current",
                    "organization-landscape",
                    layer_id,
                    layer_id.clone(),
                    SemanticZoomLayerPercentSettings {
                        active_zoom_range: range,
                        fade_percent: 0.0,
                    },
                    1.0,
                )
                .unwrap();
        }
        document
            .set_semantic_zoom_card_layer_mode(
                "current",
                "organization-landscape",
                4,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![working.clone()],
                },
            )
            .unwrap();
        assert!(
            !document
                .semantic_zoom_frame("current", "organization-landscape", 0.5)
                .unwrap()
                .cards
                .iter()
                .any(|card| card.card.id == 4)
        );

        document
            .set_semantic_zoom_card_layer_mode(
                "current",
                "organization-landscape",
                4,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![overview.clone(), working.clone()],
                },
            )
            .unwrap();
        assert!(
            document
                .semantic_zoom_frame("current", "organization-landscape", 0.5)
                .unwrap()
                .cards
                .iter()
                .any(|card| card.card.id == 4)
        );
        assert_eq!(
            document
                .semantic_zoom_card_layer_visibility("current", "organization-landscape", 4)
                .unwrap(),
            SemanticZoomCardLayerVisibility {
                mode: SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![overview.clone(), working.clone()],
                },
                base_layer_id: Some(overview.clone()),
                visible_layer_ids: vec![overview, working],
            }
        );
    }

    #[test]
    fn containing_card_remains_visible_as_context_for_a_deeper_child_layer() {
        let mut document = WorkspaceDocument::sample();
        let overview = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Overview".to_string())
            .unwrap();
        let detail = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Detail".to_string())
            .unwrap();
        for (layer_id, active_zoom_range) in [
            (
                &overview,
                ZoomPercentRange {
                    start_percent: 1.0,
                    end_percent: 65.0,
                },
            ),
            (
                &detail,
                ZoomPercentRange {
                    start_percent: 66.0,
                    end_percent: 150.0,
                },
            ),
        ] {
            document
                .update_semantic_zoom_layer_percent_settings(
                    "current",
                    "organization-landscape",
                    layer_id,
                    layer_id.clone(),
                    SemanticZoomLayerPercentSettings {
                        active_zoom_range,
                        fade_percent: 0.0,
                    },
                    1.0,
                )
                .unwrap();
        }
        let lens = document
            .current_state
            .lenses
            .iter_mut()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        lens.placements
            .iter_mut()
            .find(|placement| placement.concept_id == 1)
            .unwrap()
            .bounds = Rect::new(Point::new(0.0, 0.0), Point::new(960.0, 640.0));
        lens.placements
            .iter_mut()
            .find(|placement| placement.concept_id == 4)
            .unwrap()
            .bounds = Rect::new(
            Point::new(160.0, 160.0),
            NEW_CARD_BASELINE_SIZE_AT_100_PERCENT,
        );

        let frame = document
            .semantic_zoom_frame("current", "organization-landscape", 1.0)
            .unwrap();
        let rendered_card_ids = frame
            .cards
            .iter()
            .map(|card| card.card.id)
            .collect::<BTreeSet<_>>();

        assert!(rendered_card_ids.contains(&4));
        assert!(rendered_card_ids.contains(&1));
    }

    #[test]
    fn a_concept_and_relationship_have_one_semantic_zoom_base_layer() {
        let mut document = WorkspaceDocument::sample();
        let first = document
            .create_semantic_zoom_layer("current", "organization-landscape", "First".to_string())
            .unwrap();
        let second = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Second".to_string())
            .unwrap();
        document
            .set_semantic_zoom_layer_concepts(
                "current",
                "organization-landscape",
                &first,
                &[1],
                true,
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_concepts(
                "current",
                "organization-landscape",
                &second,
                &[1],
                true,
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_relationships(
                "current",
                "organization-landscape",
                &first,
                &["rel-product-portal".to_string()],
                true,
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_relationships(
                "current",
                "organization-landscape",
                &second,
                &["rel-product-portal".to_string()],
                true,
            )
            .unwrap();

        let lens = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        assert!(lens.semantic_zoom_layers[0].concept_ids.is_empty());
        assert_eq!(lens.semantic_zoom_layers[1].concept_ids, vec![1]);
        assert!(lens.semantic_zoom_layers[0].relationship_ids.is_empty());
        assert_eq!(
            lens.semantic_zoom_layers[1].relationship_ids,
            vec!["rel-product-portal"]
        );
    }

    #[test]
    fn direct_creation_uses_automatic_card_presentation() {
        let mut document = WorkspaceDocument::sample();
        let base_layer = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Overview".to_string())
            .unwrap();
        document
            .set_semantic_zoom_layer_concepts(
                "current",
                "organization-landscape",
                &base_layer,
                &[1],
                true,
            )
            .unwrap();
        let new_concept_id = document
            .add_concept_to_lens(
                "current",
                "organization-landscape",
                "project",
                "New delivery".to_string(),
                String::new(),
                Rect::new(Point::new(1_800.0, 120.0), Point::new(240.0, 120.0)),
            )
            .unwrap();
        let relationship = document
            .link_concepts("current", 1, new_concept_id, RelationshipKind::Contains)
            .unwrap();

        let base_layer = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers
            .first()
            .unwrap();
        assert!(!base_layer.concept_ids.contains(&new_concept_id));
        assert!(!base_layer.relationship_ids.contains(&relationship.id));
        assert_eq!(
            document
                .semantic_zoom_card_layer_mode("current", "organization-landscape", new_concept_id,)
                .unwrap(),
            super::SemanticZoomCardLayerMode::Automatic
        );
    }

    #[test]
    fn deleting_a_layer_reassigns_its_base_records_to_the_previous_layer() {
        let mut document = WorkspaceDocument::sample();
        let overview = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Overview".to_string())
            .unwrap();
        let delivery = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Delivery".to_string())
            .unwrap();
        document
            .set_semantic_zoom_layer_concepts(
                "current",
                "organization-landscape",
                &delivery,
                &[4],
                true,
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_relationships(
                "current",
                "organization-landscape",
                &delivery,
                &["rel-product-portal".to_string()],
                true,
            )
            .unwrap();
        document
            .delete_semantic_zoom_layer("current", "organization-landscape", &delivery)
            .unwrap();

        let layers = &document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers;
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].id, overview);
        assert!(layers[0].concept_ids.contains(&4));
        assert!(
            layers[0]
                .relationship_ids
                .contains(&"rel-product-portal".to_string())
        );
    }

    #[test]
    fn deleting_a_manual_layer_preserves_remaining_selections() {
        let mut document = WorkspaceDocument::sample();
        let overview = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Overview".to_string())
            .unwrap();
        let detail = document
            .create_semantic_zoom_layer("current", "organization-landscape", "Detail".to_string())
            .unwrap();
        document
            .set_semantic_zoom_card_layer_mode(
                "current",
                "organization-landscape",
                4,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![overview.clone(), detail.clone()],
                },
            )
            .unwrap();

        document
            .delete_semantic_zoom_layer("current", "organization-landscape", &overview)
            .unwrap();

        assert_eq!(
            document
                .semantic_zoom_card_layer_mode("current", "organization-landscape", 4)
                .unwrap(),
            SemanticZoomCardLayerMode::Manual {
                layer_ids: vec![detail.clone()],
            }
        );
        document
            .delete_semantic_zoom_layer("current", "organization-landscape", &detail)
            .unwrap();
        assert_eq!(
            document
                .semantic_zoom_card_layer_mode("current", "organization-landscape", 4)
                .unwrap(),
            SemanticZoomCardLayerMode::Automatic
        );
    }

    #[test]
    fn version_one_workspace_migrates_without_losing_relationships() {
        let source = include_str!("../../../tests/fixtures/acme-v1.fixture.json");

        let document = WorkspaceDocument::parse_json(source).unwrap();
        let version = document.version("current").unwrap();

        assert_eq!(document.schema_version, WORKSPACE_SCHEMA_VERSION);
        assert_eq!(version.relationships.len(), 3);
        assert!(version.relationships.iter().any(|relationship| {
            relationship.kind == RelationshipKind::Operates
                && relationship.source_id == 1
                && relationship.target_id == 3
        }));
    }

    #[test]
    fn legacy_non_team_ownership_migrates_to_a_structural_label() {
        let source = r#"
{
  "schema_version": 1,
  "workspace_id": "acme-platform",
  "title": "Acme Platform",
  "concepts": [
    {
      "id": 1,
      "kind": "service",
      "title": "Platform Service",
      "summary": "Provides shared platform capabilities"
    },
    {
      "id": 2,
      "kind": "environment",
      "title": "Production",
      "summary": "Production environment"
    }
  ],
  "relationships": [
    {
      "id": "rel-acme-platform",
      "kind": "owns",
      "source_id": 1,
      "target_id": 2
    }
  ],
  "views": [
    {
      "id": "platform-view",
      "title": "Platform view",
      "placements": [
        {
          "concept_id": 1,
          "bounds": {
            "origin": { "x": 0.0, "y": 0.0 },
            "size": { "x": 240.0, "y": 132.0 }
          }
        },
        {
          "concept_id": 2,
          "bounds": {
            "origin": { "x": 320.0, "y": 0.0 },
            "size": { "x": 240.0, "y": 132.0 }
          }
        }
      ]
    }
  ]
}
"#;
        let document = WorkspaceDocument::parse_json(source).unwrap();
        let relationship = document
            .version("current")
            .unwrap()
            .relationships
            .iter()
            .find(|relationship| relationship.id == "rel-acme-platform")
            .unwrap();

        assert_eq!(relationship.kind, RelationshipKind::Contains);
        assert_eq!(relationship.label, "contains");
    }

    #[test]
    fn custom_type_is_available_to_a_lens_and_uses_its_color() {
        let mut document = WorkspaceDocument::sample();
        document.custom_types.push(CustomType {
            id: "decision".to_string(),
            title: "Decision".to_string(),
            color: "#DC2626".to_string(),
            icon: Some("check".to_string()),
        });
        document
            .add_concept_to_lens(
                "current",
                "organization-landscape",
                "decision",
                "Choose deployment".to_string(),
                "A decision record".to_string(),
                Rect::new(Point::new(1800.0, 160.0), Point::new(240.0, 132.0)),
            )
            .unwrap();

        let dataset = document
            .canvas_dataset("current", "organization-landscape")
            .unwrap();
        let card = dataset
            .cards
            .iter()
            .find(|card| card.title == "Choose deployment")
            .unwrap();

        assert_eq!(
            card.accent_color,
            Some([220.0 / 255.0, 38.0 / 255.0, 38.0 / 255.0, 1.0])
        );
    }

    #[test]
    fn visual_boundary_is_lens_local_and_does_not_create_a_relationship() {
        let mut document = WorkspaceDocument::sample();
        let boundary_id = document
            .create_visual_boundary(
                "current",
                "organization-landscape",
                "Release context".to_string(),
                "Visual context for a delivery path.".to_string(),
                Rect::new(Point::new(600.0, 180.0), Point::new(640.0, 420.0)),
            )
            .unwrap();

        let version = document.version("current").unwrap();
        let boundary = version
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .visual_boundaries
            .first()
            .unwrap();
        assert_eq!(boundary.id, boundary_id);
        assert_eq!(boundary.title, "Release context");
        assert_eq!(version.relationships.len(), 8);
        assert!(
            version
                .lenses
                .iter()
                .find(|lens| lens.id == "delivery-and-production")
                .unwrap()
                .visual_boundaries
                .is_empty()
        );

        let serialized = document.to_pretty_json().unwrap();
        let restored = WorkspaceDocument::parse_json(&serialized).unwrap();
        assert_eq!(restored, document);
    }

    #[test]
    fn visual_boundary_edit_move_resize_and_removal_do_not_change_graph_facts() {
        let mut document = WorkspaceDocument::sample();
        let relationships_before = document.version("current").unwrap().relationships.clone();
        let boundary_id = document
            .create_visual_boundary(
                "current",
                "organization-landscape",
                "Release context".to_string(),
                String::new(),
                Rect::new(Point::new(600.0, 180.0), Point::new(640.0, 420.0)),
            )
            .unwrap();

        document
            .update_visual_boundary(
                "current",
                "organization-landscape",
                &boundary_id,
                "Shared delivery context".to_string(),
                "A visual reading aid, not a relationship.".to_string(),
            )
            .unwrap();
        document
            .move_visual_boundary(
                "current",
                "organization-landscape",
                &boundary_id,
                Point::new(24.0, -12.0),
            )
            .unwrap();
        document
            .resize_visual_boundary(
                "current",
                "organization-landscape",
                &boundary_id,
                Rect::new(Point::new(624.0, 168.0), Point::new(720.0, 480.0)),
            )
            .unwrap();

        let lens = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        let boundary = lens
            .visual_boundaries
            .iter()
            .find(|boundary| boundary.id == boundary_id)
            .unwrap();
        assert_eq!(boundary.title, "Shared delivery context");
        assert_eq!(
            boundary.summary,
            "A visual reading aid, not a relationship."
        );
        assert_eq!(
            boundary.bounds,
            Rect::new(Point::new(624.0, 168.0), Point::new(720.0, 480.0))
        );
        assert_eq!(
            document.version("current").unwrap().relationships,
            relationships_before
        );

        let removed = document
            .remove_visual_boundary("current", "organization-landscape", &boundary_id)
            .unwrap();
        assert_eq!(removed.id, boundary_id);
        assert!(
            document
                .version("current")
                .unwrap()
                .lenses
                .iter()
                .find(|lens| lens.id == "organization-landscape")
                .unwrap()
                .visual_boundaries
                .is_empty()
        );
        assert_eq!(
            document.version("current").unwrap().relationships,
            relationships_before
        );
    }

    #[test]
    fn moving_a_visual_boundary_moves_only_fully_enclosed_cards() {
        let mut document = WorkspaceDocument::sample();
        let lens = document
            .lens_mut("current", "organization-landscape")
            .unwrap();
        lens.placements = vec![
            ConceptPlacement {
                concept_id: 4,
                bounds: Rect::new(Point::new(140.0, 140.0), Point::new(120.0, 72.0)),
            },
            ConceptPlacement {
                concept_id: 5,
                bounds: Rect::new(Point::new(360.0, 180.0), Point::new(120.0, 72.0)),
            },
        ];
        let boundary_id = document
            .create_visual_boundary(
                "current",
                "organization-landscape",
                "Release context".to_string(),
                String::new(),
                Rect::new(Point::new(100.0, 100.0), Point::new(300.0, 220.0)),
            )
            .unwrap();

        let moved_cards = document
            .move_visual_boundary_with_contained_concepts(
                "current",
                "organization-landscape",
                &boundary_id,
                Point::new(32.0, -24.0),
            )
            .unwrap();

        assert_eq!(moved_cards, vec![4]);
        let lens = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        assert_eq!(
            lens.visual_boundaries[0].bounds.origin,
            Point::new(132.0, 76.0)
        );
        assert_eq!(lens.placements[0].bounds.origin, Point::new(172.0, 116.0));
        assert_eq!(lens.placements[1].bounds.origin, Point::new(360.0, 180.0));
    }

    #[test]
    fn moving_a_visual_boundary_moves_fully_nested_boundaries_once() {
        let mut document = WorkspaceDocument::sample();
        let outer_id = document
            .create_visual_boundary(
                "current",
                "organization-landscape",
                "Outer boundary".to_string(),
                String::new(),
                Rect::new(Point::new(100.0, 100.0), Point::new(640.0, 480.0)),
            )
            .unwrap();
        let nested_id = document
            .create_visual_boundary(
                "current",
                "organization-landscape",
                "Nested boundary".to_string(),
                String::new(),
                Rect::new(Point::new(180.0, 160.0), Point::new(240.0, 160.0)),
            )
            .unwrap();
        let crossing_id = document
            .create_visual_boundary(
                "current",
                "organization-landscape",
                "Crossing boundary".to_string(),
                String::new(),
                Rect::new(Point::new(680.0, 160.0), Point::new(240.0, 160.0)),
            )
            .unwrap();

        document
            .move_visual_boundary_with_contained_concepts(
                "current",
                "organization-landscape",
                &outer_id,
                Point::new(32.0, -24.0),
            )
            .unwrap();

        let lens = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        let bounds_by_id = lens
            .visual_boundaries
            .iter()
            .map(|boundary| (boundary.id.as_str(), boundary.bounds))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            bounds_by_id[outer_id.as_str()].origin,
            Point::new(132.0, 76.0)
        );
        assert_eq!(
            bounds_by_id[nested_id.as_str()].origin,
            Point::new(212.0, 136.0)
        );
        assert_eq!(
            bounds_by_id[crossing_id.as_str()].origin,
            Point::new(680.0, 160.0)
        );
    }

    #[test]
    fn visual_boundary_resize_rejects_non_positive_or_non_finite_bounds() {
        let mut document = WorkspaceDocument::sample();
        let boundary_id = document
            .create_visual_boundary(
                "current",
                "organization-landscape",
                "Release context".to_string(),
                String::new(),
                Rect::new(Point::new(600.0, 180.0), Point::new(640.0, 420.0)),
            )
            .unwrap();

        let error = document
            .resize_visual_boundary(
                "current",
                "organization-landscape",
                &boundary_id,
                Rect::new(Point::new(0.0, 0.0), Point::new(0.0, 120.0)),
            )
            .unwrap_err();
        assert!(error.to_string().contains("invalid bounds"));

        let error = document
            .move_visual_boundary(
                "current",
                "organization-landscape",
                &boundary_id,
                Point::new(f32::NAN, 0.0),
            )
            .unwrap_err();
        assert!(error.to_string().contains("movement delta must be finite"));
    }

    #[test]
    fn scenario_layout_changes_do_not_change_current_state() {
        let mut document = WorkspaceDocument::sample();
        document
            .create_planning_scenario(
                "release-plan".to_string(),
                "Release automation plan".to_string(),
                "current",
            )
            .unwrap();
        document
            .move_concept_in_lens(
                "release-plan",
                "organization-landscape",
                7,
                Point::new(300.0, 0.0),
            )
            .unwrap();

        let current = document
            .canvas_dataset("current", "organization-landscape")
            .unwrap();
        let scenario = document
            .canvas_dataset("release-plan", "organization-landscape")
            .unwrap();
        let current_card = current.cards.iter().find(|card| card.id == 7).unwrap();
        let scenario_card = scenario.cards.iter().find(|card| card.id == 7).unwrap();

        assert_ne!(current_card.bounds.origin, scenario_card.bounds.origin);
    }

    #[test]
    fn moving_a_placement_changes_only_its_lens() {
        let mut document = WorkspaceDocument::sample();
        let original = document
            .canvas_dataset("current", "delivery-and-production")
            .unwrap();
        let original_card = original
            .cards
            .iter()
            .find(|card| card.id == 5)
            .unwrap()
            .bounds
            .origin;

        document
            .move_concept_in_lens(
                "current",
                "organization-landscape",
                5,
                Point::new(160.0, -40.0),
            )
            .unwrap();

        let changed = document
            .canvas_dataset("current", "organization-landscape")
            .unwrap();
        let unchanged = document
            .canvas_dataset("current", "delivery-and-production")
            .unwrap();
        assert_eq!(
            changed
                .cards
                .iter()
                .find(|card| card.id == 5)
                .unwrap()
                .bounds
                .origin,
            Point::new(1240.0, 240.0)
        );
        assert_eq!(
            unchanged
                .cards
                .iter()
                .find(|card| card.id == 5)
                .unwrap()
                .bounds
                .origin,
            original_card
        );
    }

    #[test]
    fn moving_multiple_placements_preserves_relative_positions_and_other_lenses() {
        let mut document = WorkspaceDocument::sample();
        let original = document.clone();
        let world_delta = Point::new(160.0, -40.0);
        let selected_concept_ids = [5, 7];
        let original_positions: BTreeMap<_, _> = selected_concept_ids
            .into_iter()
            .map(|concept_id| {
                (
                    concept_id,
                    original
                        .canvas_dataset("current", "organization-landscape")
                        .unwrap()
                        .card_by_id(concept_id)
                        .unwrap()
                        .bounds
                        .origin,
                )
            })
            .collect();
        let other_lens_placements = original
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "delivery-and-production")
            .unwrap()
            .placements
            .clone();

        document
            .move_concepts_in_lens(
                "current",
                "organization-landscape",
                &selected_concept_ids,
                world_delta,
            )
            .unwrap();

        let changed = document
            .canvas_dataset("current", "organization-landscape")
            .unwrap();
        for concept_id in selected_concept_ids {
            assert_eq!(
                changed.card_by_id(concept_id).unwrap().bounds.origin,
                original_positions[&concept_id] + world_delta
            );
        }
        assert_eq!(
            changed.card_by_id(5).unwrap().bounds.origin
                - changed.card_by_id(7).unwrap().bounds.origin,
            original_positions[&5] - original_positions[&7]
        );
        assert_eq!(
            document
                .version("current")
                .unwrap()
                .lenses
                .iter()
                .find(|lens| lens.id == "delivery-and-production")
                .unwrap()
                .placements,
            other_lens_placements
        );
    }

    #[test]
    fn moving_a_card_container_moves_its_fully_enclosed_descendants() {
        let mut document = WorkspaceDocument::sample();
        let lens = document
            .lens_mut("current", "organization-landscape")
            .unwrap();
        lens.placements = vec![
            ConceptPlacement {
                concept_id: 4,
                bounds: Rect::new(Point::new(0.0, 0.0), Point::new(500.0, 360.0)),
            },
            ConceptPlacement {
                concept_id: 5,
                bounds: Rect::new(Point::new(80.0, 80.0), Point::new(240.0, 180.0)),
            },
            ConceptPlacement {
                concept_id: 7,
                bounds: Rect::new(Point::new(120.0, 120.0), Point::new(80.0, 60.0)),
            },
            ConceptPlacement {
                concept_id: 8,
                bounds: Rect::new(Point::new(440.0, 80.0), Point::new(120.0, 80.0)),
            },
        ];
        lens.visual_boundaries = vec![VisualBoundary {
            layer_id: None,
            id: "boundary-release-frame".to_string(),
            title: "Release frame".to_string(),
            summary: String::new(),
            bounds: Rect::new(Point::new(260.0, 100.0), Point::new(140.0, 100.0)),
        }];
        let world_delta = Point::new(32.0, -24.0);

        document
            .move_concepts_in_lens("current", "organization-landscape", &[4], world_delta)
            .unwrap();

        let lens = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        assert_eq!(lens.placements[0].bounds.origin, Point::new(32.0, -24.0));
        assert_eq!(lens.placements[1].bounds.origin, Point::new(112.0, 56.0));
        assert_eq!(lens.placements[2].bounds.origin, Point::new(152.0, 96.0));
        assert_eq!(lens.placements[3].bounds.origin, Point::new(440.0, 80.0));
        assert_eq!(
            lens.visual_boundaries[0].bounds.origin,
            Point::new(292.0, 76.0)
        );
    }

    #[test]
    fn invalid_multiple_placement_moves_leave_the_document_unchanged() {
        let mut document = WorkspaceDocument::sample();
        let original = document.clone();

        assert!(matches!(
            document.move_concepts_in_lens(
                "current",
                "organization-landscape",
                &[5, 999],
                Point::new(160.0, -40.0),
            ),
            Err(WorkspaceError::MissingPlacement {
                concept_id: 999,
                ..
            })
        ));
        assert_eq!(document, original);

        assert!(matches!(
            document.move_concepts_in_lens(
                "current",
                "organization-landscape",
                &[],
                Point::new(160.0, -40.0),
            ),
            Err(WorkspaceError::Validation(_))
        ));
        assert_eq!(document, original);

        assert!(matches!(
            document.move_concepts_in_lens(
                "current",
                "organization-landscape",
                &[5, 5],
                Point::new(160.0, -40.0),
            ),
            Err(WorkspaceError::Validation(_))
        ));
        assert_eq!(document, original);
    }

    #[test]
    fn lens_filter_limits_visible_concepts_and_relationships() {
        let mut document = WorkspaceDocument::sample();
        document.current_state.lenses[0].filter = LensFilter {
            concept_type_ids: vec!["process".to_string(), "repository".to_string()],
            relationship_kinds: vec![RelationshipKind::ImplementedIn],
        };

        let dataset = document
            .canvas_dataset("current", "organization-landscape")
            .unwrap();

        assert_eq!(dataset.card_count(), 2);
        assert_eq!(dataset.relationship_count(), 1);
    }

    #[test]
    fn selected_relationship_overlay_distinguishes_target_availability() {
        let mut document = WorkspaceDocument::sample();
        let version = &mut document.current_state;
        version.concepts.push(concept(
            10,
            "process",
            "Off-screen process",
            "Placed outside the test viewport",
        ));
        version.concepts.push(concept(
            11,
            "service",
            "Unplaced service",
            "Not placed in the active Lens",
        ));
        version.relationships = vec![
            relationship("visible", RelationshipKind::DependsOn, 7, 5),
            relationship("offscreen", RelationshipKind::DependsOn, 7, 10),
            relationship("filtered", RelationshipKind::DependsOn, 7, 8),
            relationship("unplaced", RelationshipKind::DependsOn, 7, 11),
            relationship("absent", RelationshipKind::DependsOn, 7, 99),
        ];
        let lens = &mut version.lenses[0];
        lens.filter = LensFilter {
            concept_type_ids: vec!["process".to_string(), "service".to_string()],
            relationship_kinds: Vec::new(),
        };
        lens.placements = vec![
            placement(7, 0.0, 0.0),
            placement(5, 400.0, 0.0),
            placement(8, 700.0, 0.0),
            placement(10, 1_000.0, 0.0),
        ];

        let overlay = document
            .selected_concept_relationship_overlay(
                "current",
                "organization-landscape",
                7,
                Rect::new(Point::new(0.0, 0.0), Point::new(800.0, 200.0)),
            )
            .unwrap();

        assert_eq!(
            overlay.selected_concept,
            SelectedConceptOverlaySource::Visible {
                bounds: placement(7, 0.0, 0.0).bounds,
            }
        );
        assert_eq!(
            overlay
                .relationships
                .iter()
                .map(|relationship| relationship.relationship_id.as_str())
                .collect::<Vec<_>>(),
            vec!["absent", "filtered", "offscreen", "unplaced", "visible"]
        );
        assert!(matches!(
            overlay.relationships[0].target,
            RelationshipOverlayTarget::Absent
        ));
        assert!(matches!(
            overlay.relationships[1].target,
            RelationshipOverlayTarget::Filtered
        ));
        assert!(matches!(
            overlay.relationships[2].target,
            RelationshipOverlayTarget::Offscreen
        ));
        assert!(matches!(
            overlay.relationships[3].target,
            RelationshipOverlayTarget::Unplaced
        ));
        assert!(matches!(
            overlay.relationships[4].target,
            RelationshipOverlayTarget::Visible { .. }
        ));

        let lines = overlay.visible_lines();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].relationship_id, "visible");
        assert_eq!(lines[0].geometry.source, Point::new(120.0, 66.0));
        assert_eq!(lines[0].geometry.target, Point::new(520.0, 66.0));
    }

    #[test]
    fn selected_relationship_paths_attach_to_card_edges_for_a_direct_route() {
        let mut document = WorkspaceDocument::sample();
        let version = &mut document.current_state;
        version.relationships = vec![relationship("visible", RelationshipKind::DependsOn, 7, 5)];
        version.lenses[0].placements = vec![placement(7, 0.0, 0.0), placement(5, 400.0, 0.0)];

        let paths = document
            .selected_concept_relationship_paths(
                "current",
                "organization-landscape",
                7,
                Rect::new(Point::new(0.0, 0.0), Point::new(800.0, 200.0)),
            )
            .unwrap();

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].relationship_id, "visible");
        assert_eq!(
            paths[0].points,
            vec![Point::new(240.0, 66.0), Point::new(400.0, 66.0)]
        );
    }

    #[test]
    fn selected_relationship_paths_route_around_visible_unrelated_cards() {
        let mut document = WorkspaceDocument::sample();
        let version = &mut document.current_state;
        version.relationships = vec![relationship("visible", RelationshipKind::DependsOn, 7, 5)];
        version.lenses[0].placements = vec![
            placement(7, 0.0, 0.0),
            placement(4, 360.0, 0.0),
            placement(5, 600.0, 0.0),
        ];
        let viewport = Rect::new(Point::new(0.0, -200.0), Point::new(1_200.0, 600.0));

        let paths = document
            .selected_concept_relationship_paths("current", "organization-landscape", 7, viewport)
            .unwrap();

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].points.len(), 4);
        assert_eq!(paths[0].points[1], Point::new(344.0, -16.0));
        assert_eq!(paths[0].points[2], Point::new(616.0, -16.0));
        assert!(paths[0].points[0].x == 240.0);
        assert!(paths[0].points[3].y == 0.0);
        let obstacle = placement(4, 360.0, 0.0).bounds;
        assert!(
            paths[0]
                .points
                .windows(2)
                .all(|segment| !super::segment_crosses_rect_interior(
                    segment[0], segment[1], obstacle
                ))
        );
        assert_eq!(
            paths,
            document
                .selected_concept_relationship_paths(
                    "current",
                    "organization-landscape",
                    7,
                    viewport,
                )
                .unwrap()
        );
    }

    #[test]
    fn selected_relationship_paths_treat_another_relationship_target_as_an_obstacle() {
        let mut document = WorkspaceDocument::sample();
        let version = &mut document.current_state;
        version.relationships = vec![
            relationship("platform", RelationshipKind::DependsOn, 7, 4),
            relationship("project", RelationshipKind::DependsOn, 7, 5),
        ];
        version.lenses[0].placements = vec![
            placement(7, 0.0, 0.0),
            placement(4, 360.0, 0.0),
            placement(5, 600.0, 0.0),
        ];
        let viewport = Rect::new(Point::new(0.0, -200.0), Point::new(1_200.0, 600.0));

        let paths = document
            .selected_concept_relationship_paths("current", "organization-landscape", 7, viewport)
            .unwrap();
        let project_path = paths
            .iter()
            .find(|path| path.relationship_id == "project")
            .unwrap();
        let platform_bounds = placement(4, 360.0, 0.0).bounds;

        assert_eq!(project_path.points.len(), 4);
        assert!(project_path.points.windows(2).all(|segment| {
            !super::segment_crosses_rect_interior(segment[0], segment[1], platform_bounds)
        }));
    }

    #[test]
    fn selected_relationship_paths_exclude_unavailable_endpoints() {
        let mut document = WorkspaceDocument::sample();
        let version = &mut document.current_state;
        version.relationships = vec![relationship("filtered", RelationshipKind::DependsOn, 7, 8)];
        version.lenses[0].filter = LensFilter {
            concept_type_ids: vec!["process".to_string()],
            relationship_kinds: Vec::new(),
        };
        version.lenses[0].placements = vec![placement(7, 0.0, 0.0), placement(8, 400.0, 0.0)];
        let viewport = Rect::new(Point::new(0.0, 0.0), Point::new(800.0, 200.0));

        let overlay = document
            .selected_concept_relationship_overlay("current", "organization-landscape", 7, viewport)
            .unwrap();

        assert!(matches!(
            overlay.relationships[0].target,
            RelationshipOverlayTarget::Filtered
        ));
        assert!(
            document
                .selected_concept_relationship_paths(
                    "current",
                    "organization-landscape",
                    7,
                    viewport
                )
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn transient_placement_overrides_recalculate_source_target_and_obstacle_paths() {
        let mut document = WorkspaceDocument::sample();
        let version = &mut document.current_state;
        version.relationships = vec![relationship("visible", RelationshipKind::DependsOn, 7, 5)];
        version.lenses[0].placements = vec![
            placement(7, 0.0, 0.0),
            placement(4, 360.0, 0.0),
            placement(5, 600.0, 0.0),
        ];
        let original = document.clone();
        let viewport = Rect::new(Point::new(0.0, -200.0), Point::new(1_200.0, 800.0));
        let stored_paths = document
            .selected_concept_relationship_paths("current", "organization-landscape", 7, viewport)
            .unwrap();
        let mut overrides = RelationshipOverlayPlacementOverrides::with_bounds(
            7,
            Rect::new(Point::new(100.0, 200.0), Point::new(240.0, 132.0)),
        );
        overrides.insert(
            5,
            Rect::new(Point::new(500.0, 200.0), Point::new(240.0, 132.0)),
        );
        overrides.insert(
            4,
            Rect::new(Point::new(360.0, 400.0), Point::new(240.0, 132.0)),
        );

        let overlay = document
            .selected_concept_relationship_overlay_with_placement_overrides(
                "current",
                "organization-landscape",
                7,
                viewport,
                &overrides,
            )
            .unwrap();
        let preview_paths = document
            .selected_concept_relationship_paths_with_placement_overrides(
                "current",
                "organization-landscape",
                7,
                viewport,
                &overrides,
            )
            .unwrap();

        assert_eq!(
            overlay.selected_concept,
            SelectedConceptOverlaySource::Visible {
                bounds: Rect::new(Point::new(100.0, 200.0), Point::new(240.0, 132.0)),
            }
        );
        assert_ne!(preview_paths, stored_paths);
        assert_eq!(stored_paths[0].points.len(), 4);
        assert_eq!(
            preview_paths[0].points,
            vec![Point::new(340.0, 266.0), Point::new(500.0, 266.0)]
        );
        assert_eq!(document, original);
    }

    #[test]
    fn selected_relationship_overlay_does_not_make_geometry_for_an_offscreen_source() {
        let document = WorkspaceDocument::sample();

        let overlay = document
            .selected_concept_relationship_overlay(
                "current",
                "organization-landscape",
                7,
                Rect::new(Point::new(1_000.0, 200.0), Point::new(600.0, 600.0)),
            )
            .unwrap();

        assert_eq!(
            overlay.selected_concept,
            SelectedConceptOverlaySource::Offscreen
        );
        assert!(overlay.visible_lines().is_empty());
        assert!(
            document
                .selected_concept_relationship_paths(
                    "current",
                    "organization-landscape",
                    7,
                    Rect::new(Point::new(1_000.0, 200.0), Point::new(600.0, 600.0)),
                )
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn invalid_custom_type_color_is_rejected() {
        let mut document = WorkspaceDocument::sample();
        document.custom_types[0].color = "purple".to_string();

        assert!(matches!(
            document.validate(),
            Err(WorkspaceError::Validation(message)) if message.contains("invalid color")
        ));
    }

    #[test]
    fn typed_relationship_creation_is_stable() {
        let mut document = WorkspaceDocument::sample();

        let relationship = document
            .link_concepts("current", 2, 7, RelationshipKind::Operates)
            .unwrap();

        assert_eq!(relationship.id, "rel-2-7-operates");
        assert_eq!(relationship.label, "operates");
    }

    #[test]
    fn relationship_creation_rejects_cards_in_the_same_containment_chain() {
        let mut document = WorkspaceDocument::sample();
        let lens = document
            .current_state
            .lenses
            .iter_mut()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        lens.placements
            .iter_mut()
            .find(|placement| placement.concept_id == 4)
            .unwrap()
            .bounds = Rect::new(Point::new(0.0, 0.0), Point::new(640.0, 480.0));
        lens.placements
            .iter_mut()
            .find(|placement| placement.concept_id == 8)
            .unwrap()
            .bounds = Rect::new(Point::new(120.0, 120.0), Point::new(240.0, 132.0));
        let relationship_count = document.current_state.relationships.len();

        for (source_id, target_id) in [(8, 4), (4, 8)] {
            assert!(matches!(
                document.link_concepts("current", source_id, target_id, RelationshipKind::DependsOn),
                Err(WorkspaceError::Validation(message)) if message.contains("containment chain")
            ));
        }
        assert_eq!(
            document.current_state.relationships.len(),
            relationship_count
        );
    }

    #[test]
    fn relationship_kinds_advertise_only_supported_endpoint_types() {
        assert!(RelationshipKind::Owns.supports_endpoints("team", "project"));
        assert!(!RelationshipKind::Owns.supports_endpoints("service", "project"));
        assert!(RelationshipKind::Operates.supports_endpoints("team", "environment"));
        assert!(!RelationshipKind::Operates.supports_endpoints("process", "environment"));
        assert!(RelationshipKind::ImplementedIn.supports_endpoints("process", "repository"));
        assert!(!RelationshipKind::ImplementedIn.supports_endpoints("process", "service"));
        assert!(RelationshipKind::DependsOn.supports_endpoints("service", "repository"));
    }

    #[test]
    fn concept_and_direct_relationships_are_retrieved_from_one_version() {
        let document = WorkspaceDocument::sample();

        let concept = document.concept("current", 5).unwrap();
        let relationship_ids: Vec<_> = document
            .relationships_for_concept("current", 5)
            .unwrap()
            .into_iter()
            .map(|relationship| relationship.id.as_str())
            .collect();

        assert_eq!(concept.title, "Portal API");
        assert_eq!(
            relationship_ids,
            vec![
                "rel-platform-api",
                "rel-portal-api",
                "rel-api-production",
                "rel-release-api",
            ]
        );
    }

    #[test]
    fn concept_queries_return_deterministic_missing_record_errors() {
        let document = WorkspaceDocument::sample();

        assert_eq!(
            document.concept("current", 99).unwrap_err(),
            WorkspaceError::MissingConcept {
                version_id: "current".to_string(),
                concept_id: 99,
            }
        );
        assert_eq!(
            document
                .relationships_for_concept("current", 99)
                .unwrap_err(),
            WorkspaceError::MissingConcept {
                version_id: "current".to_string(),
                concept_id: 99,
            }
        );
    }

    #[test]
    fn removing_a_relationship_returns_the_record_and_keeps_the_workspace_valid() {
        let mut document = WorkspaceDocument::sample();

        let removed = document
            .remove_relationship("current", "rel-release-api")
            .unwrap();

        assert_eq!(removed.kind, RelationshipKind::DependsOn);
        assert_eq!(removed.source_id, 7);
        assert_eq!(removed.target_id, 5);
        assert!(document.validate().is_ok());
        assert_eq!(
            document
                .relationships_for_concept("current", 7)
                .unwrap()
                .into_iter()
                .map(|relationship| relationship.id.as_str())
                .collect::<Vec<_>>(),
            vec!["rel-release-repository", "rel-release-access"]
        );
    }

    #[test]
    fn removing_a_relationship_removes_its_semantic_layer_reference() {
        let mut document = WorkspaceDocument::sample();
        let layer_id = document
            .create_semantic_zoom_layer(
                "current",
                "organization-landscape",
                "Delivery detail".to_string(),
            )
            .unwrap();
        document
            .set_semantic_zoom_layer_relationships(
                "current",
                "organization-landscape",
                &layer_id,
                &["rel-release-api".to_string()],
                true,
            )
            .unwrap();

        document
            .remove_relationship("current", "rel-release-api")
            .unwrap();

        let layer = document
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers
            .iter()
            .find(|layer| layer.id == layer_id)
            .unwrap();
        assert!(layer.relationship_ids.is_empty());
        assert!(document.validate().is_ok());
    }

    #[test]
    fn removing_an_unknown_relationship_returns_a_deterministic_error_without_mutation() {
        let mut document = WorkspaceDocument::sample();
        let original = document.clone();

        assert_eq!(
            document
                .remove_relationship("current", "rel-does-not-exist")
                .unwrap_err(),
            WorkspaceError::MissingRelationship {
                version_id: "current".to_string(),
                relationship_id: "rel-does-not-exist".to_string(),
            }
        );
        assert_eq!(document, original);
    }

    #[test]
    fn changing_a_relationship_kind_updates_its_default_canvas_label() {
        let mut document = WorkspaceDocument::sample();
        let mut expected = document.clone();
        let relationship_index = expected
            .current_state
            .relationships
            .iter()
            .position(|relationship| relationship.id == "rel-release-api")
            .unwrap();
        expected.current_state.relationships[relationship_index].kind =
            RelationshipKind::DeployedTo;
        expected.current_state.relationships[relationship_index].label =
            RelationshipKind::DeployedTo.label().to_string();

        let changed = document
            .change_relationship_kind("current", "rel-release-api", RelationshipKind::DeployedTo)
            .unwrap();

        assert_eq!(
            changed,
            expected.current_state.relationships[relationship_index]
        );
        assert_eq!(document, expected);
        assert!(document.validate().is_ok());
    }

    #[test]
    fn changing_a_relationship_kind_keeps_a_custom_canvas_label() {
        let mut document = WorkspaceDocument::sample();
        let relationship = document
            .current_state
            .relationships
            .iter_mut()
            .find(|relationship| relationship.id == "rel-release-api")
            .unwrap();
        relationship.label = "Runs the portal API release".to_string();

        document
            .change_relationship_kind("current", "rel-release-api", RelationshipKind::DeployedTo)
            .unwrap();

        let relationship = document
            .version("current")
            .unwrap()
            .relationships
            .iter()
            .find(|relationship| relationship.id == "rel-release-api")
            .unwrap();
        assert_eq!(relationship.kind, RelationshipKind::DeployedTo);
        assert_eq!(relationship.label, "Runs the portal API release");
    }

    #[test]
    fn invalid_relationship_kind_change_returns_validation_error_without_mutation() {
        let mut document = WorkspaceDocument::sample();
        let original = document.clone();

        assert_eq!(
            document
                .change_relationship_kind("current", "rel-release-api", RelationshipKind::Owns)
                .unwrap_err(),
            WorkspaceError::Validation(
                "owns relationship rel-release-api in version current must have a team source"
                    .to_string(),
            )
        );
        assert_eq!(document, original);
    }

    #[test]
    fn changed_relationship_kind_persists_through_json_round_trip() {
        let mut document = WorkspaceDocument::sample();

        document
            .change_relationship_kind("current", "rel-release-api", RelationshipKind::DeployedTo)
            .unwrap();

        let persisted = WorkspaceDocument::parse_json(&document.to_pretty_json().unwrap()).unwrap();
        let relationship = persisted
            .version("current")
            .unwrap()
            .relationships
            .iter()
            .find(|relationship| relationship.id == "rel-release-api")
            .unwrap();

        assert_eq!(relationship.kind, RelationshipKind::DeployedTo);
        assert_eq!(relationship.label, "deployed to");
        assert_eq!(relationship.source_id, 7);
        assert_eq!(relationship.target_id, 5);
    }

    #[test]
    fn semantic_zoom_frame_uses_the_changed_default_relationship_label() {
        let mut document = WorkspaceDocument::sample();

        document
            .change_relationship_kind("current", "rel-release-api", RelationshipKind::DeployedTo)
            .unwrap();

        let frame = document
            .semantic_zoom_frame("current", "organization-landscape", 1.0)
            .unwrap();
        let relationship = frame
            .relationships
            .iter()
            .find(|relationship| relationship.id == "rel-release-api")
            .unwrap();

        assert_eq!(relationship.kind, RelationshipKind::DeployedTo);
        assert_eq!(relationship.label, "deployed to");
    }

    #[test]
    fn editing_a_concept_preserves_its_remaining_data_and_lens_placements() {
        let mut document = WorkspaceDocument::sample();
        let original_relationships = document.current_state.relationships.clone();
        let original_placements: Vec<_> = document
            .current_state
            .lenses
            .iter()
            .map(|lens| (lens.id.clone(), lens.placements.clone()))
            .collect();
        let original = document.concept("current", 5).unwrap().clone();

        let edited = document
            .edit_concept("current", 5, "repository", "portal-api source".to_string())
            .unwrap();

        assert_eq!(edited.id, original.id);
        assert_eq!(edited.type_id, "repository");
        assert_eq!(edited.title, "portal-api source");
        assert_eq!(edited.summary, original.summary);
        assert_eq!(edited.details, original.details);
        assert_eq!(edited.execution_mode, original.execution_mode);
        assert_eq!(edited.attributes, original.attributes);
        assert_eq!(document.current_state.relationships, original_relationships);
        assert_eq!(
            document
                .current_state
                .lenses
                .iter()
                .map(|lens| (lens.id.clone(), lens.placements.clone()))
                .collect::<Vec<_>>(),
            original_placements
        );
    }

    #[test]
    fn editing_a_concept_rejects_unknown_types_without_mutation() {
        let mut document = WorkspaceDocument::sample();
        let original = document.clone();

        assert!(matches!(
            document.edit_concept("current", 5, "not-a-type", "Changed".to_string()),
            Err(WorkspaceError::Validation(message)) if message.contains("uses unknown type not-a-type")
        ));
        assert_eq!(document, original);
    }

    #[test]
    fn editing_a_process_with_an_execution_mode_rejects_a_non_process_type_without_mutation() {
        let mut document = WorkspaceDocument::sample();
        document
            .current_state
            .concepts
            .iter_mut()
            .find(|concept| concept.id == 7)
            .unwrap()
            .execution_mode = Some(super::ExecutionMode::Automated);
        let original = document.clone();

        assert!(matches!(
            document.edit_concept("current", 7, "service", "Release service".to_string()),
            Err(WorkspaceError::Validation(message)) if message.contains("has execution_mode but is not a process")
        ));
        assert_eq!(document, original);
    }

    #[test]
    fn edited_concepts_persist_without_changing_other_lens_placements() {
        let mut document = WorkspaceDocument::sample();
        let other_lens_before = document.version("current").unwrap();
        let other_lens_before = other_lens_before
            .lenses
            .iter()
            .find(|lens| lens.id == "delivery-and-production")
            .unwrap()
            .placements
            .clone();

        document
            .edit_concept("current", 5, "repository", "portal-api source".to_string())
            .unwrap();
        let persisted = WorkspaceDocument::parse_json(&document.to_pretty_json().unwrap()).unwrap();
        let other_lens_after = persisted.version("current").unwrap();
        let other_lens_after = other_lens_after
            .lenses
            .iter()
            .find(|lens| lens.id == "delivery-and-production")
            .unwrap()
            .placements
            .clone();

        assert_eq!(
            persisted.concept("current", 5).unwrap().type_id,
            "repository"
        );
        assert_eq!(
            persisted.concept("current", 5).unwrap().title,
            "portal-api source"
        );
        assert_eq!(other_lens_after, other_lens_before);
    }

    #[test]
    fn changing_an_unknown_relationship_kind_returns_a_deterministic_error_without_mutation() {
        let mut document = WorkspaceDocument::sample();
        let original = document.clone();

        assert_eq!(
            document
                .change_relationship_kind(
                    "current",
                    "rel-does-not-exist",
                    RelationshipKind::DeployedTo,
                )
                .unwrap_err(),
            WorkspaceError::MissingRelationship {
                version_id: "current".to_string(),
                relationship_id: "rel-does-not-exist".to_string(),
            }
        );
        assert_eq!(document, original);
    }

    #[test]
    fn concept_deletion_preview_counts_every_incident_record_in_one_version() {
        let document = WorkspaceDocument::sample();

        assert_eq!(
            document.preview_concept_deletion("current", 5).unwrap(),
            super::ConceptDeletionPreview {
                concept_id: 5,
                title: "Portal API".to_string(),
                incident_relationship_count: 4,
                placement_count: 2,
            }
        );
    }

    #[test]
    fn deleting_a_concept_is_version_local_and_restore_is_lossless() {
        let mut document = WorkspaceDocument::sample();
        document
            .create_planning_scenario("future".to_string(), "Future State".to_string(), "current")
            .unwrap();
        let original = document.clone();

        let deletion = document.delete_concept("current", 5).unwrap();

        assert_eq!(deletion.concept.title, "Portal API");
        assert_eq!(deletion.relationships.len(), 4);
        assert_eq!(deletion.placements.len(), 2);
        assert!(matches!(
            document.concept("current", 5),
            Err(WorkspaceError::MissingConcept { .. })
        ));
        assert!(
            document
                .current_state
                .relationships
                .iter()
                .all(|relationship| relationship.source_id != 5 && relationship.target_id != 5)
        );
        assert!(document.current_state.lenses.iter().all(|lens| {
            lens.placements
                .iter()
                .all(|placement| placement.concept_id != 5)
        }));
        assert_eq!(document.concept("future", 5).unwrap().title, "Portal API");
        assert_eq!(
            document
                .relationships_for_concept("future", 5)
                .unwrap()
                .len(),
            4
        );

        document.restore_deleted_concept(&deletion).unwrap();

        assert_eq!(document, original);
    }

    #[test]
    fn deleting_a_concept_removes_its_card_layer_mode_and_restores_it_losslessly() {
        let mut document = WorkspaceDocument::sample();
        let layer_id = document
            .create_semantic_zoom_layer(
                "current",
                "organization-landscape",
                "Layered detail".to_string(),
            )
            .unwrap();
        document
            .set_semantic_zoom_card_layer_mode(
                "current",
                "organization-landscape",
                8,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![layer_id],
                },
            )
            .unwrap();
        let original = document.clone();

        let deletion = document.delete_concept("current", 8).unwrap();

        assert_eq!(deletion.card_layer_modes.len(), 1);
        assert!(document.current_state.lenses.iter().all(|lens| {
            lens.semantic_zoom_card_modes
                .iter()
                .all(|entry| entry.concept_id != 8)
        }));
        document.validate().unwrap();

        document.restore_deleted_concept(&deletion).unwrap();

        assert_eq!(document, original);
    }

    #[test]
    fn deleting_a_missing_concept_returns_a_deterministic_error_without_mutation() {
        let mut document = WorkspaceDocument::sample();
        let original = document.clone();

        assert_eq!(
            document
                .preview_concept_deletion("current", 99)
                .unwrap_err(),
            WorkspaceError::MissingConcept {
                version_id: "current".to_string(),
                concept_id: 99,
            }
        );
        assert_eq!(
            document.delete_concept("current", 99).unwrap_err(),
            WorkspaceError::MissingConcept {
                version_id: "current".to_string(),
                concept_id: 99,
            }
        );
        assert_eq!(document, original);
    }

    #[test]
    fn malformed_deletion_receipts_are_rejected_without_mutation() {
        let mut document = WorkspaceDocument::sample();
        let deletion = document.delete_concept("current", 5).unwrap();
        let mut malformed = deletion.clone();
        malformed.relationships[0].relationship.source_id = 1;
        malformed.relationships[0].relationship.target_id = 2;
        let deleted_document = document.clone();

        assert_eq!(
            document.restore_deleted_concept(&malformed).unwrap_err(),
            WorkspaceError::InvalidConceptDeletionReceipt {
                version_id: "current".to_string(),
                reason: "relationship does not reference the deleted concept".to_string(),
            }
        );
        assert_eq!(document, deleted_document);
    }

    #[test]
    fn responsibility_relationships_require_a_team_source() {
        let mut document = WorkspaceDocument::sample();
        document
            .current_state
            .relationships
            .push(super::relationship(
                "invalid-project-ownership",
                RelationshipKind::Owns,
                4,
                5,
            ));

        assert!(matches!(
            document.validate(),
            Err(WorkspaceError::Validation(message)) if message.contains("team source")
        ));
    }

    #[test]
    fn implemented_in_requires_a_process_and_repository() {
        let mut document = WorkspaceDocument::sample();
        document
            .current_state
            .relationships
            .push(super::relationship(
                "invalid-implementation",
                RelationshipKind::ImplementedIn,
                5,
                8,
            ));

        assert!(matches!(
            document.validate(),
            Err(WorkspaceError::Validation(message)) if message.contains("process to a repository")
        ));
    }
}

#[cfg(test)]
mod authored_layer_regressions {
    use super::*;

    fn add_layer(document: &mut WorkspaceDocument, title: &str, start: f32, end: f32) -> String {
        let id = document
            .create_semantic_zoom_layer("current", "organization-landscape", title.to_string())
            .unwrap();
        document
            .update_semantic_zoom_layer_percent_settings(
                "current",
                "organization-landscape",
                &id,
                title.to_string(),
                SemanticZoomLayerPercentSettings {
                    active_zoom_range: ZoomPercentRange {
                        start_percent: start,
                        end_percent: end,
                    },
                    fade_percent: 10.0,
                },
                1.0,
            )
            .unwrap();
        id
    }

    #[test]
    fn reorder_assigns_existing_range_slots_to_the_new_layer_order() {
        let mut document = WorkspaceDocument::sample();
        let first = add_layer(&mut document, "First", 100.0, 150.0);
        let second = add_layer(&mut document, "Second", 150.0, 200.0);
        document
            .reorder_semantic_zoom_layer("current", "organization-landscape", &second, 0)
            .unwrap();
        let lens = &document.current_state.lenses[0];
        assert_eq!(lens.semantic_zoom_layers[0].id, second);
        assert_eq!(lens.semantic_zoom_layers[1].id, first);
        let range = document
            .semantic_zoom_layer_percent_settings("current", "organization-landscape", &second)
            .unwrap()
            .active_zoom_range;
        assert_eq!(
            range,
            ZoomPercentRange {
                start_percent: 100.0,
                end_percent: 150.0
            }
        );
        assert_eq!(
            document
                .semantic_zoom_layer_percent_settings("current", "organization-landscape", &first)
                .unwrap()
                .active_zoom_range,
            ZoomPercentRange {
                start_percent: 150.0,
                end_percent: 200.0
            }
        );
    }

    #[test]
    fn reorder_handles_all_destinations_and_preserves_range_slots_and_layer_properties() {
        for ranges in [
            [(1.0, 24.0), (24.0, 85.0), (85.0, 200.0)],
            [(1.0, 24.0), (30.0, 85.0), (80.0, 409_600.0)],
        ] {
            let mut original = WorkspaceDocument::sample();
            for (index, (start, end)) in ranges.into_iter().enumerate() {
                let id = add_layer(&mut original, &format!("Layer {index}"), start, end);
                let layer = original.current_state.lenses[0]
                    .semantic_zoom_layers
                    .iter_mut()
                    .find(|layer| layer.id == id)
                    .unwrap();
                layer.card_scale = 0.5 + index as f32;
                layer.percent_settings.as_mut().unwrap().fade_percent = index as f32;
            }
            let before = original.current_state.lenses[0]
                .semantic_zoom_layers
                .clone();
            for source in 0..3 {
                for destination in 0..3 {
                    let mut document = original.clone();
                    document
                        .reorder_semantic_zoom_layer(
                            "current",
                            "organization-landscape",
                            &before[source].id,
                            destination,
                        )
                        .unwrap();
                    let mut expected = before.clone();
                    let moved = expected.remove(source);
                    expected.insert(destination, moved);
                    for (layer, slot) in expected.iter_mut().zip(&before) {
                        layer.percent_settings.as_mut().unwrap().active_zoom_range =
                            slot.percent_settings.unwrap().active_zoom_range;
                    }
                    assert_eq!(
                        document.current_state.lenses[0].semantic_zoom_layers,
                        expected
                    );
                    let saved = document.to_pretty_json().unwrap();
                    assert_eq!(WorkspaceDocument::parse_json(&saved).unwrap(), document);
                    document
                        .reorder_semantic_zoom_layer(
                            "current",
                            "organization-landscape",
                            &before[source].id,
                            source,
                        )
                        .unwrap();
                    assert_eq!(document, original);
                }
            }
            let unchanged = original.clone();
            assert!(
                original
                    .reorder_semantic_zoom_layer(
                        "current",
                        "organization-landscape",
                        &before[0].id,
                        3,
                    )
                    .is_err()
            );
            assert_eq!(original, unchanged);
        }
    }

    #[test]
    fn explicit_references_share_all_content_across_versions_and_copies_stay_independent() {
        let mut document = WorkspaceDocument::sample();
        let source = document.clone();
        let source_lens = "organization-landscape";
        let version = document
            .create_version_from("current", "Linked destination".into())
            .unwrap();
        let original_placement = document.current_state.lenses[0].placements.clone();
        let reference = document
            .paste_concepts_from_lens(
                &source,
                "current",
                source_lens,
                &version,
                source_lens,
                &[5],
                Point::new(1000.0, 800.0),
                CardPasteMode::Reference,
            )
            .unwrap()[0];
        assert_ne!(
            reference, 5,
            "an independently cloned card must not become a reference"
        );
        document
            .edit_concept(&version, reference, "service", "Linked title".into())
            .unwrap();
        assert_eq!(
            document.concept("current", 5).unwrap().title,
            "Linked title"
        );
        assert_eq!(
            document.concept(&version, 5).unwrap().title,
            source.concept("current", 5).unwrap().title
        );
        let card = document
            .version_mut(&version)
            .unwrap()
            .concepts
            .iter_mut()
            .find(|card| card.id == reference)
            .unwrap();
        card.summary = "Shared summary".into();
        card.details = "Shared details".into();
        card.attributes.insert("status".into(), "ready".into());
        document
            .synchronize_concept_content(&version, reference)
            .unwrap();
        let view = document
            .create_empty_lens("current", "Linked view".into())
            .unwrap();
        let view_reference = document
            .paste_concepts_from_lens(
                &source,
                "current",
                source_lens,
                "current",
                &view,
                &[5],
                Point::default(),
                CardPasteMode::Reference,
            )
            .unwrap()[0];
        assert_eq!(view_reference, 5);
        let copy = document
            .paste_concepts_from_lens(
                &source,
                "current",
                source_lens,
                "current",
                &view,
                &[5],
                Point::new(400.0, 0.0),
                CardPasteMode::Copy,
            )
            .unwrap()[0];
        assert_ne!(copy, 5);
        assert!(
            document
                .concept("current", copy)
                .unwrap()
                .content_reference
                .is_none()
        );
        document
            .edit_concept("current", copy, "service", "Independent title".into())
            .unwrap();
        assert_eq!(
            document.concept("current", 5).unwrap().title,
            "Linked title"
        );
        assert_eq!(
            document.current_state.lenses[0].placements,
            original_placement
        );
        assert_eq!(
            document.current_state.relationships,
            source.current_state.relationships
        );
        let restored = WorkspaceDocument::parse_json(&document.to_pretty_json().unwrap()).unwrap();
        assert_eq!(restored, document);
        let mut shared = document.concept(&version, reference).unwrap().clone();
        shared.id = 5;
        assert_eq!(&shared, document.concept("current", 5).unwrap());
        let unchanged = document.clone();
        assert!(
            document
                .paste_concepts_from_lens(
                    &source,
                    "current",
                    source_lens,
                    "current",
                    &view,
                    &[5],
                    Point::default(),
                    CardPasteMode::Reference
                )
                .is_err()
        );
        assert_eq!(document, unchanged);
        let clone = document
            .create_version_from("current", "Independent snapshot".into())
            .unwrap();
        assert!(
            document
                .concept(&clone, 5)
                .unwrap()
                .content_reference
                .is_none()
        );
        document.delete_concept("current", 5).unwrap();
        assert_eq!(
            document.concept(&version, reference).unwrap().title,
            "Linked title"
        );
        document.validate().unwrap();
    }

    #[test]
    fn schema_six_migrates_without_creating_links_and_invalid_shared_content_is_rejected() {
        let mut legacy = serde_json::to_value(WorkspaceDocument::sample()).unwrap();
        legacy["schema_version"] = 6.into();
        let document = WorkspaceDocument::parse_json(&legacy.to_string()).unwrap();
        assert_eq!(document.schema_version, 7);
        assert!(
            document
                .current_state
                .concepts
                .iter()
                .all(|card| card.content_reference.is_none())
        );
        let mut invalid = document.clone();
        invalid.current_state.concepts[0].content_reference = Some("same-content".into());
        invalid.current_state.concepts[1].content_reference = Some("same-content".into());
        assert!(invalid.validate().is_err());
        assert!(WorkspaceDocument::parse_json(&serde_json::to_string(&invalid).unwrap()).is_err());
    }

    #[test]
    fn insertion_at_a_chosen_slot_shifts_later_ranges_without_changing_widths() {
        let mut document = WorkspaceDocument::sample();
        let first = add_layer(&mut document, "First", 50.0, 100.0);
        let second = add_layer(&mut document, "Second", 100.0, 150.0);
        let inserted = add_layer(&mut document, "Inserted", 100.0, 125.0);
        document
            .insert_semantic_zoom_layer("current", "organization-landscape", &inserted, 1)
            .unwrap();
        let lens = &document.current_state.lenses[0];
        assert_eq!(
            lens.semantic_zoom_layers
                .iter()
                .map(|layer| layer.id.as_str())
                .collect::<Vec<_>>(),
            vec![first.as_str(), inserted.as_str(), second.as_str()]
        );
        assert_eq!(
            document
                .semantic_zoom_layer_percent_settings("current", "organization-landscape", &second)
                .unwrap()
                .active_zoom_range,
            ZoomPercentRange {
                start_percent: 125.0,
                end_percent: 175.0
            }
        );
    }

    #[test]
    fn boundary_layer_changes_validate_roundtrip_and_clear_when_layer_deleted() {
        let mut document = WorkspaceDocument::sample();
        let layer = add_layer(&mut document, "Layer", 50.0, 100.0);
        let boundary = document
            .create_visual_boundary(
                "current",
                "organization-landscape",
                "Area".to_string(),
                String::new(),
                Rect::new(Point::new(0.0, 0.0), Point::new(800.0, 500.0)),
            )
            .unwrap();
        document
            .set_visual_boundary_layer(
                "current",
                "organization-landscape",
                &boundary,
                Some(layer.clone()),
            )
            .unwrap();
        let saved = serde_json::to_string(&document).unwrap();
        assert_eq!(
            serde_json::from_str::<WorkspaceDocument>(&saved).unwrap(),
            document
        );
        let unchanged = document.clone();
        assert!(
            document
                .set_visual_boundary_layer(
                    "current",
                    "organization-landscape",
                    &boundary,
                    Some("missing".to_string())
                )
                .is_err()
        );
        assert_eq!(document, unchanged);
        document
            .delete_semantic_zoom_layer("current", "organization-landscape", &layer)
            .unwrap();
        assert!(
            document.current_state.lenses[0]
                .visual_boundaries
                .last()
                .unwrap()
                .layer_id
                .is_none()
        );
    }

    #[test]
    fn moving_to_an_adjacent_layer_resizes_about_center_and_retains_aspect_ratio() {
        let mut document = WorkspaceDocument::sample();
        let first = add_layer(&mut document, "First", 50.0, 100.0);
        let second = add_layer(&mut document, "Second", 100.0, 150.0);
        document
            .set_semantic_zoom_card_layer_mode(
                "current",
                "organization-landscape",
                1,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![first],
                },
            )
            .unwrap();
        let before = document.current_state.lenses[0]
            .placements
            .iter()
            .find(|placement| placement.concept_id == 1)
            .unwrap()
            .bounds;
        document
            .shift_concept_layer("current", "organization-landscape", 1, true)
            .unwrap();
        let after = document.current_state.lenses[0]
            .placements
            .iter()
            .find(|placement| placement.concept_id == 1)
            .unwrap()
            .bounds;
        assert!((before.center().x - after.center().x).abs() < 0.001);
        assert!((before.center().y - after.center().y).abs() < 0.001);
        assert!((before.size.x / before.size.y - after.size.x / after.size.y).abs() < 0.001);
        assert_ne!(before.size, after.size);
        assert_eq!(
            document
                .semantic_zoom_card_layer_visibility("current", "organization-landscape", 1)
                .unwrap()
                .base_layer_id,
            Some(second)
        );
    }

    #[test]
    fn rollup_uses_visible_ancestor_without_forcing_hidden_child_and_preserves_fade() {
        let mut document = WorkspaceDocument::sample();
        let overview = add_layer(&mut document, "Overview", 50.0, 100.0);
        let detail = add_layer(&mut document, "Detail", 150.0, 200.0);
        let lens = &mut document.current_state.lenses[0];
        lens.placements = vec![
            ConceptPlacement {
                concept_id: 1,
                bounds: Rect::new(Point::new(0.0, 0.0), Point::new(1000.0, 1000.0)),
            },
            ConceptPlacement {
                concept_id: 2,
                bounds: Rect::new(Point::new(100.0, 100.0), Point::new(500.0, 500.0)),
            },
            ConceptPlacement {
                concept_id: 3,
                bounds: Rect::new(Point::new(200.0, 200.0), Point::new(100.0, 100.0)),
            },
            ConceptPlacement {
                concept_id: 4,
                bounds: Rect::new(Point::new(1200.0, 0.0), Point::new(200.0, 200.0)),
            },
        ];
        document.current_state.relationships = vec![TypedRelationship {
            id: "rollup".to_string(),
            kind: RelationshipKind::DependsOn,
            label: "Depends on".to_string(),
            source_id: 3,
            target_id: 4,
        }];
        for id in [1, 4] {
            document
                .set_semantic_zoom_card_layer_mode(
                    "current",
                    "organization-landscape",
                    id,
                    SemanticZoomCardLayerMode::Manual {
                        layer_ids: vec![overview.clone()],
                    },
                )
                .unwrap();
        }
        for id in [2, 3] {
            document
                .set_semantic_zoom_card_layer_mode(
                    "current",
                    "organization-landscape",
                    id,
                    SemanticZoomCardLayerMode::Manual {
                        layer_ids: vec![detail.clone()],
                    },
                )
                .unwrap();
        }
        let frame = document
            .semantic_zoom_frame_with_relationship_rollup(
                "current",
                "organization-landscape",
                0.75,
                true,
            )
            .unwrap();
        assert!(!frame.cards.iter().any(|card| card.card.id == 3));
        assert_eq!(frame.relationships[0].source_id, 1);
        assert_eq!(frame.relationships[0].target_id, 4);
        let unchanged = document.clone();
        let normal = document
            .semantic_zoom_frame("current", "organization-landscape", 0.75)
            .unwrap();
        assert_eq!(
            normal.cards, frame.cards,
            "roll-up must not alter card visibility"
        );
        assert!(
            normal.relationships.is_empty(),
            "Off disables the hidden-endpoint fallback"
        );
        assert_eq!(document, unchanged);
        // A child fading in is retained at its own opacity, even when the
        // relationship has a fully visible endpoint in another layer.
        let frame = document
            .semantic_zoom_frame_with_relationship_rollup(
                "current",
                "organization-landscape",
                1.45,
                true,
            )
            .unwrap();
        assert!(
            (frame
                .cards
                .iter()
                .find(|card| card.card.id == 3)
                .unwrap()
                .opacity
                - 0.5)
                .abs()
                < 0.001
        );
        document.current_state.relationships[0].target_id = 2;
        let collapsed = document
            .semantic_zoom_frame_with_relationship_rollup(
                "current",
                "organization-landscape",
                0.75,
                true,
            )
            .unwrap();
        assert!(
            collapsed.relationships.is_empty(),
            "A rolled edge inside one parent should not become a self-loop"
        );
        document.current_state.relationships[0].target_id = 4;
        document
            .set_semantic_zoom_card_layer_mode(
                "current",
                "organization-landscape",
                1,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![detail],
                },
            )
            .unwrap();
        let unavailable = document
            .semantic_zoom_frame_with_relationship_rollup(
                "current",
                "organization-landscape",
                0.75,
                true,
            )
            .unwrap();
        assert!(
            unavailable.relationships.is_empty(),
            "No visible representative means no dangling relationship"
        );
    }
}
