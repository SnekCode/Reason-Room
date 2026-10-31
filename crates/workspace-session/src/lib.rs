//! Local workspace persistence primitives with no UI or network dependencies.

pub mod evaluation_actions;
#[cfg(not(target_arch = "wasm32"))]
pub mod local_store;
pub mod session;
pub mod source_digest;

pub use evaluation_actions::{ConceptDraft, LensChoice, SemanticZoomLayerChoice, VersionChoice};
#[cfg(not(target_arch = "wasm32"))]
pub use local_store::{
    LocalCreateError, LocalSaveError, LocalWorkspacePath, LocalWorkspaceStore, PersistedCandidate,
};
pub use session::{
    StagedWorkspaceMutation, WorkspaceCreateError, WorkspaceLoadError, WorkspaceSaveError,
    WorkspaceSession, WorkspaceStageError,
};
pub use source_digest::SourceDigest;
