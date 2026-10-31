use std::fmt;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
use std::path::{Path, PathBuf};

use infinite_note_canvas_canvas::{CanvasDataset, WorkspaceDocument, WorkspaceError};

use crate::SourceDigest;
#[cfg(not(target_arch = "wasm32"))]
use crate::{LocalCreateError, LocalSaveError, LocalWorkspaceStore};

/// The maximum number of successful document changes retained for undo.
pub const WORKSPACE_HISTORY_LIMIT: usize = 64;

/// A validated local workspace and its selected canvas dataset.
pub struct WorkspaceSession {
    document: WorkspaceDocument,
    dataset: CanvasDataset,
    selection_ids: Vec<u32>,
    active_version_id: String,
    active_lens_id: String,
    store: WorkspaceStorage,
    history: WorkspaceHistory,
}

/// The workspace source backing a loaded session.
///
/// Desktop sessions keep their verified local-file boundary. Browser sessions
/// keep their document only for the current page session. The model and staged
/// actions remain identical in both cases.
enum WorkspaceStorage {
    #[cfg(not(target_arch = "wasm32"))]
    Local(LocalWorkspaceStore),
    #[cfg(target_arch = "wasm32")]
    BrowserMemory,
}

impl WorkspaceSession {
    /// Creates, verifies, and opens a new local workspace document.
    ///
    /// Creation is exclusive: if the requested path already exists, it is
    /// left unchanged and the caller receives an actionable error.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_new(
        path: PathBuf,
        document: WorkspaceDocument,
    ) -> Result<Self, WorkspaceCreateError> {
        document
            .validate()
            .map_err(WorkspaceCreateError::Validation)?;
        let active_version_id = document.current_state_id().to_string();
        let active_lens_id = document
            .default_lens_id(&active_version_id)
            .map(str::to_string)
            .map_err(WorkspaceCreateError::Dataset)?;
        let dataset = document
            .canvas_dataset(&active_version_id, &active_lens_id)
            .map_err(WorkspaceCreateError::Dataset)?;
        let source = document
            .to_pretty_json()
            .map_err(WorkspaceCreateError::Serialization)?;
        let store = LocalWorkspaceStore::create_new(path, &source, |persisted_source| {
            WorkspaceDocument::parse_json(persisted_source)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .map_err(WorkspaceCreateError::Persistence)?;

        Ok(Self {
            document,
            dataset,
            selection_ids: Vec::new(),
            active_version_id,
            active_lens_id,
            store: WorkspaceStorage::Local(store),
            history: WorkspaceHistory::default(),
        })
    }

    /// Loads a workspace from a local path and builds the requested dataset.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(
        path: PathBuf,
        active_version_id: impl Into<String>,
        active_lens_id: impl Into<String>,
    ) -> Result<Self, WorkspaceLoadError> {
        let source = fs::read_to_string(&path).map_err(|error| WorkspaceLoadError {
            path: path.clone(),
            message: format!("could not read workspace source: {error}"),
        })?;
        let document =
            WorkspaceDocument::parse_json(&source).map_err(|error| WorkspaceLoadError {
                path: path.clone(),
                message: format!("could not validate workspace document: {error}"),
            })?;
        let active_version_id = active_version_id.into();
        let active_lens_id = active_lens_id.into();
        let dataset = document
            .canvas_dataset(&active_version_id, &active_lens_id)
            .map_err(|error| WorkspaceLoadError {
                path: path.clone(),
                message: format!(
                    "could not build dataset for version {active_version_id} and lens {active_lens_id}: {error}"
                ),
            })?;

        Ok(Self {
            document,
            dataset,
            selection_ids: Vec::new(),
            active_version_id,
            active_lens_id,
            store: WorkspaceStorage::Local(LocalWorkspaceStore::open(path, source.as_bytes())),
            history: WorkspaceHistory::default(),
        })
    }

    /// Opens a validated workspace for the current browser page session.
    #[cfg(target_arch = "wasm32")]
    pub fn open_browser(document: WorkspaceDocument) -> Result<Self, WorkspaceStageError> {
        document
            .validate()
            .map_err(WorkspaceStageError::Validation)?;
        let active_version_id = document.current_state_id().to_string();
        let active_lens_id = document
            .default_lens_id(&active_version_id)
            .map(str::to_string)
            .map_err(WorkspaceStageError::Dataset)?;
        let dataset = document
            .canvas_dataset(&active_version_id, &active_lens_id)
            .map_err(WorkspaceStageError::Dataset)?;
        Ok(Self {
            document,
            dataset,
            selection_ids: Vec::new(),
            active_version_id,
            active_lens_id,
            store: WorkspaceStorage::BrowserMemory,
            history: WorkspaceHistory::default(),
        })
    }

    /// Creates a new workspace for the current browser page session.
    #[cfg(target_arch = "wasm32")]
    pub fn create_new_browser(document: WorkspaceDocument) -> Result<Self, WorkspaceCreateError> {
        document
            .validate()
            .map_err(WorkspaceCreateError::Validation)?;
        let active_version_id = document.current_state_id().to_string();
        let active_lens_id = document
            .default_lens_id(&active_version_id)
            .map(str::to_string)
            .map_err(WorkspaceCreateError::Dataset)?;
        let dataset = document
            .canvas_dataset(&active_version_id, &active_lens_id)
            .map_err(WorkspaceCreateError::Dataset)?;
        Ok(Self {
            document,
            dataset,
            selection_ids: Vec::new(),
            active_version_id,
            active_lens_id,
            store: WorkspaceStorage::BrowserMemory,
            history: WorkspaceHistory::default(),
        })
    }

    /// Loads a workspace into its current-state version and first saved Lens.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_default(path: PathBuf) -> Result<Self, WorkspaceLoadError> {
        let source = fs::read_to_string(&path).map_err(|error| WorkspaceLoadError {
            path: path.clone(),
            message: format!("could not read workspace source: {error}"),
        })?;
        let document =
            WorkspaceDocument::parse_json(&source).map_err(|error| WorkspaceLoadError {
                path: path.clone(),
                message: format!("could not validate workspace document: {error}"),
            })?;
        let active_version_id = document.current_state_id().to_string();
        let active_lens_id = document
            .default_lens_id(&active_version_id)
            .map(str::to_string)
            .map_err(|error| WorkspaceLoadError {
                path: path.clone(),
                message: format!("could not select the default Lens: {error}"),
            })?;

        Self::load(path, active_version_id, active_lens_id)
    }

    /// Returns the validated resident workspace document.
    pub fn document(&self) -> &WorkspaceDocument {
        &self.document
    }

    /// Returns the dataset for the active version and Lens.
    pub fn dataset(&self) -> &CanvasDataset {
        &self.dataset
    }

    /// Returns the active workspace version identifier.
    pub fn active_version_id(&self) -> &str {
        &self.active_version_id
    }

    /// Returns the active Lens identifier.
    pub fn active_lens_id(&self) -> &str {
        &self.active_lens_id
    }

    /// Returns the selected concept identifiers.
    pub fn selection_ids(&self) -> &[u32] {
        &self.selection_ids
    }

    /// Returns the local workspace path.
    pub fn workspace_path(&self) -> Option<&Path> {
        match &self.store {
            #[cfg(not(target_arch = "wasm32"))]
            WorkspaceStorage::Local(store) => Some(store.path()),
            #[cfg(target_arch = "wasm32")]
            WorkspaceStorage::BrowserMemory => None,
        }
    }

    /// Returns the digest expected before the next candidate save.
    pub fn expected_digest(&self) -> Option<SourceDigest> {
        match &self.store {
            #[cfg(not(target_arch = "wasm32"))]
            WorkspaceStorage::Local(store) => store.expected_digest(),
            #[cfg(target_arch = "wasm32")]
            WorkspaceStorage::BrowserMemory => None,
        }
    }

    /// Reloads this session when its local workspace source changed outside
    /// the session, for example through the local MCP server.
    ///
    /// The refreshed session preserves the active version and Lens when they
    /// remain valid. Otherwise it selects the document's current-state
    /// version and default Lens. Selection and history are deliberately
    /// cleared because they describe the previous source revision.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn reload_if_local_source_changed(&self) -> Result<Option<Self>, WorkspaceLoadError> {
        let Some(path) = self.workspace_path().map(Path::to_path_buf) else {
            return Ok(None);
        };
        let source = fs::read_to_string(&path).map_err(|error| WorkspaceLoadError {
            path: path.clone(),
            message: format!("could not read workspace source: {error}"),
        })?;
        if self.expected_digest() == Some(SourceDigest::from_bytes(source.as_bytes())) {
            return Ok(None);
        }

        let document =
            WorkspaceDocument::parse_json(&source).map_err(|error| WorkspaceLoadError {
                path: path.clone(),
                message: format!("could not validate workspace document: {error}"),
            })?;
        let (active_version_id, active_lens_id) = document
            .canvas_dataset(&self.active_version_id, &self.active_lens_id)
            .map(|_| (self.active_version_id.clone(), self.active_lens_id.clone()))
            .or_else(|_| {
                let active_version_id = document.current_state_id().to_string();
                let active_lens_id = document.default_lens_id(&active_version_id)?.to_string();
                Ok((active_version_id, active_lens_id))
            })
            .map_err(|error: WorkspaceError| WorkspaceLoadError {
                path: path.clone(),
                message: format!("could not select a canvas after the external update: {error}"),
            })?;
        let dataset = document
            .canvas_dataset(&active_version_id, &active_lens_id)
            .map_err(|error| WorkspaceLoadError {
                path: path.clone(),
                message: format!(
                    "could not build dataset for version {active_version_id} and lens {active_lens_id}: {error}"
                ),
            })?;

        Ok(Some(Self {
            document,
            dataset,
            selection_ids: Vec::new(),
            active_version_id,
            active_lens_id,
            store: WorkspaceStorage::Local(LocalWorkspaceStore::open(path, source.as_bytes())),
            history: WorkspaceHistory::default(),
        }))
    }

    /// Returns whether a successfully saved document change can be undone.
    pub fn can_undo(&self) -> bool {
        !self.history.undo.is_empty()
    }

    /// Returns whether an undone document change can be redone.
    pub fn can_redo(&self) -> bool {
        !self.history.redo.is_empty()
    }

    /// Returns the number of retained successful document changes.
    pub fn undo_depth(&self) -> usize {
        self.history.undo.len()
    }

    /// Returns the number of retained undone document changes.
    pub fn redo_depth(&self) -> usize {
        self.history.redo.len()
    }

    /// Validates a candidate document and builds its active-Lens dataset.
    pub fn stage_document_mutation(
        &self,
        document: WorkspaceDocument,
        selection_ids: Vec<u32>,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        self.stage_document_mutation_with_active_scope(
            document,
            selection_ids,
            self.active_version_id.clone(),
            self.active_lens_id.clone(),
        )
    }

    /// Stages a persisted document mutation and selects its resulting version
    /// and Lens. Structural authoring actions use this when creating or
    /// deleting a version or Lens changes the active canvas scope.
    pub fn stage_document_mutation_with_active_scope(
        &self,
        document: WorkspaceDocument,
        selection_ids: Vec<u32>,
        active_version_id: String,
        active_lens_id: String,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let mut staged = self.stage_workspace_mutation(
            document,
            selection_ids,
            active_version_id,
            active_lens_id,
            true,
        )?;
        staged.history_action = HistoryAction::Record;
        Ok(staged)
    }

    /// Stages restoration of the most recent successfully saved document change.
    pub fn stage_undo(&self) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let state = self
            .history
            .undo
            .last()
            .ok_or(WorkspaceStageError::NoUndo)?;
        let mut staged = self.stage_workspace_mutation(
            state.document.clone(),
            state.selection_ids.clone(),
            state.active_version_id.clone(),
            state.active_lens_id.clone(),
            true,
        )?;
        staged.history_action = HistoryAction::Undo;
        Ok(staged)
    }

    /// Stages restoration of the next undone document change.
    pub fn stage_redo(&self) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        let state = self
            .history
            .redo
            .last()
            .ok_or(WorkspaceStageError::NoRedo)?;
        let mut staged = self.stage_workspace_mutation(
            state.document.clone(),
            state.selection_ids.clone(),
            state.active_version_id.clone(),
            state.active_lens_id.clone(),
            true,
        )?;
        staged.history_action = HistoryAction::Redo;
        Ok(staged)
    }

    pub(crate) fn stage_workspace_mutation(
        &self,
        document: WorkspaceDocument,
        selection_ids: Vec<u32>,
        active_version_id: String,
        active_lens_id: String,
        persist_document: bool,
    ) -> Result<StagedWorkspaceMutation, WorkspaceStageError> {
        document
            .validate()
            .map_err(WorkspaceStageError::Validation)?;
        let dataset = document
            .canvas_dataset(&active_version_id, &active_lens_id)
            .map_err(WorkspaceStageError::Dataset)?;

        Ok(StagedWorkspaceMutation {
            document,
            dataset,
            selection_ids,
            active_version_id,
            active_lens_id,
            persist_document,
            history_action: HistoryAction::None,
        })
    }

    /// Persists a staged candidate before replacing the resident document state.
    pub fn commit(&mut self, staged: StagedWorkspaceMutation) -> Result<(), WorkspaceSaveError> {
        let StagedWorkspaceMutation {
            document,
            dataset,
            selection_ids,
            active_version_id,
            active_lens_id,
            persist_document,
            history_action,
        } = staged;
        let prior_state = self.snapshot();

        if persist_document {
            let source = document
                .to_pretty_json()
                .map_err(WorkspaceSaveError::Serialization)?;
            match &mut self.store {
                #[cfg(not(target_arch = "wasm32"))]
                WorkspaceStorage::Local(store) => {
                    store
                        .persist_candidate(&source, |persisted_source| {
                            WorkspaceDocument::parse_json(persisted_source)
                                .map(|_| ())
                                .map_err(|error| error.to_string())
                        })
                        .map_err(WorkspaceSaveError::Persistence)?;
                }
                #[cfg(target_arch = "wasm32")]
                WorkspaceStorage::BrowserMemory => {
                    let _ = source;
                }
            }
        }

        self.document = document;
        self.dataset = dataset;
        self.selection_ids = selection_ids;
        self.active_version_id = active_version_id;
        self.active_lens_id = active_lens_id;
        match history_action {
            HistoryAction::None => {}
            HistoryAction::Record if self.document != prior_state.document => {
                self.history.push_undo(prior_state);
                self.history.redo.clear();
            }
            HistoryAction::Record => {}
            HistoryAction::Undo => {
                self.history.undo.pop();
                self.history.push_redo(prior_state);
            }
            HistoryAction::Redo => {
                self.history.redo.pop();
                self.history.push_undo(prior_state);
            }
        }
        Ok(())
    }

    /// Verifies and saves the current workspace without changing its history.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&mut self) -> Result<(), WorkspaceSaveError> {
        let source = self
            .document
            .to_pretty_json()
            .map_err(WorkspaceSaveError::Serialization)?;
        match &mut self.store {
            WorkspaceStorage::Local(store) => {
                store
                    .persist_candidate(&source, |persisted_source| {
                        WorkspaceDocument::parse_json(persisted_source)
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    })
                    .map_err(WorkspaceSaveError::Persistence)?;
            }
        }
        Ok(())
    }

    fn snapshot(&self) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            document: self.document.clone(),
            selection_ids: self.selection_ids.clone(),
            active_version_id: self.active_version_id.clone(),
            active_lens_id: self.active_lens_id.clone(),
        }
    }
}

