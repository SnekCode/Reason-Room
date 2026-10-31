#[cfg(not(target_arch = "wasm32"))]
use std::{env, fs, path::PathBuf};

use infinite_note_canvas_canvas::WorkspaceDocument;

pub(crate) use infinite_note_canvas_workspace_session::WorkspaceSession;

#[cfg(not(target_arch = "wasm32"))]
const WORKSPACE_ENVIRONMENT_VARIABLE: &str = "INFINITE_NOTE_CANVAS_WORKSPACE";
#[cfg(not(target_arch = "wasm32"))]
const FALLBACK_WORKSPACE_ENVIRONMENT_VARIABLE: &str = "INFINITE_NOTE_CANVAS_FALLBACK_WORKSPACE";
#[cfg(not(target_arch = "wasm32"))]
const REPOSITORY_WORKSPACE_PATH: &str = "workspaces/acme.infinite-note.json";
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_WORKSPACE_FILE_NAME: &str = "acme.infinite-note.json";

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn load_from_environment() -> Result<WorkspaceSession, String> {
    let path = configured_workspace_path()?;
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("{}: could not read: {error}", path.display()))?;
    let document = WorkspaceDocument::parse_json(&source)
        .map_err(|error| format!("{}: could not validate: {error}", path.display()))?;
    let active_version_id = env::var("INFINITE_NOTE_CANVAS_VERSION")
        .ok()
        .or_else(|| env::var("INFINITE_NOTE_CANVAS_SCENARIO").ok())
        .unwrap_or_else(|| document.current_state_id().to_owned());

    document.version(&active_version_id).map_err(|error| {
        format!(
            "{}: could not select version {active_version_id}: {error}",
            path.display()
        )
    })?;

    let active_lens_id = env::var("INFINITE_NOTE_CANVAS_LENS")
        .ok()
        .or_else(|| env::var("INFINITE_NOTE_CANVAS_VIEW").ok())
        .map_or_else(
            || {
                document
                    .default_lens_id(&active_version_id)
                    .map(str::to_owned)
                    .map_err(|error| {
                        format!("{}: workspace has no saved lenses: {error}", path.display())
                    })
            },
            Ok,
        )?;

    WorkspaceSession::load(path, active_version_id, active_lens_id)
        .map_err(|error| error.to_string())
}

/// An explicit workspace is a user-directed launch choice. A saved recent
/// document must not replace it.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn has_explicit_workspace_override() -> bool {
    env::var_os(WORKSPACE_ENVIRONMENT_VARIABLE).is_some()
}

/// Browser sessions start with the bundled fixture for the current page only.
#[cfg(target_arch = "wasm32")]
pub(crate) fn load_from_environment() -> Result<WorkspaceSession, String> {
    let fallback_document = WorkspaceDocument::parse_json(include_str!(
        "../../../workspaces/gpui-evaluation.fixture.json"
    ))
    .map_err(|error| format!("could not validate bundled browser workspace: {error}"))?;

    WorkspaceSession::open_browser(fallback_document).map_err(|error| error.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn configured_workspace_path() -> Result<PathBuf, String> {
    let repository_workspace_path = PathBuf::from(REPOSITORY_WORKSPACE_PATH);
    select_workspace_path(
        env::var_os(WORKSPACE_ENVIRONMENT_VARIABLE).map(PathBuf::from),
        env::var_os(FALLBACK_WORKSPACE_ENVIRONMENT_VARIABLE).map(PathBuf::from),
        repository_workspace_path.clone(),
        repository_workspace_path.exists(),
        env::var_os("HOME").map(PathBuf::from),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn select_workspace_path(
    configured_path: Option<PathBuf>,
    fallback_path: Option<PathBuf>,
    repository_workspace_path: PathBuf,
    repository_workspace_exists: bool,
    home_directory: Option<PathBuf>,
) -> Result<PathBuf, String> {
    if let Some(path) = configured_path {
        return Ok(path);
    }

    if let Some(path) = fallback_path {
        return Ok(path);
    }

    if repository_workspace_exists {
        return Ok(repository_workspace_path);
    }

    let home_directory = home_directory.ok_or_else(|| {
        "HOME is not set and no workspace path was provided. Set INFINITE_NOTE_CANVAS_WORKSPACE."
            .to_string()
    })?;
    Ok(home_directory
        .join("Library")
        .join("Application Support")
        .join("Infinite Note Canvas")
        .join(DEFAULT_WORKSPACE_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_workspace_path_has_priority() {
        let configured_path = PathBuf::from("selected.infinite-note.json");

        let selected_path = select_workspace_path(
            Some(configured_path.clone()),
            Some(PathBuf::from("fallback.infinite-note.json")),
            PathBuf::from("workspaces/acme.infinite-note.json"),
            true,
            Some(PathBuf::from("/Users/tester")),
        )
        .unwrap();

        assert_eq!(selected_path, configured_path);
    }

    #[test]
    fn repository_workspace_is_the_development_default() {
        let repository_workspace_path = PathBuf::from("workspaces/acme.infinite-note.json");

        let selected_path = select_workspace_path(
            None,
            None,
            repository_workspace_path.clone(),
            true,
            Some(PathBuf::from("/Users/tester")),
        )
        .unwrap();

        assert_eq!(selected_path, repository_workspace_path);
    }

    #[test]
    fn application_support_workspace_is_the_local_default() {
        let selected_path = select_workspace_path(
            None,
            None,
            PathBuf::from("workspaces/acme.infinite-note.json"),
            false,
            Some(PathBuf::from("/Users/tester")),
        )
        .unwrap();

        assert_eq!(
            selected_path,
            PathBuf::from(
                "/Users/tester/Library/Application Support/Infinite Note Canvas/acme.infinite-note.json"
            )
        );
    }

    #[test]
    fn fallback_workspace_precedes_the_repository_default() {
        let fallback_path = PathBuf::from("/Users/tester/Canvas/recent.infinite-note.json");

        let selected_path = select_workspace_path(
            None,
            Some(fallback_path.clone()),
            PathBuf::from("workspaces/acme.infinite-note.json"),
            true,
            Some(PathBuf::from("/Users/tester")),
        )
        .unwrap();

        assert_eq!(selected_path, fallback_path);
    }
}
