//! UI-free staged actions for the interim workspace evaluation workflows.

use std::collections::{BTreeMap, BTreeSet};

use infinite_note_canvas_canvas::{
    CanvasItemId, Point, Rect, RelationshipKind, SemanticZoomCardLayerMode,
    SemanticZoomLayerPercentSettings, SemanticZoomLayerSettings, WorkspaceConcept,
    WorkspaceDocument, WorkspaceError,
};

use crate::{StagedWorkspaceMutation, WorkspaceSession, WorkspaceStageError};

/// The required data for a new concept placed in the active Lens.
#[derive(Clone, Debug, PartialEq)]
pub struct ConceptDraft {
    pub type_id: String,
    pub title: String,
    pub summary: String,
    pub bounds: Rect,
}

/// The complete editable concept data supplied by a concept inspector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConceptInspectorEditDraft {
    pub concept_id: u32,
    pub title: String,
    pub type_id: String,
    pub summary: String,
    pub details: String,
    pub assertion_attributes: BTreeMap<String, String>,
}

/// A workspace version available for selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionChoice {
    pub id: String,
    pub title: String,
}

/// A Lens available for selection within one workspace version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LensChoice {
    pub id: String,
    pub title: String,
}

/// A semantic zoom layer available for editing in the active Lens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticZoomLayerChoice {
    pub id: String,
    pub title: String,
}

/// A deterministic edit to one user-stated assertion attribute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssertionMutation {
    /// Adds or replaces an assertion value for a key.
    Set { key: String, value: String },
    /// Removes an assertion for a key when it exists.
    Remove { key: String },
}

impl WorkspaceSession {
    /// Lists workspace versions in their document order.
    pub fn available_versions(&self) -> Vec<VersionChoice> {
        self.document()
            .version_ids()
            .into_iter()
            .map(|id| VersionChoice {
                id: id.to_string(),
                title: self
                    .document()
                    .version_title(id)
                    .expect("resident workspace versions must be valid")
                    .to_string(),
            })
            .collect()
    }

    /// Lists the Lenses available in the requested workspace version.
    pub fn available_lenses(
        &self,
        version_id: &str,
    ) -> Result<Vec<LensChoice>, WorkspaceStageError> {
        let version = self
            .document()
            .version(version_id)
            .map_err(WorkspaceStageError::Mutation)?;

        Ok(version
            .lenses
            .iter()
            .map(|lens| LensChoice {
                id: lens.id.clone(),
                title: lens.title.clone(),
            })
            .collect())
    }

    /// Lists semantic zoom layers in their Lens order.
    pub fn available_semantic_zoom_layers(
        &self,
    ) -> Result<Vec<SemanticZoomLayerChoice>, WorkspaceStageError> {
        let lens = self
            .document()
            .version(self.active_version_id())
            .map_err(WorkspaceStageError::Mutation)?
            .lenses
            .iter()
            .find(|lens| lens.id == self.active_lens_id())
            .ok_or_else(|| {
                WorkspaceStageError::Mutation(WorkspaceError::MissingLens {
                    version_id: self.active_version_id().to_string(),
                    lens_id: self.active_lens_id().to_string(),
                })
            })?;
        Ok(lens
            .semantic_zoom_layers
            .iter()
            .map(|layer| SemanticZoomLayerChoice {
                id: layer.id.clone(),
                title: layer.title.clone(),
            })
            .collect())
    }

    /// Reads one layer's user-percent range and symmetric fade settings.
    pub fn semantic_zoom_layer_percent_settings(
        &self,
        layer_id: &str,
    ) -> Result<SemanticZoomLayerPercentSettings, WorkspaceStageError> {
        self.document()
            .semantic_zoom_layer_percent_settings(
                self.active_version_id(),
                self.active_lens_id(),
                layer_id,
            )
            .map_err(WorkspaceStageError::Mutation)
    }