#[derive(Default)]
struct WorkspaceHistory {
    undo: Vec<WorkspaceSnapshot>,
    redo: Vec<WorkspaceSnapshot>,
}

impl WorkspaceHistory {
    fn push_undo(&mut self, state: WorkspaceSnapshot) {
        push_bounded(&mut self.undo, state);
    }

    fn push_redo(&mut self, state: WorkspaceSnapshot) {
        push_bounded(&mut self.redo, state);
    }
}

fn push_bounded(history: &mut Vec<WorkspaceSnapshot>, state: WorkspaceSnapshot) {
    if history.len() == WORKSPACE_HISTORY_LIMIT {
        history.remove(0);
    }
    history.push(state);
}

#[derive(Clone)]
struct WorkspaceSnapshot {
    document: WorkspaceDocument,
    selection_ids: Vec<u32>,
    active_version_id: String,
    active_lens_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HistoryAction {
    None,
    Record,
    Undo,
    Redo,
}

/// A document mutation that has passed in-memory validation but is not saved.
#[derive(Debug)]
pub struct StagedWorkspaceMutation {
    document: WorkspaceDocument,
    dataset: CanvasDataset,
    selection_ids: Vec<u32>,
    active_version_id: String,
    active_lens_id: String,
    persist_document: bool,
    history_action: HistoryAction,
}

impl StagedWorkspaceMutation {
    /// Returns the candidate workspace document.
    pub fn document(&self) -> &WorkspaceDocument {
        &self.document
    }

