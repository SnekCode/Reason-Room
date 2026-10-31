use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::SourceDigest;

/// A local workspace path and the sibling paths used by candidate persistence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalWorkspacePath(PathBuf);

impl LocalWorkspacePath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn backup_path(&self) -> PathBuf {
        append_suffix(&self.0, ".bak")
    }

    pub fn temporary_path(&self) -> PathBuf {
        append_suffix(&self.0, ".tmp")
    }
}

impl From<PathBuf> for LocalWorkspacePath {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}

/// A verified source candidate that was safely persisted to local storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCandidate {
    pub source: String,
    pub digest: SourceDigest,
}

/// A failure while creating the first verified source for a new workspace.
///
/// New documents have no prior source to back up. Creation therefore uses an
/// exclusive file open, writes and syncs the complete candidate, then reads
/// and validates the exact persisted bytes before a session can open it.
#[derive(Debug)]
pub enum LocalCreateError {
    ParentDirectory {
        path: LocalWorkspacePath,
        detail: String,
    },
    AlreadyExists {
        path: LocalWorkspacePath,
    },
    Create {
        path: LocalWorkspacePath,
        detail: String,
    },
    Write {
        path: LocalWorkspacePath,
        detail: String,
    },
    Sync {
        path: LocalWorkspacePath,
        detail: String,
    },
    VerificationRead {
        path: LocalWorkspacePath,
        detail: String,
    },
    PersistedBytesMismatch {
        path: LocalWorkspacePath,
    },
    DocumentValidation {
        path: LocalWorkspacePath,
        detail: String,
    },
}

impl fmt::Display for LocalCreateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParentDirectory { path, detail } => write!(
                formatter,
                "could not create the folder for new workspace {}: {detail}",
                path.as_path().display()
            ),
            Self::AlreadyExists { path } => write!(
                formatter,
                "a workspace already exists at {}. Choose a different save location",
                path.as_path().display()
            ),
            Self::Create { path, detail } => write!(
                formatter,
                "could not create new workspace {}: {detail}",
                path.as_path().display()
            ),
            Self::Write { path, detail } => write!(
                formatter,
                "could not write new workspace {}: {detail}",
                path.as_path().display()
            ),
            Self::Sync { path, detail } => write!(
                formatter,
                "could not finish saving new workspace {}: {detail}",
                path.as_path().display()
            ),
            Self::VerificationRead { path, detail } => write!(
                formatter,
                "could not verify new workspace {} after saving: {detail}",
                path.as_path().display()
            ),
            Self::PersistedBytesMismatch { path } => write!(
                formatter,
                "could not verify new workspace {} after saving: persisted bytes do not match the candidate source",
                path.as_path().display()
            ),
            Self::DocumentValidation { path, detail } => write!(
                formatter,
                "could not validate new workspace {} after saving: {detail}",
                path.as_path().display()
            ),
        }
    }
}

impl std::error::Error for LocalCreateError {}

/// A local candidate-save error with a stable category and actionable message.
#[derive(Debug)]
#[non_exhaustive]
pub enum LocalSaveError {
    ReadBeforeSave {
        path: LocalWorkspacePath,
        detail: String,
    },
    ExternalChange {
        path: LocalWorkspacePath,
        expected: SourceDigest,
        found: SourceDigest,
    },
    BackupWrite {
        path: LocalWorkspacePath,
        detail: String,
    },
    CandidateWrite {
        path: LocalWorkspacePath,
        detail: String,
    },
    Replace {
        workspace_path: LocalWorkspacePath,
        temporary_path: PathBuf,
        detail: String,
    },
    VerificationRead {
        path: LocalWorkspacePath,
        detail: String,
        restored: bool,
    },
    PersistedBytesMismatch {
        path: LocalWorkspacePath,
        restored: bool,
    },
    DocumentValidation {
        path: LocalWorkspacePath,
        detail: String,
        restored: bool,
    },
    RestorationWrite {
        workspace_path: LocalWorkspacePath,
        temporary_path: PathBuf,
        original: PostReplacementFailure,
        detail: String,
    },
    RestorationReplace {
        workspace_path: LocalWorkspacePath,
        temporary_path: PathBuf,
        original: PostReplacementFailure,
        detail: String,
    },
    RestorationRead {
        path: LocalWorkspacePath,
        original: PostReplacementFailure,
        detail: String,
    },
    RestorationBytesMismatch {
        path: LocalWorkspacePath,
        original: PostReplacementFailure,
    },
}