    /// Stages creation of an empty semantic zoom layer in the active Lens.
    pub fn stage_create_semantic_zoom_layer(
        &self,
        title: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .create_semantic_zoom_layer(self.active_version_id(), self.active_lens_id(), title)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages creation of an empty layer with its initial user-percent range
    /// and symmetric fade. This keeps creation to one saved history entry.
    pub fn stage_create_semantic_zoom_layer_with_percent_settings(
        &self,
        title: String,
        percent_settings: SemanticZoomLayerPercentSettings,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let layer_id = document
            .create_semantic_zoom_layer(
                self.active_version_id(),
                self.active_lens_id(),
                title.clone(),
            )
            .map_err(WorkspaceStageError::Mutation)?;
        document
            .update_semantic_zoom_layer_percent_settings(
                self.active_version_id(),
                self.active_lens_id(),
                &layer_id,
                title,
                percent_settings,
                1.0,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    pub fn stage_create_semantic_zoom_layer_at(
        &self,
        title: String,
        settings: SemanticZoomLayerPercentSettings,
        index: usize,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let layer_id = document
            .create_semantic_zoom_layer(
                self.active_version_id(),
                self.active_lens_id(),
                title.clone(),
            )
            .map_err(WorkspaceStageError::Mutation)?;
        document
            .update_semantic_zoom_layer_percent_settings(
                self.active_version_id(),
                self.active_lens_id(),
                &layer_id,
                title,
                settings,
                1.0,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        document
            .insert_semantic_zoom_layer(
                self.active_version_id(),
                self.active_lens_id(),
                &layer_id,
                index,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Creates the first semantic zoom layer and assigns the active Lens's
    /// existing cards and relationships to it as their base level.
    pub fn stage_create_semantic_zoom_base_layer(
        &self,
        title: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let version = document
            .version(self.active_version_id())
            .map_err(WorkspaceStageError::Mutation)?;
        let lens = version
            .lenses
            .iter()
            .find(|lens| lens.id == self.active_lens_id())
            .ok_or_else(|| {
                WorkspaceStageError::Mutation(WorkspaceError::MissingLens {
                    version_id: self.active_version_id().to_string(),
                    lens_id: self.active_lens_id().to_string(),
                })
            })?;
        let concept_ids = lens
            .placements
            .iter()
            .map(|placement| placement.concept_id)
            .collect::<Vec<_>>();
        let relationship_ids = lens
            .relationships
            .as_deref()
            .unwrap_or(&version.relationships)
            .iter()
            .map(|relationship| relationship.id.clone())
            .collect::<Vec<_>>();
        let layer_id = document
            .create_semantic_zoom_layer(self.active_version_id(), self.active_lens_id(), title)
            .map_err(WorkspaceStageError::Mutation)?;
        document
            .set_semantic_zoom_layer_concepts(
                self.active_version_id(),
                self.active_lens_id(),
                &layer_id,
                &concept_ids,
                true,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        document
            .set_semantic_zoom_layer_relationships(
                self.active_version_id(),
                self.active_lens_id(),
                &layer_id,
                &relationship_ids,
                true,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages the first layer, assigns existing presentation records to it,
    /// and saves the requested user-percent range in one mutation.
    pub fn stage_create_semantic_zoom_base_layer_with_percent_settings(
        &self,
        title: String,
        percent_settings: SemanticZoomLayerPercentSettings,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let staged = self.stage_create_semantic_zoom_base_layer(title.clone())?;
        let mut document = staged.document().clone();
        let layer_id = document
            .version(self.active_version_id())
            .map_err(WorkspaceStageError::Mutation)?
            .lenses
            .iter()
            .find(|lens| lens.id == self.active_lens_id())
            .and_then(|lens| lens.semantic_zoom_layers.last())
            .map(|layer| layer.id.clone())
            .ok_or_else(|| {
                WorkspaceStageError::Mutation(WorkspaceError::MissingSemanticZoomLayer {
                    version_id: self.active_version_id().to_string(),
                    lens_id: self.active_lens_id().to_string(),
                    layer_id: title.clone(),
                })
            })?;
        document
            .update_semantic_zoom_layer_percent_settings(
                self.active_version_id(),
                self.active_lens_id(),
                &layer_id,
                title,
                percent_settings,
                1.0,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages an edit to one semantic zoom layer's presentation settings.
    pub fn stage_update_semantic_zoom_layer(
        &self,
        layer_id: &str,
        settings: SemanticZoomLayerSettings,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .update_semantic_zoom_layer(
                self.active_version_id(),
                self.active_lens_id(),
                layer_id,
                settings,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a user-percent active range and symmetric fade for one layer.
    pub fn stage_update_semantic_zoom_layer_percent_settings(
        &self,
        layer_id: &str,
        title: String,
        percent_settings: SemanticZoomLayerPercentSettings,
        card_scale: f32,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .update_semantic_zoom_layer_percent_settings(
                self.active_version_id(),
                self.active_lens_id(),
                layer_id,
                title,
                percent_settings,
                card_scale,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a Lens-local semantic zoom layer reorder.
    pub fn stage_reorder_semantic_zoom_layer(
        &self,
        layer_id: &str,
        destination_index: usize,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .reorder_semantic_zoom_layer(
                self.active_version_id(),
                self.active_lens_id(),
                layer_id,
                destination_index,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages one Lens-local card presentation policy.
    pub fn stage_set_semantic_zoom_card_layer_mode(
        &self,
        concept_id: u32,
        mode: SemanticZoomCardLayerMode,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .set_semantic_zoom_card_layer_mode(
                self.active_version_id(),
                self.active_lens_id(),
                concept_id,
                mode,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages base-layer assignment or removal for concepts in the active Lens.
    pub fn stage_set_semantic_zoom_layer_concepts(
        &self,
        layer_id: &str,
        concept_ids: &[u32],
        included: bool,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .set_semantic_zoom_layer_concepts(
                self.active_version_id(),
                self.active_lens_id(),
                layer_id,
                concept_ids,
                included,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages base-layer assignment or removal for relationships in the active Lens.
    pub fn stage_set_semantic_zoom_layer_relationships(
        &self,
        layer_id: &str,
        relationship_ids: &[String],
        included: bool,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .set_semantic_zoom_layer_relationships(
                self.active_version_id(),
                self.active_lens_id(),
                layer_id,
                relationship_ids,
                included,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages deletion of one semantic zoom layer from the active Lens.
    pub fn stage_delete_semantic_zoom_layer(
        &self,
        layer_id: &str,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .delete_semantic_zoom_layer(self.active_version_id(), self.active_lens_id(), layer_id)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a move for a concept placement in the active version and Lens.
    pub fn stage_move_active_concept(
        &self,
        concept_id: u32,
        world_delta: Point,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .move_concept_in_lens(
                self.active_version_id(),
                self.active_lens_id(),
                concept_id,
                world_delta,
            )
            .map_err(WorkspaceStageError::Mutation)?;

        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a Lens-local card resize. It does not change the shared concept
    /// or relationships and participates in normal local undo and redo.
    pub fn stage_shift_concept_layer(
        &self,
        concept_id: u32,
        deeper: bool,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .shift_concept_layer(
                self.active_version_id(),
                self.active_lens_id(),
                concept_id,
                deeper,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, vec![concept_id])
    }

    pub fn stage_resize_active_concept(
        &self,
        concept_id: u32,
        bounds: Rect,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .resize_concept_in_lens(
                self.active_version_id(),
                self.active_lens_id(),
                concept_id,
                bounds,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, vec![concept_id])
    }

    /// Removes a card reference and incident relationships from the active view.
    /// Shared concept content and all other views remain intact.
    pub fn stage_remove_active_concept_reference(
        &self,
        concept_id: u32,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        self.stage_remove_active_concept_references(&[concept_id])
    }

    /// Removes several active-view references as one atomic workspace mutation.
    /// The resident session is unchanged if any requested reference is absent.
    pub fn stage_remove_active_concept_references(
        &self,
        concept_ids: &[u32],
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        for concept_id in concept_ids.iter().copied().collect::<BTreeSet<_>>() {
            document
                .remove_concept_from_lens(
                    self.active_version_id(),
                    self.active_lens_id(),
                    concept_id,
                )
                .map_err(WorkspaceStageError::Mutation)?;
        }
        self.stage_document_mutation(document, Vec::new())
    }

    /// Permanently removes concepts and their incident records from every view
    /// in the active version. External concept deletion retains this explicit contract.
    pub fn stage_delete_active_concept(
        &self,
        concept_id: u32,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        self.stage_delete_active_concepts(&[concept_id])
    }

    /// Permanently removes multiple concepts as a single history entry.
    pub fn stage_delete_active_concepts(
        &self,
        concept_ids: &[u32],
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        for concept_id in concept_ids.iter().copied().collect::<BTreeSet<_>>() {
            document
                .delete_concept(self.active_version_id(), concept_id)
                .map_err(WorkspaceStageError::Mutation)?;
        }
        self.stage_document_mutation(document, Vec::new())
    }

    /// Stages an atomic move for selected concept placements in the active version and Lens.
    pub fn stage_move_active_concepts(
        &self,
        selected_concept_ids: &[u32],
        world_delta: Point,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .move_concepts_in_lens(
                self.active_version_id(),
                self.active_lens_id(),
                selected_concept_ids,
                world_delta,
            )
            .map_err(WorkspaceStageError::Mutation)?;

        self.stage_document_mutation(document, selected_concept_ids.to_vec())
    }

    /// Stages an atomic move for cards and Lens-local boundary frames. The
    /// document resolves every fully enclosed descendant through one shared
    /// canvas-containment rule.
    pub fn stage_move_active_canvas_items(
        &self,
        item_ids: &[CanvasItemId],
        world_delta: Point,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .move_canvas_items_in_lens(
                self.active_version_id(),
                self.active_lens_id(),
                item_ids,
                world_delta,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        let selected_concept_ids = item_ids
            .iter()
            .filter_map(|item_id| match item_id {
                CanvasItemId::Card(concept_id) => Some(*concept_id),
                CanvasItemId::Boundary(_) => None,
            })
            .collect();
        self.stage_document_mutation(document, selected_concept_ids)
    }

    /// Stages a valid concept and its placement in the active version and Lens.
    pub fn stage_create_active_concept(
        &self,
        draft: ConceptDraft,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        self.stage_create_active_concept_with_layer_mode(draft, None)
    }

    /// Stages a valid concept and its placement with an optional Lens-local
    /// display policy. Supplying a policy keeps creation atomic: the new card
    /// never appears in an unintended zoom layer between mutations.
    pub fn stage_create_active_concept_with_layer_mode(
        &self,
        draft: ConceptDraft,
        layer_mode: Option<SemanticZoomCardLayerMode>,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let concept_id = document
            .add_concept_to_lens(
                self.active_version_id(),
                self.active_lens_id(),
                &draft.type_id,
                draft.title,
                draft.summary,
                draft.bounds,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        if let Some(layer_mode) = layer_mode {
            document
                .set_semantic_zoom_card_layer_mode(
                    self.active_version_id(),
                    self.active_lens_id(),
                    concept_id,
                    layer_mode,
                )
                .map_err(WorkspaceStageError::Mutation)?;
        }

        self.stage_document_mutation(document, vec![concept_id])
    }

    /// Stages a neutral visual boundary in the active Lens.
    pub fn stage_create_visual_boundary(
        &self,
        title: String,
        summary: String,
        bounds: Rect,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .create_visual_boundary(
                self.active_version_id(),
                self.active_lens_id(),
                title,
                summary,
                bounds,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a label or context change for one visual boundary in the active Lens.
    pub fn stage_update_visual_boundary(
        &self,
        boundary_id: &str,
        title: String,
        summary: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .update_visual_boundary(
                self.active_version_id(),
                self.active_lens_id(),
                boundary_id,
                title,
                summary,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    pub fn stage_set_visual_boundary_layer(
        &self,
        boundary_id: &str,
        layer_id: Option<String>,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .set_visual_boundary_layer(
                self.active_version_id(),
                self.active_lens_id(),
                boundary_id,
                layer_id,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a visual-boundary move in the active Lens. Fully enclosed cards
    /// and nested boundaries travel with it; graph relationships do not change.
    pub fn stage_move_visual_boundary(
        &self,
        boundary_id: &str,
        world_delta: Point,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .move_visual_boundary_with_contained_concepts(
                self.active_version_id(),
                self.active_lens_id(),
                boundary_id,
                world_delta,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a visual-boundary resize in the active Lens. It does not move
    /// cards or change any graph relationship.
    pub fn stage_resize_visual_boundary(
        &self,
        boundary_id: &str,
        bounds: Rect,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .resize_visual_boundary(
                self.active_version_id(),
                self.active_lens_id(),
                boundary_id,
                bounds,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages removal of one visual boundary from the active Lens.
    pub fn stage_remove_visual_boundary(
        &self,
        boundary_id: &str,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .remove_visual_boundary(self.active_version_id(), self.active_lens_id(), boundary_id)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a typed relationship in the active workspace version.
    pub fn stage_create_relationship(
        &self,
        source_id: u32,
        target_id: u32,
        relationship_kind: RelationshipKind,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .link_concepts_in_lens(
                self.active_version_id(),
                self.active_lens_id(),
                source_id,
                target_id,
                relationship_kind,
            )
            .map_err(WorkspaceStageError::Mutation)?;

        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a relationship-kind change in the active workspace version.
    pub fn stage_change_relationship_kind(
        &self,
        relationship_id: &str,
        relationship_kind: RelationshipKind,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .change_relationship_kind_in_lens(
                self.active_version_id(),
                self.active_lens_id(),
                relationship_id,
                relationship_kind,
            )
            .map_err(WorkspaceStageError::Mutation)?;

        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages relationship removal in the active workspace version.
    pub fn stage_remove_relationship(
        &self,
        relationship_id: &str,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .remove_relationship_in_lens(
                self.active_version_id(),
                self.active_lens_id(),
                relationship_id,
            )
            .map_err(WorkspaceStageError::Mutation)?;

        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a new planning version copied from the selected active version.
    /// The cloned version becomes active and retains the active Lens ID.
    pub fn stage_create_version_from_active(
        &self,
        title: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let version_id = document
            .create_version_from(self.active_version_id(), title)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation_with_active_scope(
            document,
            Vec::new(),
            version_id,
            self.active_lens_id().to_string(),
        )
    }

    /// Stages a version-title change while preserving the active canvas scope.
    pub fn stage_rename_version(
        &self,
        version_id: &str,
        title: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .rename_version(version_id, title)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages removal of a version. When the active planning version is
    /// removed, its source version becomes active.
    pub fn stage_delete_version(
        &self,
        version_id: &str,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let deleting_active_version = version_id == self.active_version_id();
        let next_version_id = if deleting_active_version {
            document
                .planning_scenarios
                .iter()
                .find(|scenario| scenario.version.id == version_id)
                .map(|scenario| scenario.based_on_version_id.clone())
                .unwrap_or_else(|| version_id.to_string())
        } else {
            self.active_version_id().to_string()
        };
        document
            .delete_version(version_id)
            .map_err(WorkspaceStageError::Mutation)?;
        let next_lens_id = if deleting_active_version {
            document
                .default_lens_id(&next_version_id)
                .map_err(WorkspaceStageError::Mutation)?
                .to_string()
        } else {
            self.active_lens_id().to_string()
        };
        self.stage_document_mutation_with_active_scope(
            document,
            Vec::new(),
            next_version_id,
            next_lens_id,
        )
    }

    /// Stages a new Lens copied from the active Lens. The new saved view
    /// becomes active after the document is persisted.
    pub fn stage_create_lens_from_active(
        &self,
        title: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let lens_id = document
            .create_lens_from(self.active_version_id(), self.active_lens_id(), title)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation_with_active_scope(
            document,
            Vec::new(),
            self.active_version_id().to_string(),
            lens_id,
        )
    }

    /// Stages a blank saved view. The workspace facts stay available, but the
    /// new Lens begins with no copied layout or presentation records.
    pub fn stage_create_empty_lens(
        &self,
        title: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let lens_id = document
            .create_empty_lens(self.active_version_id(), title)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation_with_active_scope(
            document,
            Vec::new(),
            self.active_version_id().to_string(),
            lens_id,
        )
    }

    /// Stages a Lens-title change while preserving the active canvas scope.
    pub fn stage_rename_lens(
        &self,
        lens_id: &str,
        title: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .rename_lens(self.active_version_id(), lens_id, title)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages removal of a Lens. When the active Lens is removed, the first
    /// remaining Lens in the active version becomes active.
    pub fn stage_delete_lens(
        &self,
        lens_id: &str,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        document
            .delete_lens(self.active_version_id(), lens_id)
            .map_err(WorkspaceStageError::Mutation)?;
        let next_lens_id = if lens_id == self.active_lens_id() {
            document
                .default_lens_id(self.active_version_id())
                .map_err(WorkspaceStageError::Mutation)?
                .to_string()
        } else {
            self.active_lens_id().to_string()
        };
        self.stage_document_mutation_with_active_scope(
            document,
            Vec::new(),
            self.active_version_id().to_string(),
            next_lens_id,
        )
    }

    /// Stages copies of selected cards in the active Lens. The new cards are
    /// selected, while graph relationships remain exactly as authored.
    pub fn stage_duplicate_active_concepts(
        &self,
        concept_ids: &[u32],
        world_delta: Point,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let copied_concept_ids = document
            .duplicate_concepts_in_lens(
                self.active_version_id(),
                self.active_lens_id(),
                concept_ids,
                world_delta,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, copied_concept_ids)
    }

    /// Pastes captured card content into the active view, including across versions.
    pub fn stage_paste_concepts(
        &self,
        source: &WorkspaceDocument,
        source_version_id: &str,
        source_lens_id: &str,
        concept_ids: &[u32],
        world_delta: Point,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let ids = document
            .copy_concepts_from_lens(
                source,
                source_version_id,
                source_lens_id,
                self.active_version_id(),
                self.active_lens_id(),
                concept_ids,
                world_delta,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, ids)
    }

    /// Stages an explicit paste choice through the normal save/history boundary.
    pub fn stage_paste_concepts_with_mode(
        &self,
        source: &WorkspaceDocument,
        source_version_id: &str,
        source_lens_id: &str,
        concept_ids: &[u32],
        world_delta: Point,
        mode: infinite_note_canvas_canvas::CardPasteMode,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let ids = document
            .paste_concepts_from_lens(
                source,
                source_version_id,
                source_lens_id,
                self.active_version_id(),
                self.active_lens_id(),
                concept_ids,
                world_delta,
                mode,
            )
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, ids)
    }

    /// Stages selection of a version and that version's default Lens.
    pub fn stage_select_version(
        &self,
        version_id: &str,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let lens_id = self
            .document()
            .default_lens_id(version_id)
            .map_err(WorkspaceStageError::Mutation)?
            .to_string();

        self.stage_workspace_mutation(
            self.document().clone(),
            Vec::new(),
            version_id.to_string(),
            lens_id,
            false,
        )
    }

    /// Stages selection of a Lens in the active workspace version.
    pub fn stage_select_lens(
        &self,
        lens_id: &str,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        self.document()
            .lens_title(self.active_version_id(), lens_id)
            .map_err(WorkspaceStageError::Mutation)?;

        self.stage_workspace_mutation(
            self.document().clone(),
            Vec::new(),
            self.active_version_id().to_string(),
            lens_id.to_string(),
            false,
        )
    }

    /// Stages a type change that preserves the concept's canonical data.
    pub fn stage_edit_concept_type(
        &self,
        concept_id: u32,
        type_id: &str,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let title = document
            .concept(self.active_version_id(), concept_id)
            .map_err(WorkspaceStageError::Mutation)?
            .title
            .clone();
        document
            .edit_concept(self.active_version_id(), concept_id, type_id, title)
            .map_err(WorkspaceStageError::Mutation)?;

        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a title change that preserves the concept's canonical data.
    pub fn stage_edit_concept_title(
        &self,
        concept_id: u32,
        title: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let type_id = document
            .concept(self.active_version_id(), concept_id)
            .map_err(WorkspaceStageError::Mutation)?
            .type_id
            .clone();
        document
            .edit_concept(self.active_version_id(), concept_id, &type_id, title)
            .map_err(WorkspaceStageError::Mutation)?;

        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a summary edit while preserving all remaining concept data.
    pub fn stage_edit_concept_summary(
        &self,
        concept_id: u32,
        summary: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        active_concept_mut(&mut document, self.active_version_id(), concept_id)?.summary = summary;

        document
            .synchronize_concept_content(self.active_version_id(), concept_id)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a details edit while preserving all remaining concept data.
    pub fn stage_edit_concept_details(
        &self,
        concept_id: u32,
        details: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        active_concept_mut(&mut document, self.active_version_id(), concept_id)?.details = details;

        document
            .synchronize_concept_content(self.active_version_id(), concept_id)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages a deterministic assertion attribute edit for a concept.
    pub fn stage_edit_concept_assertion(
        &self,
        concept_id: u32,
        mutation: AssertionMutation,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut document = self.document().clone();
        let concept = active_concept_mut(&mut document, self.active_version_id(), concept_id)?;
        match mutation {
            AssertionMutation::Set { key, value } => {
                validate_assertion_key(&key)?;
                concept.attributes.insert(key, value);
            }
            AssertionMutation::Remove { key } => {
                validate_assertion_key(&key)?;
                concept.attributes.remove(&key);
            }
        }

        document
            .synchronize_concept_content(self.active_version_id(), concept_id)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }

    /// Stages one complete concept-inspector save in the active workspace version.
    pub fn stage_edit_active_concept_inspector(
        &self,
        draft: ConceptInspectorEditDraft,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        validate_concept_inspector_draft(&draft)?;

        let mut document = self.document().clone();
        document
            .edit_concept(
                self.active_version_id(),
                draft.concept_id,
                &draft.type_id,
                draft.title,
            )
            .map_err(WorkspaceStageError::Mutation)?;

        let concept =
            active_concept_mut(&mut document, self.active_version_id(), draft.concept_id)?;
        concept.summary = draft.summary;
        concept.details = draft.details;
        concept.attributes = draft.assertion_attributes;

        document
            .synchronize_concept_content(self.active_version_id(), draft.concept_id)
            .map_err(WorkspaceStageError::Mutation)?;
        self.stage_document_mutation(document, self.selection_ids().to_vec())
    }
}

fn active_concept_mut<'a>(
    document: &'a mut WorkspaceDocument,
    version_id: &str,
    concept_id: u32,
) -> Result<&'a mut WorkspaceConcept, WorkspaceStageError> {
    let version = if document.current_state.id == version_id {
        &mut document.current_state
    } else {
        document
            .planning_scenarios
            .iter_mut()
            .find(|scenario| scenario.version.id == version_id)
            .map(|scenario| &mut scenario.version)
            .ok_or_else(|| {
                WorkspaceStageError::Mutation(WorkspaceError::MissingVersion(
                    version_id.to_string(),
                ))
            })?
    };
    version
        .concepts
        .iter_mut()
        .find(|concept| concept.id == concept_id)
        .ok_or_else(|| {
            WorkspaceStageError::Mutation(WorkspaceError::MissingConcept {
                version_id: version_id.to_string(),
                concept_id,
            })
        })
}

fn validate_assertion_key(key: &str) -> Result<(), WorkspaceStageError> {
    if key.trim().is_empty() {
        return Err(WorkspaceStageError::Mutation(WorkspaceError::Validation(
            "assertion key must not be empty".to_string(),
        )));
    }
    if key.chars().any(char::is_control) {
        return Err(WorkspaceStageError::Mutation(WorkspaceError::Validation(
            "assertion key must not contain control characters".to_string(),
        )));
    }
    Ok(())
}

fn validate_concept_inspector_draft(
    draft: &ConceptInspectorEditDraft,
) -> Result<(), WorkspaceStageError> {
    validate_required_inspector_field("concept title", &draft.title)?;
    validate_required_inspector_field("concept type_id", &draft.type_id)?;
    for key in draft.assertion_attributes.keys() {
        validate_assertion_key(key)?;
    }
    Ok(())
}

fn validate_required_inspector_field(
    field_name: &str,
    value: &str,
) -> Result<(), WorkspaceStageError> {
    if value.trim().is_empty() {
        return Err(WorkspaceStageError::Mutation(WorkspaceError::Validation(
            format!("{field_name} must not be empty"),
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use infinite_note_canvas_canvas::WorkspaceDocument;

    use super::*;

    static NEXT_TEMPORARY_PATH: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn move_stages_then_commits_only_the_active_lens_placement() {
        let workspace = TemporaryWorkspace::new("move", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let other_lens_bounds_before = session
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "delivery-and-production")
            .unwrap()
            .placements
            .clone();

        let staged = session
            .stage_move_active_concept(5, Point::new(40.0, -20.0))
            .unwrap();

        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(
            staged.dataset().card_by_id(5).unwrap().bounds.origin,
            Point::new(1120.0, 260.0)
        );
        session.commit(staged).unwrap();

        assert_eq!(
            session.dataset().card_by_id(5).unwrap().bounds.origin,
            Point::new(1120.0, 260.0)
        );
        assert_eq!(
            session
                .document()
                .version("current")
                .unwrap()
                .lenses
                .iter()
                .find(|lens| lens.id == "delivery-and-production")
                .unwrap()
                .placements,
            other_lens_bounds_before
        );
    }

    #[test]
    fn card_resize_stages_persists_and_undoes_only_the_active_lens_placement() {
        let workspace = TemporaryWorkspace::new("resize-card", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let new_bounds = Rect::new(Point::new(1_080.0, 280.0), Point::new(480.0, 240.0));

        let staged = session.stage_resize_active_concept(5, new_bounds).unwrap();

        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(staged.dataset().card_by_id(5).unwrap().bounds, new_bounds);
        session.commit(staged).unwrap();
        assert_eq!(session.dataset().card_by_id(5).unwrap().bounds, new_bounds);
        assert_eq!(session.dataset().relationship_count(), 8);

        session.commit(session.stage_undo().unwrap()).unwrap();
        assert_eq!(
            session.dataset().card_by_id(5).unwrap().bounds,
            Rect::new(Point::new(1_080.0, 280.0), Point::new(240.0, 132.0))
        );
    }

    #[test]
    fn cross_version_reference_content_edits_persist_and_undo_as_one_workspace_change() {
        let workspace = TemporaryWorkspace::new("linked-content", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source = session.document().clone();
        let source_version = session.active_version_id().to_string();
        let source_lens = session.active_lens_id().to_string();
        session
            .commit(
                session
                    .stage_create_version_from_active("Destination".into())
                    .unwrap(),
            )
            .unwrap();
        let target_version = session.active_version_id().to_string();
        let staged = session
            .stage_paste_concepts_with_mode(
                &source,
                &source_version,
                &source_lens,
                &[5],
                Point::new(800.0, 0.0),
                infinite_note_canvas_canvas::CardPasteMode::Reference,
            )
            .unwrap();
        let reference = staged.selection_ids()[0];
        session.commit(staged).unwrap();
        let linked = session.document().clone();
        session
            .commit(
                session
                    .stage_edit_concept_summary(reference, "Shared summary".into())
                    .unwrap(),
            )
            .unwrap();
        session
            .commit(
                session
                    .stage_edit_concept_details(reference, "Shared details".into())
                    .unwrap(),
            )
            .unwrap();
        session
            .commit(
                session
                    .stage_edit_concept_assertion(
                        reference,
                        AssertionMutation::Set {
                            key: "status".into(),
                            value: "ready".into(),
                        },
                    )
                    .unwrap(),
            )
            .unwrap();
        session
            .commit(
                session
                    .stage_edit_concept_title(reference, "Shared title".into())
                    .unwrap(),
            )
            .unwrap();
        let edited = session.document().clone();
        let source_card = edited.concept(&source_version, 5).unwrap();
        assert_eq!(source_card.summary, "Shared summary");
        assert_eq!(source_card.details, "Shared details");
        assert_eq!(source_card.attributes["status"], "ready");
        assert_eq!(source_card.title, "Shared title");
        assert_eq!(
            WorkspaceDocument::parse_json(&workspace.read_source()).unwrap(),
            edited
        );
        for _ in 0..4 {
            session.commit(session.stage_undo().unwrap()).unwrap();
        }
        assert_eq!(session.document(), &linked);
        for _ in 0..4 {
            session.commit(session.stage_redo().unwrap()).unwrap();
        }
        assert_eq!(session.document(), &edited);
        assert_eq!(
            session.document().concept(&target_version, 5).unwrap(),
            source.concept(&source_version, 5).unwrap(),
            "the independently cloned card must stay independent"
        );
    }

    #[test]
    fn reference_removal_preserves_shared_content_and_restores_with_undo() {
        let workspace = TemporaryWorkspace::new("delete-concept", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let before = WorkspaceDocument::parse_json(&workspace.read_source()).unwrap();
        let deleted_concept_id = 5;

        let staged = session
            .stage_remove_active_concept_reference(deleted_concept_id)
            .unwrap();
        assert_eq!(workspace.read_source(), sample_source());
        session.commit(staged).unwrap();

        let after_delete = WorkspaceDocument::parse_json(&workspace.read_source()).unwrap();
        assert_eq!(
            after_delete.current_state.concepts,
            before.current_state.concepts
        );
        assert_eq!(
            after_delete.current_state.relationships,
            before.current_state.relationships
        );
        let active_lens = after_delete
            .current_state
            .lenses
            .iter()
            .find(|lens| lens.id == session.active_lens_id())
            .unwrap();
        assert!(
            active_lens
                .placements
                .iter()
                .all(|placement| placement.concept_id != deleted_concept_id)
        );
        for lens in &after_delete.current_state.lenses {
            if lens.id != session.active_lens_id() {
                assert_eq!(
                    lens,
                    before
                        .current_state
                        .lenses
                        .iter()
                        .find(|other| other.id == lens.id)
                        .unwrap()
                );
            }
        }
        assert!(session.selection_ids().is_empty());

        session.commit(session.stage_undo().unwrap()).unwrap();
        assert_eq!(
            WorkspaceDocument::parse_json(&workspace.read_source()).unwrap(),
            before
        );
    }

    #[test]
    fn multiple_reference_removal_is_atomic_and_records_one_history_entry() {
        let workspace = TemporaryWorkspace::new("delete-concepts", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let before_source = workspace.read_source();
        let deleted_concept_ids = [5, 7];

        let staged = session
            .stage_remove_active_concept_references(&deleted_concept_ids)
            .unwrap();

        assert_eq!(workspace.read_source(), before_source);
        assert!(staged.selection_ids().is_empty());
        assert!(
            deleted_concept_ids
                .iter()
                .all(|concept_id| staged.dataset().card_by_id(*concept_id).is_none())
        );

        session.commit(staged).unwrap();

        assert_eq!(session.undo_depth(), 1);
        assert!(session.selection_ids().is_empty());
        let deleted_document = WorkspaceDocument::parse_json(&workspace.read_source()).unwrap();
        let before = WorkspaceDocument::parse_json(&before_source).unwrap();
        assert_eq!(
            deleted_document.current_state.concepts,
            before.current_state.concepts
        );
        assert_eq!(
            deleted_document.current_state.relationships,
            before.current_state.relationships
        );
        assert!(
            deleted_document
                .current_state
                .lenses
                .iter()
                .find(|lens| lens.id == session.active_lens_id())
                .unwrap()
                .placements
                .iter()
                .all(|placement| !deleted_concept_ids.contains(&placement.concept_id))
        );

        session.commit(session.stage_undo().unwrap()).unwrap();
        assert_eq!(workspace.read_source(), before_source);
    }

    #[test]
    fn multiple_move_stages_selected_cards_and_records_one_history_entry() {
        let workspace = TemporaryWorkspace::new("multiple-move", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let selected_concept_ids = vec![5, 7];
        let source_before = workspace.read_source();
        let other_lens_placements = session
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "delivery-and-production")
            .unwrap()
            .placements
            .clone();

        let staged = session
            .stage_move_active_concepts(&selected_concept_ids, Point::new(40.0, -20.0))
            .unwrap();

        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(staged.selection_ids(), selected_concept_ids);
        assert_eq!(
            staged.dataset().card_by_id(5).unwrap().bounds.origin,
            Point::new(1120.0, 260.0)
        );
        assert_eq!(
            staged.dataset().card_by_id(7).unwrap().bounds.origin,
            Point::new(800.0, 500.0)
        );
        assert_eq!(session.undo_depth(), 0);

        session.commit(staged).unwrap();

        assert_eq!(session.selection_ids(), selected_concept_ids);
        assert_eq!(session.undo_depth(), 1);
        assert_eq!(
            session
                .document()
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
    fn invalid_multiple_move_does_not_stage_or_change_session_history() {
        let workspace = TemporaryWorkspace::new("invalid-multiple-move", &sample_source());
        let session = load_sample_session(workspace.path.clone());
        let state_before = ResidentState::capture(&session, &workspace);

        assert!(matches!(
            session.stage_move_active_concepts(&[5, 999], Point::new(40.0, -20.0)),
            Err(WorkspaceStageError::Mutation(
                WorkspaceError::MissingPlacement {
                    concept_id: 999,
                    ..
                }
            ))
        ));
        assert_eq!(session.undo_depth(), 0);
        assert_unchanged(&session, &workspace, &state_before);
    }

    #[test]
    fn concept_creation_stages_then_commits_a_selected_active_lens_card() {
        let workspace = TemporaryWorkspace::new("create-concept", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();

        let staged = session
            .stage_create_active_concept(ConceptDraft {
                type_id: "project".to_string(),
                title: "Evaluation Project".to_string(),
                summary: "A deterministic staged concept.".to_string(),
                bounds: Rect::new(Point::new(1600.0, 100.0), Point::new(240.0, 120.0)),
            })
            .unwrap();

        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(staged.selection_ids(), [10]);
        assert_eq!(
            staged.dataset().card_by_id(10).unwrap().title,
            "Evaluation Project"
        );
        session.commit(staged).unwrap();

        assert_eq!(session.selection_ids(), [10]);
        assert_eq!(
            session.document().concept("current", 10).unwrap().title,
            "Evaluation Project"
        );
    }

    #[test]
    fn visual_boundary_creation_stages_then_commits_without_changing_the_graph() {
        let workspace = TemporaryWorkspace::new("create-boundary", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let relationship_count = session.dataset().relationship_count();

        let staged = session
            .stage_create_visual_boundary(
                "Release context".to_string(),
                "Visual scope for the release path.".to_string(),
                Rect::new(Point::new(640.0, 180.0), Point::new(560.0, 360.0)),
            )
            .unwrap();

        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(staged.dataset().relationship_count(), relationship_count);
        session.commit(staged).unwrap();

        let lens = session
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        assert_eq!(lens.visual_boundaries[0].title, "Release context");
        assert_eq!(session.dataset().relationship_count(), relationship_count);
    }

    #[test]
    fn visual_boundary_actions_persist_undo_redo_and_leave_graph_unchanged() {
        let workspace = TemporaryWorkspace::new("edit-boundary", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let relationship_count = session.dataset().relationship_count();

        let staged = session
            .stage_create_visual_boundary(
                "Release context".to_string(),
                String::new(),
                Rect::new(Point::new(640.0, 180.0), Point::new(560.0, 360.0)),
            )
            .unwrap();
        session.commit(staged).unwrap();
        let boundary_id = session
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .visual_boundaries[0]
            .id
            .clone();

        session
            .commit(
                session
                    .stage_update_visual_boundary(
                        &boundary_id,
                        "Shared delivery context".to_string(),
                        "Visual context only.".to_string(),
                    )
                    .unwrap(),
            )
            .unwrap();
        session
            .commit(
                session
                    .stage_move_visual_boundary(&boundary_id, Point::new(32.0, 24.0))
                    .unwrap(),
            )
            .unwrap();
        session
            .commit(
                session
                    .stage_resize_visual_boundary(
                        &boundary_id,
                        Rect::new(Point::new(672.0, 204.0), Point::new(640.0, 400.0)),
                    )
                    .unwrap(),
            )
            .unwrap();

        let lens = session
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap();
        assert_eq!(lens.visual_boundaries[0].title, "Shared delivery context");
        assert_eq!(
            lens.visual_boundaries[0].bounds,
            Rect::new(Point::new(672.0, 204.0), Point::new(640.0, 400.0))
        );
        assert_eq!(session.dataset().relationship_count(), relationship_count);

        session.commit(session.stage_undo().unwrap()).unwrap();
        assert_eq!(
            session
                .document()
                .version("current")
                .unwrap()
                .lenses
                .iter()
                .find(|lens| lens.id == "organization-landscape")
                .unwrap()
                .visual_boundaries[0]
                .bounds,
            Rect::new(Point::new(672.0, 204.0), Point::new(560.0, 360.0))
        );
        session.commit(session.stage_redo().unwrap()).unwrap();
        assert_eq!(session.dataset().relationship_count(), relationship_count);

        session
            .commit(session.stage_remove_visual_boundary(&boundary_id).unwrap())
            .unwrap();
        assert!(
            session
                .document()
                .version("current")
                .unwrap()
                .lenses
                .iter()
                .find(|lens| lens.id == "organization-landscape")
                .unwrap()
                .visual_boundaries
                .is_empty()
        );
        assert_eq!(session.dataset().relationship_count(), relationship_count);
    }

    #[test]
    fn relationship_actions_stage_create_change_and_removal() {
        let workspace = TemporaryWorkspace::new("relationship", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();

        let staged = session
            .stage_create_relationship(2, 4, RelationshipKind::Owns)
            .unwrap();
        assert_eq!(workspace.read_source(), source_before);
        session.commit(staged).unwrap();
        let relationship_id = "rel-2-4-owns";
        assert!(
            session
                .document()
                .relationships_in_lens("current", "organization-landscape")
                .unwrap()
                .iter()
                .any(|relationship| relationship.id == relationship_id)
        );

        let staged = session
            .stage_change_relationship_kind(relationship_id, RelationshipKind::Operates)
            .unwrap();
        session.commit(staged).unwrap();
        assert_eq!(
            session
                .document()
                .relationships_in_lens("current", "organization-landscape")
                .unwrap()
                .iter()
                .find(|relationship| relationship.id == relationship_id)
                .unwrap()
                .kind,
            RelationshipKind::Operates
        );

        let staged = session.stage_remove_relationship(relationship_id).unwrap();
        session.commit(staged).unwrap();
        assert!(
            !session
                .document()
                .relationships_in_lens("current", "organization-landscape")
                .unwrap()
                .iter()
                .any(|relationship| relationship.id == relationship_id)
        );
    }

    #[test]
    fn layer_creation_stages_the_requested_initial_percent_range_and_fade() {
        let workspace = TemporaryWorkspace::new("layer-percent-creation", &sample_source());
        let session = load_sample_session(workspace.path.clone());
        let settings = SemanticZoomLayerPercentSettings {
            active_zoom_range: infinite_note_canvas_canvas::ZoomPercentRange {
                start_percent: 72.0,
                end_percent: 128.0,
            },
            fade_percent: 11.0,
        };

        let staged = session
            .stage_create_semantic_zoom_base_layer_with_percent_settings(
                "Working map".to_string(),
                settings,
            )
            .unwrap();
        let layer = &staged
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers[0];

        assert_eq!(layer.percent_settings, Some(settings));
        assert_eq!(workspace.read_source(), sample_source());
    }

    #[test]
    fn semantic_zoom_layer_actions_stage_persist_and_record_history() {
        let workspace = TemporaryWorkspace::new("semantic-zoom-layers", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();

        let staged = session
            .stage_create_semantic_zoom_base_layer("Overview".to_string())
            .unwrap();
        let base_layer = &staged
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers[0];
        assert_eq!(base_layer.concept_ids.len(), 9);
        assert_eq!(base_layer.relationship_ids.len(), 8);
        assert_eq!(workspace.read_source(), source_before);
        session.commit(staged).unwrap();

        let staged = session
            .stage_create_semantic_zoom_layer("Delivery detail".to_string())
            .unwrap();
        session.commit(staged).unwrap();
        let detail_layer_id = session.available_semantic_zoom_layers().unwrap()[1]
            .id
            .clone();
        let staged = session
            .stage_set_semantic_zoom_layer_concepts(&detail_layer_id, &[7], true)
            .unwrap();
        session.commit(staged).unwrap();

        let layers = session.available_semantic_zoom_layers().unwrap();
        assert_eq!(
            layers
                .iter()
                .map(|layer| layer.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Overview", "Delivery detail"]
        );
        let semantic_layers = &session
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers;
        assert!(!semantic_layers[0].concept_ids.contains(&7));
        assert_eq!(semantic_layers[1].concept_ids, vec![7]);
        assert!(session.can_undo());

        let staged = session.stage_undo().unwrap();
        session.commit(staged).unwrap();
        let restored_layers = &session
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "organization-landscape")
            .unwrap()
            .semantic_zoom_layers;
        assert!(restored_layers[0].concept_ids.contains(&7));
        assert!(restored_layers[1].concept_ids.is_empty());
    }

    #[test]
    fn version_and_lens_selection_stage_without_persisting_the_document() {
        let mut document = WorkspaceDocument::sample();
        document
            .create_planning_scenario(
                "release-plan".to_string(),
                "Release Plan".to_string(),
                "current",
            )
            .unwrap();
        let workspace = TemporaryWorkspace::new(
            "selection",
            &document.to_pretty_json().expect("sample must serialize"),
        );
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let digest_before = session.expected_digest();

        assert_eq!(
            session.available_versions(),
            vec![
                VersionChoice {
                    id: "current".to_string(),
                    title: "Current State".to_string(),
                },
                VersionChoice {
                    id: "release-plan".to_string(),
                    title: "Release Plan".to_string(),
                },
            ]
        );
        assert_eq!(
            session.available_lenses("release-plan").unwrap(),
            vec![
                LensChoice {
                    id: "organization-landscape".to_string(),
                    title: "Organization Landscape".to_string(),
                },
                LensChoice {
                    id: "delivery-and-production".to_string(),
                    title: "Delivery and Production".to_string(),
                },
            ]
        );

        let staged = session.stage_select_version("release-plan").unwrap();
        assert_eq!(staged.active_version_id(), "release-plan");
        assert_eq!(staged.active_lens_id(), "organization-landscape");
        assert_eq!(workspace.read_source(), source_before);
        session.commit(staged).unwrap();

        let staged = session
            .stage_select_lens("delivery-and-production")
            .unwrap();
        assert_eq!(workspace.read_source(), source_before);
        session.commit(staged).unwrap();

        assert_eq!(session.active_version_id(), "release-plan");
        assert_eq!(session.active_lens_id(), "delivery-and-production");
        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(session.expected_digest(), digest_before);
    }

    #[test]
    fn structural_authoring_and_card_copy_stage_as_persisted_changes() {
        let workspace = TemporaryWorkspace::new("structure", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let before_source = workspace.read_source();

        let staged = session
            .stage_create_version_from_active("Future state".to_string())
            .unwrap();
        assert_eq!(staged.active_version_id(), "version-future-state");
        assert_eq!(staged.active_lens_id(), "organization-landscape");
        assert_eq!(workspace.read_source(), before_source);
        session.commit(staged).unwrap();
        assert_eq!(session.active_version_id(), "version-future-state");
        assert_eq!(session.undo_depth(), 1);

        let staged = session
            .stage_create_lens_from_active("Decision view".to_string())
            .unwrap();
        assert_eq!(staged.active_lens_id(), "lens-decision-view");
        session.commit(staged).unwrap();
        assert_eq!(session.active_lens_id(), "lens-decision-view");

        let staged = session
            .stage_create_empty_lens("Blank decision view".to_string())
            .unwrap();
        assert_eq!(staged.active_lens_id(), "lens-blank-decision-view");
        let blank_lens = staged
            .document()
            .version(staged.active_version_id())
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == "lens-blank-decision-view")
            .unwrap();
        assert!(blank_lens.placements.is_empty());
        assert!(blank_lens.visual_boundaries.is_empty());
        assert!(blank_lens.semantic_zoom_layers.is_empty());
        session.commit(staged).unwrap();
        assert_eq!(session.active_lens_id(), "lens-blank-decision-view");

        let staged = session.stage_select_lens("lens-decision-view").unwrap();
        session.commit(staged).unwrap();

        let relationship_count = session
            .document()
            .version(session.active_version_id())
            .unwrap()
            .relationships
            .len();
        let staged = session
            .stage_duplicate_active_concepts(&[5], Point::new(36.0, 36.0))
            .unwrap();
        let copied_id = staged.selection_ids()[0];
        assert_ne!(copied_id, 5);
        assert_eq!(
            staged
                .document()
                .version(staged.active_version_id())
                .unwrap()
                .relationships
                .len(),
            relationship_count
        );
        session.commit(staged).unwrap();
        assert_eq!(session.selection_ids(), &[copied_id]);

        let staged = session
            .stage_rename_lens("lens-decision-view", "Decision view 2".to_string())
            .unwrap();
        session.commit(staged).unwrap();
        assert_eq!(
            session
                .document()
                .lens_title(session.active_version_id(), session.active_lens_id())
                .unwrap(),
            "Decision view 2"
        );

        let staged = session.stage_delete_lens("lens-decision-view").unwrap();
        session.commit(staged).unwrap();
        assert_eq!(session.active_lens_id(), "organization-landscape");

        let staged = session
            .stage_delete_version("version-future-state")
            .unwrap();
        session.commit(staged).unwrap();
        assert_eq!(session.active_version_id(), "current");
        assert_eq!(session.active_lens_id(), "organization-landscape");
    }

    #[test]
    fn concept_type_and_title_edits_stage_then_commit() {
        let workspace = TemporaryWorkspace::new("edit-concept", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let original = session.document().concept("current", 5).unwrap().clone();

        let staged = session.stage_edit_concept_type(5, "repository").unwrap();
        assert_eq!(workspace.read_source(), source_before);
        session.commit(staged).unwrap();
        let type_edited = session.document().concept("current", 5).unwrap();
        assert_eq!(type_edited.type_id, "repository");
        assert_eq!(type_edited.title, original.title);
        assert_eq!(type_edited.summary, original.summary);

        let staged = session
            .stage_edit_concept_title(5, "Portal API Source".to_string())
            .unwrap();
        session.commit(staged).unwrap();
        let title_edited = session.document().concept("current", 5).unwrap();
        assert_eq!(title_edited.type_id, "repository");
        assert_eq!(title_edited.title, "Portal API Source");
        assert_eq!(title_edited.summary, original.summary);
    }

    #[test]
    fn content_and_assertion_edits_persist_deterministically_without_changing_other_data() {
        let workspace = TemporaryWorkspace::new("content-and-assertions", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let original = session.document().concept("current", 5).unwrap().clone();

        let staged = session
            .stage_edit_concept_summary(5, String::new())
            .expect("empty summary must stage");
        assert_eq!(workspace.read_source(), sample_source());
        assert_eq!(staged.document().concept("current", 5).unwrap().summary, "");
        session.commit(staged).expect("summary must save");

        let staged = session
            .stage_edit_concept_details(5, "User-provided details.".to_string())
            .expect("details must stage");
        session.commit(staged).expect("details must save");

        for mutation in [
            AssertionMutation::Set {
                key: "zeta".to_string(),
                value: "last".to_string(),
            },
            AssertionMutation::Set {
                key: "alpha".to_string(),
                value: String::new(),
            },
            AssertionMutation::Set {
                key: "zeta".to_string(),
                value: "replaced".to_string(),
            },
        ] {
            let staged = session
                .stage_edit_concept_assertion(5, mutation)
                .expect("assertion must stage");
            session.commit(staged).expect("assertion must save");
        }

        let edited = session.document().concept("current", 5).unwrap();
        assert_eq!(edited.type_id, original.type_id);
        assert_eq!(edited.title, original.title);
        assert_eq!(edited.summary, "");
        assert_eq!(edited.details, "User-provided details.");
        assert_eq!(
            edited.attributes,
            std::collections::BTreeMap::from([
                ("alpha".to_string(), String::new()),
                ("zeta".to_string(), "replaced".to_string()),
            ])
        );
        let source = workspace.read_source();
        assert!(
            source.find("\"alpha\": \"\"").unwrap()
                < source.find("\"zeta\": \"replaced\"").unwrap()
        );
        assert_eq!(
            WorkspaceDocument::parse_json(&source)
                .unwrap()
                .concept("current", 5)
                .unwrap(),
            edited
        );

        let staged = session
            .stage_edit_concept_assertion(
                5,
                AssertionMutation::Remove {
                    key: "zeta".to_string(),
                },
            )
            .expect("assertion removal must stage");
        session.commit(staged).expect("assertion removal must save");
        assert_eq!(
            session.document().concept("current", 5).unwrap().attributes,
            std::collections::BTreeMap::from([("alpha".to_string(), String::new())])
        );
    }

    #[test]
    fn complete_inspector_save_stages_one_history_entry_and_preserves_unedited_data() {
        let workspace = TemporaryWorkspace::new("complete-inspector-save", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let document_before = session.document().clone();
        let concept_before = document_before.concept("current", 5).unwrap().clone();
        let relationships_before = document_before
            .version("current")
            .unwrap()
            .relationships
            .clone();
        let lenses_before = document_before.version("current").unwrap().lenses.clone();
        let selection_before = session.selection_ids().to_vec();

        let staged = session
            .stage_edit_active_concept_inspector(ConceptInspectorEditDraft {
                concept_id: 5,
                title: "Portal API Source".to_string(),
                type_id: "repository".to_string(),
                summary: "The canonical Portal API repository.".to_string(),
                details: "Contains the API implementation and deployment configuration."
                    .to_string(),
                assertion_attributes: BTreeMap::from([
                    ("owner".to_string(), "platform-team".to_string()),
                    ("tier".to_string(), "critical".to_string()),
                ]),
            })
            .expect("the complete inspector draft must stage");

        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(session.document(), &document_before);
        assert_eq!(session.undo_depth(), 0);
        assert_eq!(staged.selection_ids(), selection_before);
        let staged_concept = staged.document().concept("current", 5).unwrap();
        assert_eq!(staged_concept.id, concept_before.id);
        assert_eq!(staged_concept.title, "Portal API Source");
        assert_eq!(staged_concept.type_id, "repository");
        assert_eq!(
            staged_concept.summary,
            "The canonical Portal API repository."
        );
        assert_eq!(
            staged_concept.details,
            "Contains the API implementation and deployment configuration."
        );
        assert_eq!(staged_concept.execution_mode, concept_before.execution_mode);
        assert_eq!(
            staged_concept.attributes,
            BTreeMap::from([
                ("owner".to_string(), "platform-team".to_string()),
                ("tier".to_string(), "critical".to_string()),
            ])
        );
        assert_eq!(
            staged.document().version("current").unwrap().relationships,
            relationships_before
        );
        assert_eq!(
            staged.document().version("current").unwrap().lenses,
            lenses_before
        );

        session
            .commit(staged)
            .expect("the complete inspector save must persist");

        assert_eq!(session.undo_depth(), 1);
        assert_eq!(session.redo_depth(), 0);
        assert_eq!(session.selection_ids(), selection_before);
        assert_eq!(
            session.document().version("current").unwrap().relationships,
            relationships_before
        );
        assert_eq!(
            session.document().version("current").unwrap().lenses,
            lenses_before
        );
        assert_eq!(
            WorkspaceDocument::parse_json(&workspace.read_source()).unwrap(),
            *session.document()
        );
    }

    #[test]
    fn invalid_complete_inspector_drafts_leave_source_session_and_history_unchanged() {
        let workspace =
            TemporaryWorkspace::new("rejected-complete-inspector-save", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let staged = session
            .stage_edit_concept_summary(5, "Saved before invalid drafts.".to_string())
            .expect("a deterministic history entry must stage");
        session.commit(staged).expect("the history entry must save");
        let state_before = ResidentState::capture(&session, &workspace);
        assert_eq!(state_before.undo_depth, 1);

        for draft in [
            ConceptInspectorEditDraft {
                concept_id: 5,
                title: " \t".to_string(),
                type_id: "repository".to_string(),
                summary: "Summary".to_string(),
                details: "Details".to_string(),
                assertion_attributes: BTreeMap::new(),
            },
            ConceptInspectorEditDraft {
                concept_id: 5,
                title: "Portal API Source".to_string(),
                type_id: "unknown".to_string(),
                summary: "Summary".to_string(),
                details: "Details".to_string(),
                assertion_attributes: BTreeMap::new(),
            },
            ConceptInspectorEditDraft {
                concept_id: 5,
                title: "Portal API Source".to_string(),
                type_id: "repository".to_string(),
                summary: "Summary".to_string(),
                details: "Details".to_string(),
                assertion_attributes: BTreeMap::from([(" \t".to_string(), "value".to_string())]),
            },
            ConceptInspectorEditDraft {
                concept_id: 7,
                title: "Release process".to_string(),
                type_id: "service".to_string(),
                summary: "Summary".to_string(),
                details: "Details".to_string(),
                assertion_attributes: BTreeMap::new(),
            },
        ] {
            assert!(session.stage_edit_active_concept_inspector(draft).is_err());
            assert_unchanged(&session, &workspace, &state_before);
        }
    }

    #[test]
    fn invalid_or_no_op_assertion_edits_leave_the_workspace_and_history_unchanged() {
        let workspace = TemporaryWorkspace::new("rejected-assertions", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let document_before = session.document().clone();

        assert!(matches!(
            session.stage_edit_concept_assertion(
                5,
                AssertionMutation::Set {
                    key: " \t".to_string(),
                    value: "value".to_string(),
                },
            ),
            Err(WorkspaceStageError::Mutation(WorkspaceError::Validation(message)))
                if message == "assertion key must not be empty"
        ));
        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(session.document(), &document_before);
        assert!(!session.can_undo());

        let staged = session
            .stage_edit_concept_assertion(
                5,
                AssertionMutation::Remove {
                    key: "absent".to_string(),
                },
            )
            .expect("removing an absent assertion must be a no-op");
        session.commit(staged).expect("no-op mutation must commit");

        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(session.document(), &document_before);
        assert!(!session.can_undo());
    }

    #[test]
    fn rejected_actions_leave_document_dataset_selection_and_source_unchanged() {
        let workspace = TemporaryWorkspace::new("rejected-actions", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let staged = session
            .stage_document_mutation(session.document().clone(), vec![2])
            .unwrap();
        session.commit(staged).unwrap();
        let state_before = ResidentState::capture(&session, &workspace);

        assert!(
            session
                .stage_move_active_concept(999, Point::new(1.0, 1.0))
                .is_err()
        );
        assert_unchanged(&session, &workspace, &state_before);
        assert!(
            session
                .stage_create_active_concept(ConceptDraft {
                    type_id: "unknown".to_string(),
                    title: "Rejected".to_string(),
                    summary: "Rejected".to_string(),
                    bounds: Rect::new(Point::new(1.0, 1.0), Point::new(10.0, 10.0)),
                })
                .is_err()
        );
        assert!(
            session
                .stage_create_relationship(2, 2, RelationshipKind::Owns)
                .is_err()
        );
        assert!(
            session
                .stage_change_relationship_kind("missing", RelationshipKind::Owns)
                .is_err()
        );
        assert!(session.stage_remove_relationship("missing").is_err());
        assert!(session.stage_select_version("missing").is_err());
        assert!(session.stage_select_lens("missing").is_err());
        assert!(session.stage_edit_concept_type(5, "unknown").is_err());
        assert!(
            session
                .stage_edit_concept_title(5, "   ".to_string())
                .is_err()
        );
        assert_unchanged(&session, &workspace, &state_before);
    }

    struct ResidentState {
        source: String,
        digest: Option<crate::SourceDigest>,
        document: WorkspaceDocument,
        card_count: usize,
        relationship_count: usize,
        selection_ids: Vec<u32>,
        undo_depth: usize,
        redo_depth: usize,
    }

    impl ResidentState {
        fn capture(session: &WorkspaceSession, workspace: &TemporaryWorkspace) -> Self {
            Self {
                source: workspace.read_source(),
                digest: session.expected_digest(),
                document: session.document().clone(),
                card_count: session.dataset().card_count(),
                relationship_count: session.dataset().relationship_count(),
                selection_ids: session.selection_ids().to_vec(),
                undo_depth: session.undo_depth(),
                redo_depth: session.redo_depth(),
            }
        }
    }

    fn assert_unchanged(
        session: &WorkspaceSession,
        workspace: &TemporaryWorkspace,
        state_before: &ResidentState,
    ) {
        assert_eq!(workspace.read_source(), state_before.source);
        assert_eq!(session.expected_digest(), state_before.digest);
        assert_eq!(session.document(), &state_before.document);
        assert_eq!(session.dataset().card_count(), state_before.card_count);
        assert_eq!(
            session.dataset().relationship_count(),
            state_before.relationship_count
        );
        assert_eq!(session.selection_ids(), state_before.selection_ids);
        assert_eq!(session.undo_depth(), state_before.undo_depth);
        assert_eq!(session.redo_depth(), state_before.redo_depth);
    }

    fn load_sample_session(path: PathBuf) -> WorkspaceSession {
        WorkspaceSession::load(path, "current", "organization-landscape").unwrap()
    }

    fn sample_source() -> String {
        WorkspaceDocument::sample().to_pretty_json().unwrap()
    }

    struct TemporaryWorkspace {
        path: PathBuf,
    }

    impl TemporaryWorkspace {
        fn new(label: &str, source: &str) -> Self {
            let unique = NEXT_TEMPORARY_PATH.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "infinite-note-canvas-evaluation-actions-{label}-{}-{unique}.json",
                std::process::id()
            ));
            remove_workspace_files(&path);
            fs::write(&path, source).unwrap();
            Self { path }
        }

        fn read_source(&self) -> String {
            fs::read_to_string(&self.path).unwrap()
        }
    }

    impl Drop for TemporaryWorkspace {
        fn drop(&mut self) {
            remove_workspace_files(&self.path);
        }
    }

    fn remove_workspace_files(path: &Path) {
        for candidate in [
            path.to_path_buf(),
            PathBuf::from(format!("{}.bak", path.display())),
            PathBuf::from(format!("{}.tmp", path.display())),
        ] {
            if candidate.exists() {
                fs::remove_file(candidate).unwrap();
            }
        }
    }
}