    /// Returns the candidate dataset for the selected version and Lens.
    pub fn dataset(&self) -> &CanvasDataset {
        &self.dataset
    }

    /// Returns the candidate selected concept identifiers.
    pub fn selection_ids(&self) -> &[u32] {
        &self.selection_ids
    }

    /// Returns the candidate active workspace version identifier.
    pub fn active_version_id(&self) -> &str {
        &self.active_version_id
    }

    /// Returns the candidate active Lens identifier.
    pub fn active_lens_id(&self) -> &str {
        &self.active_lens_id
    }
}

/// A local workspace load failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceLoadError {
    path: PathBuf,
    message: String,
}

impl WorkspaceLoadError {
    /// Returns the local workspace path that could not be loaded.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for WorkspaceLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path.display(), self.message)
    }
}

impl std::error::Error for WorkspaceLoadError {}

/// A failure while creating the first saved workspace document.
#[derive(Debug)]
pub enum WorkspaceCreateError {
    Validation(WorkspaceError),
    Dataset(WorkspaceError),
    Serialization(WorkspaceError),
    #[cfg(not(target_arch = "wasm32"))]
    Persistence(LocalCreateError),
}

impl fmt::Display for WorkspaceCreateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => write!(
                formatter,
                "could not validate new workspace before saving: {error}"
            ),
            Self::Dataset(error) => {
                write!(formatter, "could not open the initial Canvas Lens: {error}")
            }
            Self::Serialization(error) => {
                write!(formatter, "could not serialize new workspace: {error}")
            }
            #[cfg(not(target_arch = "wasm32"))]
            Self::Persistence(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for WorkspaceCreateError {}

/// An in-memory candidate validation failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceStageError {
    Mutation(WorkspaceError),
    Validation(WorkspaceError),
    Dataset(WorkspaceError),
    NoUndo,
    NoRedo,
}