/// The post-replacement condition that triggered a restoration attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostReplacementFailure {
    VerificationRead { detail: String },
    PersistedBytesMismatch,
    DocumentValidation { detail: String },
}

impl LocalSaveError {
    /// Returns a stable, actionable message for a UI operation prefix.
    pub fn user_message(&self) -> String {
        match self {
            Self::ReadBeforeSave { path, detail } => format!(
                "could not read the local workspace {} before saving: {detail}. Check that the file is available, then reload it before saving again",
                path.as_path().display()
            ),
            Self::ExternalChange { path, .. } => format!(
                "the local workspace {} changed outside the app; reload it before saving again",
                path.as_path().display()
            ),
            Self::BackupWrite { path, detail } => format!(
                "could not write the local backup {}: {detail}. Check available disk space or folder permissions, then try saving again",
                path.as_path().display()
            ),
            Self::CandidateWrite { path, detail } => format!(
                "could not write the local save candidate {}: {detail}. Check available disk space or folder permissions, then try saving again",
                path.as_path().display()
            ),
            Self::Replace {
                workspace_path,
                temporary_path,
                detail,
            } => format!(
                "could not replace the local workspace {} with {}: {detail}. Leave the .bak backup in place, check folder permissions, then try saving again",
                workspace_path.as_path().display(),
                temporary_path.display()
            ),
            Self::VerificationRead {
                path,
                detail,
                restored,
            } => post_replacement_message(
                format!(
                    "could not verify the local workspace {} after saving: {detail}",
                    path.as_path().display()
                ),
                *restored,
                None,
            ),
            Self::PersistedBytesMismatch { path, restored } => post_replacement_message(
                format!(
                    "could not validate the local workspace {} after saving: persisted bytes do not match the candidate source",
                    path.as_path().display()
                ),
                *restored,
                None,
            ),
            Self::DocumentValidation {
                path,
                detail,
                restored,
            } => post_replacement_message(
                format!(
                    "could not validate the local workspace {} after saving: {detail}",
                    path.as_path().display()
                ),
                *restored,
                None,
            ),
            Self::RestorationWrite {
                workspace_path,
                temporary_path,
                original,
                detail,
            } => restoration_failed_message(
                workspace_path,
                original,
                format!(
                    "could not write {} for local restoration: {detail}. Check available disk space or folder permissions",
                    temporary_path.display()
                ),
            ),
            Self::RestorationReplace {
                workspace_path,
                temporary_path,
                original,
                detail,
            } => restoration_failed_message(
                workspace_path,
                original,
                format!(
                    "could not restore the local workspace {} from {}: {detail}. Leave the .bak backup in place",
                    workspace_path.as_path().display(),
                    temporary_path.display()
                ),
            ),
            Self::RestorationRead {
                path,
                original,
                detail,
            } => restoration_failed_message(
                path,
                original,
                format!(
                    "could not read the local workspace {} after restoration: {detail}. Leave the .bak backup in place",
                    path.as_path().display()
                ),
            ),
            Self::RestorationBytesMismatch { path, original } => restoration_failed_message(
                path,
                original,
                format!(
                    "restored local workspace {} does not match the previous workspace source. Leave the .bak backup in place",
                    path.as_path().display()
                ),
            ),
        }
    }

