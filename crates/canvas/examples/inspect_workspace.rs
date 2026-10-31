use std::{env, fs, process::ExitCode};

use infinite_note_canvas_canvas::WorkspaceDocument;

const DEFAULT_WORKSPACE_PATH: &str = "workspaces/static-react-website.infinite-note.json";

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);
    let workspace_path = arguments
        .next()
        .unwrap_or_else(|| DEFAULT_WORKSPACE_PATH.to_string());
    let source = match fs::read_to_string(&workspace_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not read {workspace_path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let workspace = match WorkspaceDocument::parse_json(&source) {
        Ok(workspace) => workspace,
        Err(error) => {
            eprintln!("Could not validate {workspace_path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let version_id = arguments
        .next()
        .unwrap_or_else(|| workspace.current_state_id().to_string());
    let lens_id = match arguments.next() {
        Some(lens_id) => lens_id,
        None => match workspace.default_lens_id(&version_id) {
            Ok(lens_id) => lens_id.to_string(),
            Err(error) => {
                eprintln!("Workspace has no usable lens: {error}");
                return ExitCode::FAILURE;
            }
        },
    };
    let version = match workspace.version(&version_id) {
        Ok(version) => version,
        Err(error) => {
            eprintln!("Could not select workspace version {version_id}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let dataset = match workspace.canvas_dataset(&version_id, &lens_id) {
        Ok(dataset) => dataset,
        Err(error) => {
            eprintln!("Could not build canvas records from {workspace_path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "Workspace: {} ({})",
        workspace.title, workspace.workspace_id
    );
    println!("Schema version: {}", workspace.schema_version);
    println!("Current state: {}", workspace.current_state_id());
    println!("Available versions: {}", workspace.version_ids().join(", "));
    println!("Selected version: {} ({})", version.id, version.title);
    println!("Concepts: {}", version.concepts.len());
    println!("Typed relationships: {}", version.relationships.len());
    println!("Custom types: {}", workspace.custom_types.len());
    println!("Lenses: {}", version.lenses.len());
    println!(
        "Selected lens: {lens_id} ({} placed concepts, {} visible relationships)",
        dataset.card_count(),
        dataset.relationship_count()
    );
    ExitCode::SUCCESS
}