impl fmt::Display for WorkspaceStageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mutation(error) => {
                write!(formatter, "could not apply candidate mutation: {error}")
            }
            Self::Validation(error) => {
                write!(formatter, "could not validate candidate workspace: {error}")
            }
            Self::Dataset(error) => write!(formatter, "could not build candidate dataset: {error}"),
            Self::NoUndo => {
                formatter.write_str("no successful document change is available to undo")
            }
            Self::NoRedo => formatter.write_str("no undone document change is available to redo"),
        }
    }
}

impl std::error::Error for WorkspaceStageError {}

/// A candidate serialization or local persistence failure.
#[derive(Debug)]
pub enum WorkspaceSaveError {
    Serialization(WorkspaceError),
    #[cfg(not(target_arch = "wasm32"))]
    Persistence(LocalSaveError),
}

impl fmt::Display for WorkspaceSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(error) => write!(
                formatter,
                "could not serialize candidate workspace: {error}"
            ),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Persistence(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for WorkspaceSaveError {}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use infinite_note_canvas_canvas::WorkspaceDocument;

    use crate::{LocalSaveError, LocalWorkspacePath, SourceDigest, WorkspaceSaveError};

    use super::{
        WORKSPACE_HISTORY_LIMIT, WorkspaceHistory, WorkspaceSession, WorkspaceSnapshot,
        WorkspaceStageError,
    };

    static NEXT_TEMPORARY_PATH: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn successful_commit_persists_validated_candidate_before_replacing_resident_state() {
        let workspace = TemporaryWorkspace::new("successful-commit", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let original_source = workspace.read_source();
        let original_digest = session.expected_digest();
        let mut candidate = session.document().clone();
        candidate.title = "Committed workspace title".to_string();
        let staged = session
            .stage_document_mutation(candidate.clone(), vec![2, 5])
            .expect("candidate must stage");

        session.commit(staged).expect("candidate must commit");

        let persisted_source = workspace.read_source();
        assert_eq!(
            WorkspaceDocument::parse_json(&persisted_source).unwrap(),
            candidate
        );
        assert_eq!(workspace.read_backup(), original_source);
        assert_ne!(session.expected_digest(), original_digest);
        assert_eq!(
            session.expected_digest(),
            Some(SourceDigest::from_bytes(persisted_source.as_bytes()))
        );
        assert_eq!(session.document().title, "Committed workspace title");
        assert_eq!(session.dataset().card_count(), 9);
        assert_eq!(session.active_version_id(), "current");
        assert_eq!(session.active_lens_id(), "organization-landscape");
        assert_eq!(session.selection_ids(), [2, 5]);
    }

    #[test]
    fn create_new_persists_and_opens_a_valid_empty_workspace() {
        let workspace = TemporaryWorkspace::new_uncreated("create-new");
        let document =
            WorkspaceDocument::blank("empty-canvas".to_string(), "Empty canvas".to_string())
                .unwrap();

        let session = WorkspaceSession::create_new(workspace.path.clone(), document.clone())
            .expect("new workspace must be saved and opened");

        assert_eq!(session.document(), &document);
        assert_eq!(session.workspace_path(), Some(workspace.path.as_path()));
        assert_eq!(session.active_version_id(), "current");
        assert_eq!(session.active_lens_id(), "canvas");
        assert_eq!(session.dataset().card_count(), 0);
        assert_eq!(workspace.read_source(), document.to_pretty_json().unwrap());
        assert_eq!(
            session.expected_digest(),
            Some(SourceDigest::from_bytes(workspace.read_source().as_bytes()))
        );
    }

    #[test]
    fn create_new_never_replaces_an_existing_workspace() {
        let workspace = TemporaryWorkspace::new("create-existing", "existing source");
        let document =
            WorkspaceDocument::blank("empty-canvas".to_string(), "Empty canvas".to_string())
                .unwrap();

        let error = match WorkspaceSession::create_new(workspace.path.clone(), document) {
            Ok(_) => panic!("existing workspace must remain untouched"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("already exists"));
        assert_eq!(workspace.read_source(), "existing source");
    }

    #[test]
    fn load_default_selects_the_current_state_and_first_saved_lens() {
        let document =
            WorkspaceDocument::blank("empty-canvas".to_string(), "Empty canvas".to_string())
                .unwrap();
        let workspace = TemporaryWorkspace::new(
            "load-default",
            &document
                .to_pretty_json()
                .expect("blank workspace serializes"),
        );

        let session = WorkspaceSession::load_default(workspace.path.clone())
            .expect("blank workspace must load into its default view");

        assert_eq!(session.active_version_id(), "current");
        assert_eq!(session.active_lens_id(), "canvas");
        assert_eq!(session.document(), &document);
    }

    #[test]
    fn explicit_save_verifies_the_current_workspace_without_changing_history() {
        let workspace = TemporaryWorkspace::new("explicit-save", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source = workspace.read_source();
        let digest = session.expected_digest();

        session.save().expect("current workspace must save");

        assert_eq!(workspace.read_source(), source);
        assert_eq!(workspace.read_backup(), source);
        assert_eq!(session.expected_digest(), digest);
        assert_eq!(session.undo_depth(), 0);
        assert_eq!(session.redo_depth(), 0);
    }

    #[test]
    fn explicit_save_rejects_an_external_workspace_change() {
        let workspace = TemporaryWorkspace::new("explicit-save-external-change", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        fs::write(&workspace.path, "outside change").expect("external workspace change");

        let error = session
            .save()
            .expect_err("external change must reject save");

        assert!(matches!(
            error,
            WorkspaceSaveError::Persistence(LocalSaveError::ExternalChange { .. })
        ));
        assert_eq!(workspace.read_source(), "outside change");
    }

    #[test]
    fn reload_if_local_source_changed_uses_a_valid_external_document() {
        let workspace = TemporaryWorkspace::new("external-refresh", &sample_source());
        let session = load_sample_session(workspace.path.clone());

        assert!(
            session
                .reload_if_local_source_changed()
                .expect("unchanged source must be readable")
                .is_none()
        );

        let mut external_document = session.document().clone();
        external_document.title = "MCP refreshed workspace".to_string();
        let external_source = external_document
            .to_pretty_json()
            .expect("external document must serialize");
        fs::write(&workspace.path, &external_source).expect("external workspace write");

        let refreshed = session
            .reload_if_local_source_changed()
            .expect("valid external source must refresh")
            .expect("changed source must create a refreshed session");

        assert_eq!(session.document().title, "Acme Platform Landscape");
        assert_eq!(refreshed.document().title, "MCP refreshed workspace");
        assert_eq!(refreshed.active_version_id(), session.active_version_id());
        assert_eq!(refreshed.active_lens_id(), session.active_lens_id());
        assert!(refreshed.selection_ids().is_empty());
        assert!(!refreshed.can_undo());
        assert!(!refreshed.can_redo());
        assert_eq!(
            refreshed.expected_digest(),
            Some(SourceDigest::from_bytes(external_source.as_bytes()))
        );
    }

    #[test]
    fn invalid_external_source_does_not_replace_the_resident_session() {
        let workspace = TemporaryWorkspace::new("invalid-external-refresh", &sample_source());
        let session = load_sample_session(workspace.path.clone());
        let document_before = session.document().clone();
        let digest_before = session.expected_digest();
        fs::write(&workspace.path, "invalid external workspace").expect("external workspace write");

        let error = match session.reload_if_local_source_changed() {
            Ok(_) => panic!("invalid external source must be rejected"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("could not validate workspace document")
        );
        assert_eq!(session.document(), &document_before);
        assert_eq!(session.expected_digest(), digest_before);
    }

    #[test]
    fn rejected_candidate_keeps_resident_state_and_digest_unchanged() {
        let workspace = TemporaryWorkspace::new("rejected-candidate", &sample_source());
        let session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let digest_before = session.expected_digest();
        let document_before = session.document().clone();
        let card_count_before = session.dataset().card_count();
        let mut invalid_candidate = session.document().clone();
        invalid_candidate.schema_version = u32::MAX;

        let error = session
            .stage_document_mutation(invalid_candidate, vec![1])
            .expect_err("invalid candidate must be rejected");

        assert!(matches!(error, WorkspaceStageError::Validation(_)));
        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(session.document(), &document_before);
        assert_eq!(session.dataset().card_count(), card_count_before);
        assert!(session.selection_ids().is_empty());
        assert_eq!(session.expected_digest(), digest_before);
    }

    #[test]
    fn save_failure_keeps_resident_state_selection_and_digest_unchanged() {
        let workspace = TemporaryWorkspace::new("save-failure", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let source_before = workspace.read_source();
        let digest_before = session.expected_digest();
        let document_before = session.document().clone();
        let card_count_before = session.dataset().card_count();
        let version_before = session.active_version_id().to_owned();
        let lens_before = session.active_lens_id().to_owned();
        let mut candidate = session.document().clone();
        candidate.title = "Unsaved title".to_string();
        let staged = session
            .stage_document_mutation(candidate, vec![1, 8])
            .expect("candidate must stage");
        fs::create_dir(workspace.temporary_path()).expect("temporary path directory setup");

        let error = session
            .commit(staged)
            .expect_err("candidate save must fail");

        assert!(matches!(
            error,
            WorkspaceSaveError::Persistence(LocalSaveError::CandidateWrite { .. })
        ));
        assert_eq!(workspace.read_source(), source_before);
        assert_eq!(session.document(), &document_before);
        assert_eq!(session.dataset().card_count(), card_count_before);
        assert_eq!(session.active_version_id(), version_before);
        assert_eq!(session.active_lens_id(), lens_before);
        assert!(session.selection_ids().is_empty());
        assert_eq!(session.expected_digest(), digest_before);
    }

    #[test]
    fn history_records_only_successful_saves_and_restores_them_with_undo_and_redo() {
        let workspace = TemporaryWorkspace::new("history", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());
        let original_source = workspace.read_source();
        let mut candidate = session.document().clone();
        candidate.title = "Saved title".to_string();
        let staged = session
            .stage_document_mutation(candidate.clone(), vec![5])
            .expect("candidate must stage");
        fs::create_dir(workspace.temporary_path()).expect("temporary path directory setup");

        session
            .commit(staged)
            .expect_err("candidate save must fail");

        assert!(!session.can_undo());
        assert!(!session.can_redo());
        assert_eq!(session.undo_depth(), 0);
        assert_eq!(session.redo_depth(), 0);
        assert_eq!(workspace.read_source(), original_source);

        fs::remove_dir(workspace.temporary_path()).expect("temporary path directory cleanup");
        let staged = session
            .stage_document_mutation(candidate, vec![5])
            .expect("candidate must stage after save failure");
        session.commit(staged).expect("candidate save must succeed");

        assert!(session.can_undo());
        assert!(!session.can_redo());
        assert_eq!(session.undo_depth(), 1);
        assert_eq!(session.document().title, "Saved title");

        let staged = session.stage_undo().expect("saved change must undo");
        session.commit(staged).expect("undo must save");
        assert_eq!(workspace.read_source(), original_source);
        assert!(!session.can_undo());
        assert!(session.can_redo());
        assert_eq!(session.redo_depth(), 1);

        let staged = session.stage_redo().expect("undone change must redo");
        session.commit(staged).expect("redo must save");
        assert_eq!(session.document().title, "Saved title");
        assert!(session.can_undo());
        assert!(!session.can_redo());
    }

    #[test]
    fn cancelled_and_failed_divergent_edits_preserve_redo_until_a_save_succeeds() {
        let workspace = TemporaryWorkspace::new("history-divergence", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());

        let mut first_document = session.document().clone();
        first_document.title = "First saved title".to_string();
        let staged = session
            .stage_document_mutation(first_document, vec![5])
            .expect("first edit must stage");
        session.commit(staged).expect("first edit must save");

        let staged = session.stage_undo().expect("first edit must undo");
        session.commit(staged).expect("undo must save");
        assert_eq!(session.undo_depth(), 0);
        assert_eq!(session.redo_depth(), 1);

        let mut divergent_document = session.document().clone();
        divergent_document.title = "Divergent title".to_string();
        let cancelled_staged = session
            .stage_document_mutation(divergent_document.clone(), vec![2])
            .expect("divergent edit must stage");
        drop(cancelled_staged);
        assert_eq!(session.undo_depth(), 0);
        assert_eq!(session.redo_depth(), 1);

        let failed_staged = session
            .stage_document_mutation(divergent_document.clone(), vec![2])
            .expect("divergent edit must stage again");
        fs::create_dir(workspace.temporary_path()).expect("temporary path directory setup");
        session
            .commit(failed_staged)
            .expect_err("divergent edit must fail to save");
        assert_eq!(session.undo_depth(), 0);
        assert_eq!(session.redo_depth(), 1);
        assert_eq!(session.document().title, WorkspaceDocument::sample().title);

        fs::remove_dir(workspace.temporary_path()).expect("temporary path directory cleanup");
        let staged = session
            .stage_document_mutation(divergent_document, vec![2])
            .expect("divergent edit must stage after failure");
        session.commit(staged).expect("divergent edit must save");

        assert_eq!(session.undo_depth(), 1);
        assert_eq!(session.redo_depth(), 0);
        assert_eq!(session.document().title, "Divergent title");
    }

    #[test]
    fn session_history_evicts_only_the_oldest_snapshot_and_round_trips_retained_edits() {
        let workspace = TemporaryWorkspace::new("history-capacity", &sample_source());
        let mut session = load_sample_session(workspace.path.clone());

        for revision in 1..=WORKSPACE_HISTORY_LIMIT + 1 {
            let mut document = session.document().clone();
            document.title = format!("Revision {revision}");
            let staged = session
                .stage_document_mutation(document, vec![5])
                .expect("revision must stage");
            session.commit(staged).expect("revision must save");
        }

        assert_eq!(session.undo_depth(), WORKSPACE_HISTORY_LIMIT);
        assert_eq!(session.redo_depth(), 0);

        for _ in 0..WORKSPACE_HISTORY_LIMIT {
            let staged = session.stage_undo().expect("retained edit must undo");
            session.commit(staged).expect("undo must save");
        }

        assert_eq!(session.document().title, "Revision 1");
        assert_eq!(session.selection_ids(), [5]);
        assert!(!session.can_undo());
        assert_eq!(session.redo_depth(), WORKSPACE_HISTORY_LIMIT);

        for _ in 0..WORKSPACE_HISTORY_LIMIT {
            let staged = session.stage_redo().expect("retained edit must redo");
            session.commit(staged).expect("redo must save");
        }

        assert_eq!(
            session.document().title,
            format!("Revision {}", WORKSPACE_HISTORY_LIMIT + 1)
        );
        assert_eq!(session.selection_ids(), [5]);
        assert_eq!(session.undo_depth(), WORKSPACE_HISTORY_LIMIT);
        assert!(!session.can_redo());
    }

    #[test]
    fn history_retains_a_bounded_number_of_snapshots() {
        let mut history = WorkspaceHistory::default();
        assert_eq!(WORKSPACE_HISTORY_LIMIT, 64);

        for index in 0..=64 {
            let mut document = WorkspaceDocument::sample();
            document.title = format!("Workspace {index}");
            history.push_undo(WorkspaceSnapshot {
                document,
                selection_ids: Vec::new(),
                active_version_id: "current".to_string(),
                active_lens_id: "organization-landscape".to_string(),
            });
        }

        assert_eq!(history.undo.len(), WORKSPACE_HISTORY_LIMIT);
        assert_eq!(history.undo.first().unwrap().document.title, "Workspace 1");
        assert_eq!(history.undo.last().unwrap().document.title, "Workspace 64");
    }

    fn load_sample_session(path: PathBuf) -> WorkspaceSession {
        WorkspaceSession::load(path, "current", "organization-landscape")
            .expect("sample workspace must load")
    }

    fn sample_source() -> String {
        WorkspaceDocument::sample()
            .to_pretty_json()
            .expect("sample workspace must serialize")
    }

    struct TemporaryWorkspace {
        path: PathBuf,
    }

    impl TemporaryWorkspace {
        fn new(label: &str, source: &str) -> Self {
            let unique = NEXT_TEMPORARY_PATH.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "infinite-note-canvas-workspace-session-{label}-{}-{unique}.json",
                std::process::id()
            ));
            remove_workspace_files(&path);
            fs::write(&path, source).expect("temporary workspace write");
            Self { path }
        }

        fn new_uncreated(label: &str) -> Self {
            let unique = NEXT_TEMPORARY_PATH.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "infinite-note-canvas-workspace-session-{label}-{}-{unique}.json",
                std::process::id()
            ));
            remove_workspace_files(&path);
            Self { path }
        }

        fn read_source(&self) -> String {
            fs::read_to_string(&self.path).expect("temporary workspace read")
        }

        fn read_backup(&self) -> String {
            fs::read_to_string(LocalWorkspacePath::new(self.path.clone()).backup_path())
                .expect("temporary workspace backup read")
        }

        fn temporary_path(&self) -> PathBuf {
            LocalWorkspacePath::new(self.path.clone()).temporary_path()
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
            LocalWorkspacePath::new(path.to_path_buf()).backup_path(),
            LocalWorkspacePath::new(path.to_path_buf()).temporary_path(),
        ] {
            if candidate.is_dir() {
                fs::remove_dir_all(candidate).expect("temporary workspace directory cleanup");
            } else if candidate.exists() {
                fs::remove_file(candidate).expect("temporary workspace file cleanup");
            }
        }
    }
}