    pub fn category(&self) -> LocalSaveErrorCategory {
        match self {
            Self::ReadBeforeSave { .. } => LocalSaveErrorCategory::ReadBeforeSave,
            Self::ExternalChange { .. } => LocalSaveErrorCategory::ExternalChange,
            Self::BackupWrite { .. } => LocalSaveErrorCategory::BackupWrite,
            Self::CandidateWrite { .. } => LocalSaveErrorCategory::CandidateWrite,
            Self::Replace { .. } => LocalSaveErrorCategory::Replace,
            Self::VerificationRead { .. } => LocalSaveErrorCategory::VerificationRead,
            Self::PersistedBytesMismatch { .. } => LocalSaveErrorCategory::PersistedBytesMismatch,
            Self::DocumentValidation { .. } => LocalSaveErrorCategory::DocumentValidation,
            Self::RestorationWrite { .. } => LocalSaveErrorCategory::RestorationWrite,
            Self::RestorationReplace { .. } => LocalSaveErrorCategory::RestorationReplace,
            Self::RestorationRead { .. } => LocalSaveErrorCategory::RestorationRead,
            Self::RestorationBytesMismatch { .. } => {
                LocalSaveErrorCategory::RestorationBytesMismatch
            }
        }
    }
}

impl fmt::Display for LocalSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.user_message())
    }
}

impl std::error::Error for LocalSaveError {}

/// A stable classification for a local candidate-save error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LocalSaveErrorCategory {
    ReadBeforeSave,
    ExternalChange,
    BackupWrite,
    CandidateWrite,
    Replace,
    VerificationRead,
    PersistedBytesMismatch,
    DocumentValidation,
    RestorationWrite,
    RestorationReplace,
    RestorationRead,
    RestorationBytesMismatch,
}

/// Owns local candidate persistence state for one workspace file.
pub struct LocalWorkspaceStore {
    path: LocalWorkspacePath,
    expected_digest: Option<SourceDigest>,
    file_access: Box<dyn LocalFileAccess>,
}

impl LocalWorkspaceStore {
    /// Creates a workspace file without replacing an existing document.
    ///
    /// The exact persisted source is read and validated before the returned
    /// store accepts it as its initial expected digest.
    pub fn create_new<F>(
        path: PathBuf,
        candidate_source: &str,
        validate: F,
    ) -> Result<Self, LocalCreateError>
    where
        F: FnOnce(&str) -> Result<(), String>,
    {
        let workspace_path = LocalWorkspacePath::new(path);
        if let Some(parent) = workspace_path.as_path().parent() {
            fs::create_dir_all(parent).map_err(|error| LocalCreateError::ParentDirectory {
                path: workspace_path.clone(),
                detail: error.to_string(),
            })?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(workspace_path.as_path())
            .map_err(|error| {
                if error.kind() == io::ErrorKind::AlreadyExists {
                    LocalCreateError::AlreadyExists {
                        path: workspace_path.clone(),
                    }
                } else {
                    LocalCreateError::Create {
                        path: workspace_path.clone(),
                        detail: error.to_string(),
                    }
                }
            })?;
        file.write_all(candidate_source.as_bytes())
            .map_err(|error| LocalCreateError::Write {
                path: workspace_path.clone(),
                detail: error.to_string(),
            })?;
        file.sync_all().map_err(|error| LocalCreateError::Sync {
            path: workspace_path.clone(),
            detail: error.to_string(),
        })?;
        drop(file);

        let persisted_source = fs::read_to_string(workspace_path.as_path()).map_err(|error| {
            LocalCreateError::VerificationRead {
                path: workspace_path.clone(),
                detail: error.to_string(),
            }
        })?;
        if persisted_source != candidate_source {
            return Err(LocalCreateError::PersistedBytesMismatch {
                path: workspace_path,
            });
        }
        validate(&persisted_source).map_err(|detail| LocalCreateError::DocumentValidation {
            path: workspace_path.clone(),
            detail,
        })?;

        Ok(Self::open_at(workspace_path, persisted_source.as_bytes()))
    }

    pub fn open(path: PathBuf, source: &[u8]) -> Self {
        Self::open_at(LocalWorkspacePath::new(path), source)
    }

    pub fn open_at(path: LocalWorkspacePath, source: &[u8]) -> Self {
        Self {
            path,
            expected_digest: Some(SourceDigest::from_bytes(source)),
            file_access: Box::new(StdLocalFileAccess),
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn workspace_path(&self) -> &LocalWorkspacePath {
        &self.path
    }

    pub fn expected_digest(&self) -> Option<SourceDigest> {
        self.expected_digest
    }

    pub fn persist_candidate<F>(
        &mut self,
        candidate_source: &str,
        validate: F,
    ) -> Result<PersistedCandidate, LocalSaveError>
    where
        F: FnOnce(&str) -> Result<(), String>,
    {
        let previous_source = self
            .file_access
            .read(self.path.as_path())
            .map_err(|error| LocalSaveError::ReadBeforeSave {
                path: self.path.clone(),
                detail: error.to_string(),
            })?;

        let found_digest = SourceDigest::from_bytes(&previous_source);
        if let Some(expected_digest) = self.expected_digest
            && found_digest != expected_digest
        {
            return Err(LocalSaveError::ExternalChange {
                path: self.path.clone(),
                expected: expected_digest,
                found: found_digest,
            });
        }

        let backup_path = self.path.backup_path();
        self.file_access
            .write(&backup_path, &previous_source)
            .map_err(|error| LocalSaveError::BackupWrite {
                path: LocalWorkspacePath::new(backup_path),
                detail: error.to_string(),
            })?;

        let temporary_path = self.path.temporary_path();
        self.file_access
            .write(&temporary_path, candidate_source.as_bytes())
            .map_err(|error| LocalSaveError::CandidateWrite {
                path: LocalWorkspacePath::new(temporary_path.clone()),
                detail: error.to_string(),
            })?;
        self.file_access
            .rename(&temporary_path, self.path.as_path())
            .map_err(|error| LocalSaveError::Replace {
                workspace_path: self.path.clone(),
                temporary_path,
                detail: error.to_string(),
            })?;

        let persisted_source = match self.verify_persisted(candidate_source, validate) {
            Ok(persisted_source) => persisted_source,
            Err(original) => return Err(self.restore_after_failure(&previous_source, original)),
        };
        let digest = SourceDigest::from_bytes(persisted_source.as_bytes());
        self.expected_digest = Some(digest);
        Ok(PersistedCandidate {
            source: persisted_source,
            digest,
        })
    }

    fn verify_persisted<F>(
        &self,
        candidate_source: &str,
        validate: F,
    ) -> Result<String, PostReplacementFailure>
    where
        F: FnOnce(&str) -> Result<(), String>,
    {
        let persisted_bytes = self
            .file_access
            .read(self.path.as_path())
            .map_err(|error| PostReplacementFailure::VerificationRead {
                detail: error.to_string(),
            })?;
        let persisted_source = String::from_utf8(persisted_bytes).map_err(|error| {
            PostReplacementFailure::VerificationRead {
                detail: error.to_string(),
            }
        })?;
        if persisted_source != candidate_source {
            return Err(PostReplacementFailure::PersistedBytesMismatch);
        }
        validate(&persisted_source)
            .map_err(|detail| PostReplacementFailure::DocumentValidation { detail })?;
        Ok(persisted_source)
    }

    fn restore_after_failure(
        &self,
        previous_source: &[u8],
        original: PostReplacementFailure,
    ) -> LocalSaveError {
        let temporary_path = self.path.temporary_path();
        if let Err(error) = self.file_access.write(&temporary_path, previous_source) {
            return LocalSaveError::RestorationWrite {
                workspace_path: self.path.clone(),
                temporary_path,
                original,
                detail: error.to_string(),
            };
        }
        if let Err(error) = self
            .file_access
            .rename(&temporary_path, self.path.as_path())
        {
            return LocalSaveError::RestorationReplace {
                workspace_path: self.path.clone(),
                temporary_path,
                original,
                detail: error.to_string(),
            };
        }
        let restored_source = match self.file_access.read(self.path.as_path()) {
            Ok(source) => source,
            Err(error) => {
                return LocalSaveError::RestorationRead {
                    path: self.path.clone(),
                    original,
                    detail: error.to_string(),
                };
            }
        };
        if restored_source != previous_source {
            return LocalSaveError::RestorationBytesMismatch {
                path: self.path.clone(),
                original,
            };
        }
        original.into_restored_error(self.path.clone())
    }

    #[cfg(test)]
    fn with_file_access(
        path: LocalWorkspacePath,
        source: &[u8],
        file_access: Box<dyn LocalFileAccess>,
    ) -> Self {
        Self {
            path,
            expected_digest: Some(SourceDigest::from_bytes(source)),
            file_access,
        }
    }
}

impl PostReplacementFailure {
    fn into_restored_error(self, path: LocalWorkspacePath) -> LocalSaveError {
        match self {
            Self::VerificationRead { detail } => LocalSaveError::VerificationRead {
                path,
                detail,
                restored: true,
            },
            Self::PersistedBytesMismatch => LocalSaveError::PersistedBytesMismatch {
                path,
                restored: true,
            },
            Self::DocumentValidation { detail } => LocalSaveError::DocumentValidation {
                path,
                detail,
                restored: true,
            },
        }
    }

    fn user_message(&self, path: &LocalWorkspacePath) -> String {
        match self {
            Self::VerificationRead { detail } => format!(
                "could not verify the local workspace {} after saving: {detail}",
                path.as_path().display()
            ),
            Self::PersistedBytesMismatch => format!(
                "could not validate the local workspace {} after saving: persisted bytes do not match the candidate source",
                path.as_path().display()
            ),
            Self::DocumentValidation { detail } => format!(
                "could not validate the local workspace {} after saving: {detail}",
                path.as_path().display()
            ),
        }
    }
}

fn restoration_failed_message(
    workspace_path: &LocalWorkspacePath,
    original: &PostReplacementFailure,
    restoration_detail: String,
) -> String {
    post_replacement_message(
        original.user_message(workspace_path),
        false,
        Some(restoration_detail),
    )
}

fn post_replacement_message(
    initial_message: String,
    restored: bool,
    restoration_detail: Option<String>,
) -> String {
    if restored {
        format!(
            "{initial_message}; restored the previous workspace source. Reload the workspace before you save again"
        )
    } else if let Some(restoration_detail) = restoration_detail {
        format!(
            "{initial_message}; restoration failed: {restoration_detail}. The replacement may still be present. Do not save again; copy the workspace and its .bak backup to a safe local folder before you inspect them"
        )
    } else {
        initial_message
    }
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut suffixed = path.as_os_str().to_os_string();
    suffixed.push(suffix);
    PathBuf::from(suffixed)
}

trait LocalFileAccess {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write(&self, path: &Path, source: &[u8]) -> io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
}

struct StdLocalFileAccess;

impl LocalFileAccess for StdLocalFileAccess {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn write(&self, path: &Path, source: &[u8]) -> io::Result<()> {
        fs::write(path, source)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LocalFileAccess, LocalSaveError, LocalSaveErrorCategory, LocalWorkspacePath,
        LocalWorkspaceStore, SourceDigest,
    };
    use std::cell::RefCell;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEMPORARY_PATH: AtomicUsize = AtomicUsize::new(0);

    const PREVIOUS_SOURCE: &str = "previous workspace source";
    const CANDIDATE_SOURCE: &str = "candidate workspace source";

    #[test]
    fn successful_save_replaces_source_refreshes_backup_and_digest() {
        let temporary_workspace = TemporaryWorkspace::new("success", PREVIOUS_SOURCE);
        let mut store =
            LocalWorkspaceStore::open(temporary_workspace.path.clone(), PREVIOUS_SOURCE.as_bytes());

        let persisted = store
            .persist_candidate(CANDIDATE_SOURCE, |_| Ok(()))
            .expect("candidate must persist");

        assert_eq!(persisted.source, CANDIDATE_SOURCE);
        assert_eq!(
            persisted.digest,
            SourceDigest::from_bytes(CANDIDATE_SOURCE.as_bytes())
        );
        assert_eq!(store.expected_digest(), Some(persisted.digest));
        assert_eq!(temporary_workspace.read_workspace(), CANDIDATE_SOURCE);
        assert_eq!(temporary_workspace.read_backup(), PREVIOUS_SOURCE);
        assert!(!temporary_workspace.temporary_path().exists());
    }

    #[test]
    fn backup_write_failure_keeps_workspace_and_digest_unchanged() {
        let temporary_workspace = TemporaryWorkspace::new("backup-write", PREVIOUS_SOURCE);
        let mut store = fault_injected_store(&temporary_workspace, Fault::WriteCall(1));

        let error = store
            .persist_candidate(CANDIDATE_SOURCE, |_| Ok(()))
            .expect_err("backup write failure must reject the candidate");

        assert_error_category(&error, LocalSaveErrorCategory::BackupWrite);
        assert!(
            error
                .user_message()
                .contains("Check available disk space or folder permissions")
        );
        assert_eq!(temporary_workspace.read_workspace(), PREVIOUS_SOURCE);
        assert_eq!(
            store.expected_digest(),
            Some(SourceDigest::from_bytes(PREVIOUS_SOURCE.as_bytes()))
        );
        assert!(!temporary_workspace.backup_path().exists());
    }

    #[test]
    fn external_change_is_detected_before_backup_write() {
        let temporary_workspace = TemporaryWorkspace::new("external-change", PREVIOUS_SOURCE);
        let mut store =
            LocalWorkspaceStore::open(temporary_workspace.path.clone(), PREVIOUS_SOURCE.as_bytes());
        fs::write(&temporary_workspace.path, "changed outside the app")
            .expect("external source writes");

        let error = store
            .persist_candidate(CANDIDATE_SOURCE, |_| Ok(()))
            .expect_err("external changes must reject the candidate");

        assert_error_category(&error, LocalSaveErrorCategory::ExternalChange);
        assert!(
            error
                .user_message()
                .contains("reload it before saving again")
        );
        assert_eq!(
            temporary_workspace.read_workspace(),
            "changed outside the app"
        );
        assert!(!temporary_workspace.backup_path().exists());
        assert_eq!(
            store.expected_digest(),
            Some(SourceDigest::from_bytes(PREVIOUS_SOURCE.as_bytes()))
        );
    }

    #[test]
    fn candidate_write_failure_keeps_backup_and_workspace_source() {
        let temporary_workspace = TemporaryWorkspace::new("candidate-write", PREVIOUS_SOURCE);
        let mut store = fault_injected_store(&temporary_workspace, Fault::WriteCall(2));

        let error = store
            .persist_candidate(CANDIDATE_SOURCE, |_| Ok(()))
            .expect_err("candidate write failure must reject the candidate");

        assert_error_category(&error, LocalSaveErrorCategory::CandidateWrite);
        assert_eq!(temporary_workspace.read_workspace(), PREVIOUS_SOURCE);
        assert_eq!(temporary_workspace.read_backup(), PREVIOUS_SOURCE);
        assert_eq!(
            store.expected_digest(),
            Some(SourceDigest::from_bytes(PREVIOUS_SOURCE.as_bytes()))
        );
    }

    #[test]
    fn verification_read_failure_restores_previous_source() {
        let temporary_workspace = TemporaryWorkspace::new("verification-read", PREVIOUS_SOURCE);
        let mut store = fault_injected_store(&temporary_workspace, Fault::ReadCall(2));

        let error = store
            .persist_candidate(CANDIDATE_SOURCE, |_| Ok(()))
            .expect_err("verification read failure must reject the candidate");

        assert_error_category(&error, LocalSaveErrorCategory::VerificationRead);
        assert!(
            error
                .user_message()
                .contains("restored the previous workspace source")
        );
        assert_eq!(temporary_workspace.read_workspace(), PREVIOUS_SOURCE);
        assert_eq!(temporary_workspace.read_backup(), PREVIOUS_SOURCE);
    }

    #[test]
    fn persisted_bytes_mismatch_skips_validator_and_restores_previous_source() {
        let temporary_workspace = TemporaryWorkspace::new("verification-mismatch", PREVIOUS_SOURCE);
        let mut store = fault_injected_store(
            &temporary_workspace,
            Fault::ReadCallBytes(2, b"different persisted source".to_vec()),
        );
        let validator_called = RefCell::new(false);

        let error = store
            .persist_candidate(CANDIDATE_SOURCE, |_| {
                *validator_called.borrow_mut() = true;
                Ok(())
            })
            .expect_err("mismatched persisted bytes must reject the candidate");

        assert_error_category(&error, LocalSaveErrorCategory::PersistedBytesMismatch);
        assert!(!*validator_called.borrow());
        assert_eq!(temporary_workspace.read_workspace(), PREVIOUS_SOURCE);
        assert_eq!(temporary_workspace.read_backup(), PREVIOUS_SOURCE);
    }

    #[test]
    fn validation_failure_restores_previous_source() {
        let temporary_workspace = TemporaryWorkspace::new("validation", PREVIOUS_SOURCE);
        let mut store =
            LocalWorkspaceStore::open(temporary_workspace.path.clone(), PREVIOUS_SOURCE.as_bytes());

        let error = store
            .persist_candidate(CANDIDATE_SOURCE, |_| {
                Err("forced document validation failure".to_string())
            })
            .expect_err("validator failure must reject the candidate");

        assert_error_category(&error, LocalSaveErrorCategory::DocumentValidation);
        assert!(
            error
                .user_message()
                .contains("forced document validation failure")
        );
        assert!(
            error
                .user_message()
                .contains("restored the previous workspace source")
        );
        assert_eq!(temporary_workspace.read_workspace(), PREVIOUS_SOURCE);
        assert_eq!(temporary_workspace.read_backup(), PREVIOUS_SOURCE);
    }

    #[test]
    fn replacement_failure_keeps_backup_and_previous_source() {
        let temporary_workspace = TemporaryWorkspace::new("replace", PREVIOUS_SOURCE);
        let mut store = fault_injected_store(&temporary_workspace, Fault::RenameCall(1));

        let error = store
            .persist_candidate(CANDIDATE_SOURCE, |_| Ok(()))
            .expect_err("replace failure must reject the candidate");

        assert_error_category(&error, LocalSaveErrorCategory::Replace);
        assert_eq!(temporary_workspace.read_workspace(), PREVIOUS_SOURCE);
        assert_eq!(temporary_workspace.read_backup(), PREVIOUS_SOURCE);
    }

    #[test]
    fn restoration_write_failure_reports_safe_next_step() {
        assert_restoration_failure(
            Fault::WriteCall(3),
            LocalSaveErrorCategory::RestorationWrite,
        );
    }

    #[test]
    fn restoration_replace_failure_reports_safe_next_step() {
        assert_restoration_failure(
            Fault::RenameCall(2),
            LocalSaveErrorCategory::RestorationReplace,
        );
    }

    #[test]
    fn restoration_read_failure_reports_safe_next_step() {
        assert_restoration_failure(Fault::ReadCall(3), LocalSaveErrorCategory::RestorationRead);
    }

    #[test]
    fn restoration_byte_mismatch_reports_safe_next_step() {
        assert_restoration_failure(
            Fault::ReadCallBytes(3, b"incorrect restored source".to_vec()),
            LocalSaveErrorCategory::RestorationBytesMismatch,
        );
    }

    fn assert_restoration_failure(fault: Fault, expected_category: LocalSaveErrorCategory) {
        let temporary_workspace = TemporaryWorkspace::new("restoration", PREVIOUS_SOURCE);
        let mut store = fault_injected_store(&temporary_workspace, fault);

        let error = store
            .persist_candidate(CANDIDATE_SOURCE, |_| {
                Err("forced validation failure".to_string())
            })
            .expect_err("restoration failure must reject the candidate");

        assert_error_category(&error, expected_category);
        assert!(error.user_message().contains("Do not save again"));
        assert_eq!(
            store.expected_digest(),
            Some(SourceDigest::from_bytes(PREVIOUS_SOURCE.as_bytes()))
        );
        assert_eq!(temporary_workspace.read_backup(), PREVIOUS_SOURCE);
    }

    fn assert_error_category(error: &LocalSaveError, expected: LocalSaveErrorCategory) {
        assert_eq!(error.category(), expected, "unexpected error: {error}");
    }

    fn fault_injected_store(
        temporary_workspace: &TemporaryWorkspace,
        fault: Fault,
    ) -> LocalWorkspaceStore {
        LocalWorkspaceStore::with_file_access(
            LocalWorkspacePath::new(temporary_workspace.path.clone()),
            PREVIOUS_SOURCE.as_bytes(),
            Box::new(FaultInjectingFileAccess::new(fault)),
        )
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
            fs::write(&path, source).expect("temporary workspace writes");
            Self { path }
        }

        fn backup_path(&self) -> PathBuf {
            LocalWorkspacePath::new(self.path.clone()).backup_path()
        }

        fn temporary_path(&self) -> PathBuf {
            LocalWorkspacePath::new(self.path.clone()).temporary_path()
        }

        fn read_workspace(&self) -> String {
            fs::read_to_string(&self.path).expect("workspace source reads")
        }

        fn read_backup(&self) -> String {
            fs::read_to_string(self.backup_path()).expect("backup source reads")
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

    enum Fault {
        ReadCall(usize),
        ReadCallBytes(usize, Vec<u8>),
        WriteCall(usize),
        RenameCall(usize),
    }

    struct FaultInjectingFileAccess {
        fault: Fault,
        read_calls: RefCell<usize>,
        write_calls: RefCell<usize>,
        rename_calls: RefCell<usize>,
    }

    impl FaultInjectingFileAccess {
        fn new(fault: Fault) -> Self {
            Self {
                fault,
                read_calls: RefCell::new(0),
                write_calls: RefCell::new(0),
                rename_calls: RefCell::new(0),
            }
        }

        fn failure() -> io::Error {
            io::Error::other("injected local file failure")
        }
    }

    impl LocalFileAccess for FaultInjectingFileAccess {
        fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
            let call = increment_call(&self.read_calls);
            match &self.fault {
                Fault::ReadCall(expected_call) if call == *expected_call => Err(Self::failure()),
                Fault::ReadCallBytes(expected_call, source) if call == *expected_call => {
                    Ok(source.clone())
                }
                _ => fs::read(path),
            }
        }

        fn write(&self, path: &Path, source: &[u8]) -> io::Result<()> {
            let call = increment_call(&self.write_calls);
            if matches!(&self.fault, Fault::WriteCall(expected_call) if call == *expected_call) {
                return Err(Self::failure());
            }
            fs::write(path, source)
        }

        fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
            let call = increment_call(&self.rename_calls);
            if matches!(&self.fault, Fault::RenameCall(expected_call) if call == *expected_call) {
                return Err(Self::failure());
            }
            fs::rename(from, to)
        }
    }

    fn increment_call(counter: &RefCell<usize>) -> usize {
        let mut calls = counter.borrow_mut();
        *calls += 1;
        *calls
    }
}
