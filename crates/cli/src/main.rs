mod mcp;

use std::{collections::BTreeSet, env, fs, process::ExitCode};

use infinite_note_canvas_canvas::{RelationshipKind, WorkspaceDocument, WorkspaceVersion};
use serde::Serialize;

const CONTRACT_VERSION: u32 = 1;

fn main() -> ExitCode {
    let arguments: Vec<_> = env::args().skip(1).collect();
    if arguments.first().is_some_and(|argument| argument == "mcp") {
        return match mcp::run(&arguments[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        };
    }

    match run(arguments) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            if error.is_invalid_workspace() {
                ExitCode::from(2)
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

fn run(arguments: Vec<String>) -> Result<String, CliError> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(CliError::Usage(usage()));
    };
    if matches!(command, "help" | "--help" | "-h") {
        return Ok(usage());
    }
    let options = Options::parse(&arguments[1..])?;
    let workspace_path = options
        .workspace_path
        .as_deref()
        .ok_or_else(|| CliError::Usage("--workspace PATH is required".to_string()))?;
    let source = fs::read_to_string(workspace_path).map_err(|error| {
        CliError::Operational(format!("could not read {workspace_path}: {error}"))
    })?;
    let workspace = WorkspaceDocument::parse_json(&source).map_err(|error| {
        CliError::InvalidWorkspace(format!("could not validate {workspace_path}: {error}"))
    })?;
    let version_id = options
        .version_id
        .as_deref()
        .unwrap_or_else(|| workspace.current_state_id());
    let version = workspace
        .version(version_id)
        .map_err(|error| CliError::Operational(error.to_string()))?;
    let digest = format!("fnv1a64:{:016x}", fnv1a64(source.as_bytes()));

    match command {
        "inspect" => json(&InspectOutput {
            contract_version: CONTRACT_VERSION,
            schema_version: workspace.schema_version,
            workspace_id: &workspace.workspace_id,
            workspace_title: &workspace.title,
            workspace_digest: &digest,
            current_state_id: workspace.current_state_id(),
            selected_version: VersionSummary::from(version),
            version_ids: workspace.version_ids(),
            custom_type_count: workspace.custom_types.len(),
        }),
        "validate" => json(&ValidateOutput {
            contract_version: CONTRACT_VERSION,
            schema_version: workspace.schema_version,
            workspace_id: &workspace.workspace_id,
            workspace_digest: &digest,
            ok: true,
        }),
        "query" => query(&workspace, version, &options, &digest),
        "semantic-zoom" => semantic_zoom(&workspace, version, &options),
        _ => Err(CliError::Usage(format!(
            "unknown command: {command}\n\n{}",
            usage()
        ))),
    }
}

fn semantic_zoom(
    workspace: &WorkspaceDocument,
    version: &WorkspaceVersion,
    options: &Options,
) -> Result<String, CliError> {
    let lens_id = options
        .lens_id
        .as_deref()
        .ok_or_else(|| CliError::Usage("semantic-zoom requires --lens ID".to_string()))?;
    let scale = options
        .scale
        .ok_or_else(|| CliError::Usage("semantic-zoom requires --scale NUMBER".to_string()))?;
    if !scale.is_finite() || scale <= 0.0 {
        return Err(CliError::Usage(
            "--scale must be a finite positive number".to_string(),
        ));
    }

    let frame = workspace
        .semantic_zoom_frame(&version.id, lens_id, scale)
        .map_err(|error| CliError::Operational(error.to_string()))?;
    let mut cards = frame
        .cards
        .into_iter()
        .map(|card| SemanticZoomCardOutput {
            id: card.card.id,
            bounds: card.card.bounds,
            opacity: card.opacity,
            card_scale: card.card_scale,
        })
        .collect::<Vec<_>>();
    cards.sort_by_key(|card| card.id);

    let mut relationships = frame
        .relationships
        .into_iter()
        .map(|relationship| SemanticZoomRelationshipOutput {
            id: relationship.id,
            source_id: relationship.source_id,
            target_id: relationship.target_id,
            opacity: relationship.opacity,
        })
        .collect::<Vec<_>>();
    relationships.sort_by(|left, right| left.id.cmp(&right.id));

    json(&SemanticZoomOutput {
        contract_version: CONTRACT_VERSION,
        version_id: &version.id,
        lens_id,
        scale,
        cards,
        relationships,
    })
}

fn query(
    workspace: &WorkspaceDocument,
    version: &WorkspaceVersion,
    options: &Options,
    digest: &str,
) -> Result<String, CliError> {
    let root_id = options
        .concept_id
        .ok_or_else(|| CliError::Usage("query requires --concept ID or --from ID".to_string()))?;
    if !version.concepts.iter().any(|concept| concept.id == root_id) {
        return Err(CliError::Operational(format!(
            "concept {root_id} does not exist in version {}",
            version.id
        )));
    }
    let max_depth = options.depth.unwrap_or(1);
    if max_depth == 0 {
        return Err(CliError::Usage(
            "--depth must be greater than zero".to_string(),
        ));
    }
    let mut result_ids = BTreeSet::from([root_id]);
    let mut frontier = BTreeSet::from([root_id]);
    let mut relationship_ids = BTreeSet::new();

    for _ in 0..max_depth {
        let mut next_frontier = BTreeSet::new();
        for relationship in &version.relationships {
            if options
                .relationship_kind
                .is_some_and(|kind| relationship.kind != kind)
            {
                continue;
            }
            let connected = match options.direction {
                Direction::Out if frontier.contains(&relationship.source_id) => {
                    Some(relationship.target_id)
                }
                Direction::In if frontier.contains(&relationship.target_id) => {
                    Some(relationship.source_id)
                }
                Direction::Both if frontier.contains(&relationship.source_id) => {
                    Some(relationship.target_id)
                }
                Direction::Both if frontier.contains(&relationship.target_id) => {
                    Some(relationship.source_id)
                }
                _ => None,
            };
            if let Some(concept_id) = connected {
                relationship_ids.insert(relationship.id.as_str());
                if result_ids.insert(concept_id) {
                    next_frontier.insert(concept_id);
                }
            }
        }
        if next_frontier.is_empty() {
            break;
        }
        frontier = next_frontier;
    }

    let concepts = version
        .concepts
        .iter()
        .filter(|concept| result_ids.contains(&concept.id))
        .collect::<Vec<_>>();
    let relationships = version
        .relationships
        .iter()
        .filter(|relationship| relationship_ids.contains(relationship.id.as_str()))
        .collect::<Vec<_>>();
    json(&QueryOutput {
        contract_version: CONTRACT_VERSION,
        schema_version: workspace.schema_version,
        workspace_id: &workspace.workspace_id,
        workspace_digest: digest,
        version_id: &version.id,
        root_concept_id: root_id,
        direction: options.direction,
        depth: max_depth,
        concepts,
        relationships,
    })
}

fn json<T: Serialize>(value: &T) -> Result<String, CliError> {
    serde_json::to_string_pretty(value)
        .map_err(|error| CliError::Operational(format!("could not encode JSON output: {error}")))
}

#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "snake_case")]
enum Direction {
    In,
    Out,
    #[default]
    Both,
}

#[derive(Default)]
struct Options {
    workspace_path: Option<String>,
    version_id: Option<String>,
    lens_id: Option<String>,
    scale: Option<f32>,
    concept_id: Option<u32>,
    direction: Direction,
    relationship_kind: Option<RelationshipKind>,
    depth: Option<u32>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, CliError> {
        let mut options = Self::default();
        let mut index = 0;
        while index < arguments.len() {
            let option = &arguments[index];
            let value = arguments
                .get(index + 1)
                .ok_or_else(|| CliError::Usage(format!("{option} requires a value")))?;
            match option.as_str() {
                "--workspace" => options.workspace_path = Some(value.clone()),
                "--version" => options.version_id = Some(value.clone()),
                "--lens" => options.lens_id = Some(value.clone()),
                "--scale" => {
                    options.scale = Some(
                        value
                            .parse()
                            .map_err(|_| CliError::Usage("--scale must be a number".to_string()))?,
                    )
                }
                "--concept" | "--from" => {
                    options.concept_id = Some(value.parse().map_err(|_| {
                        CliError::Usage(format!("{option} must be a numeric concept ID"))
                    })?)
                }
                "--direction" => {
                    options.direction = match value.as_str() {
                        "in" => Direction::In,
                        "out" => Direction::Out,
                        "both" => Direction::Both,
                        _ => {
                            return Err(CliError::Usage(
                                "--direction must be in, out, or both".to_string(),
                            ));
                        }
                    }
                }
                "--kind" => options.relationship_kind = Some(parse_relationship_kind(value)?),
                "--depth" => {
                    options.depth = Some(value.parse().map_err(|_| {
                        CliError::Usage("--depth must be a positive integer".to_string())
                    })?)
                }
                "--format" if value == "json" => {}
                "--format" => {
                    return Err(CliError::Usage(
                        "only --format json is supported".to_string(),
                    ));
                }
                _ => return Err(CliError::Usage(format!("unknown option: {option}"))),
            }
            index += 2;
        }
        Ok(options)
    }
}

fn parse_relationship_kind(value: &str) -> Result<RelationshipKind, CliError> {
    match value {
        "contains" => Ok(RelationshipKind::Contains),
        "owns" => Ok(RelationshipKind::Owns),
        "operates" => Ok(RelationshipKind::Operates),
        "depends_on" => Ok(RelationshipKind::DependsOn),
        "implemented_in" => Ok(RelationshipKind::ImplementedIn),
        "delivers" => Ok(RelationshipKind::Delivers),
        "deployed_to" => Ok(RelationshipKind::DeployedTo),
        _ => Err(CliError::Usage(format!(
            "unsupported relationship kind: {value}"
        ))),
    }
}

#[derive(Serialize)]
struct InspectOutput<'a> {
    contract_version: u32,
    schema_version: u32,
    workspace_id: &'a str,
    workspace_title: &'a str,
    workspace_digest: &'a str,
    current_state_id: &'a str,
    selected_version: VersionSummary<'a>,
    version_ids: Vec<&'a str>,
    custom_type_count: usize,
}

#[derive(Serialize)]
struct VersionSummary<'a> {
    id: &'a str,
    title: &'a str,
    concept_count: usize,
    relationship_count: usize,
    lens_count: usize,
}

impl<'a> From<&'a WorkspaceVersion> for VersionSummary<'a> {
    fn from(version: &'a WorkspaceVersion) -> Self {
        Self {
            id: &version.id,
            title: &version.title,
            concept_count: version.concepts.len(),
            relationship_count: version.relationships.len(),
            lens_count: version.lenses.len(),
        }
    }
}

#[derive(Serialize)]
struct ValidateOutput<'a> {
    contract_version: u32,
    schema_version: u32,
    workspace_id: &'a str,
    workspace_digest: &'a str,
    ok: bool,
}

#[derive(Serialize)]
struct QueryOutput<'a> {
    contract_version: u32,
    schema_version: u32,
    workspace_id: &'a str,
    workspace_digest: &'a str,
    version_id: &'a str,
    root_concept_id: u32,
    direction: Direction,
    depth: u32,
    concepts: Vec<&'a infinite_note_canvas_canvas::WorkspaceConcept>,
    relationships: Vec<&'a infinite_note_canvas_canvas::TypedRelationship>,
}

#[derive(Serialize)]
struct SemanticZoomOutput<'a> {
    contract_version: u32,
    version_id: &'a str,
    lens_id: &'a str,
    scale: f32,
    cards: Vec<SemanticZoomCardOutput>,
    relationships: Vec<SemanticZoomRelationshipOutput>,
}

#[derive(Serialize)]
struct SemanticZoomCardOutput {
    id: u32,
    bounds: infinite_note_canvas_canvas::Rect,
    opacity: f32,
    card_scale: f32,
}

#[derive(Serialize)]
struct SemanticZoomRelationshipOutput {
    id: String,
    source_id: u32,
    target_id: u32,
    opacity: f32,
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Operational(String),
    InvalidWorkspace(String),
}

impl CliError {
    fn is_invalid_workspace(&self) -> bool {
        matches!(self, Self::InvalidWorkspace(_))
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) | Self::Operational(message) | Self::InvalidWorkspace(message) => {
                formatter.write_str(message)
            }
        }
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn usage() -> String {
    [
        "Usage:",
        "  infinite-note-canvas-cli inspect --workspace PATH [--version ID] --format json",
        "  infinite-note-canvas-cli validate --workspace PATH --format json",
        "  infinite-note-canvas-cli query --workspace PATH --concept ID [--version ID] [--direction in|out|both] [--kind KIND] [--depth N] --format json",
        "  infinite-note-canvas-cli semantic-zoom --workspace PATH [--version ID] --lens ID --scale NUMBER --format json",
        "  infinite-note-canvas-cli mcp --workspace PATH [--allow-write]",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::run;
    use infinite_note_canvas_canvas::WorkspaceDocument;
    use std::{env, fs};

    const SEMANTIC_ZOOM_FIXTURE: &str =
        include_str!("../../../workspaces/semantic-zoom-evaluation.fixture.json");

    #[test]
    fn inspect_and_query_emit_json_contract_records() {
        let workspace_path = env::temp_dir().join(format!(
            "infinite-note-canvas-cli-{}-inspect-query.json",
            std::process::id()
        ));
        fs::write(
            &workspace_path,
            WorkspaceDocument::sample().to_pretty_json().unwrap(),
        )
        .unwrap();

        let inspect = run(vec![
            "inspect".to_string(),
            "--workspace".to_string(),
            workspace_path.to_string_lossy().into_owned(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .unwrap();
        let query = run(vec![
            "query".to_string(),
            "--workspace".to_string(),
            workspace_path.to_string_lossy().into_owned(),
            "--concept".to_string(),
            "7".to_string(),
            "--direction".to_string(),
            "out".to_string(),
            "--depth".to_string(),
            "1".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .unwrap();

        let inspect: serde_json::Value = serde_json::from_str(&inspect).unwrap();
        let query: serde_json::Value = serde_json::from_str(&query).unwrap();
        assert_eq!(inspect["contract_version"], 1);
        assert_eq!(inspect["selected_version"]["id"], "current");
        assert_eq!(query["root_concept_id"], 7);
        assert_eq!(query["relationships"].as_array().unwrap().len(), 3);

        fs::remove_file(workspace_path).unwrap();
    }

    #[test]
    fn semantic_zoom_emits_a_stable_read_only_projection_contract() {
        let workspace_path = env::temp_dir().join(format!(
            "infinite-note-canvas-cli-{}-semantic-zoom.json",
            std::process::id()
        ));
        fs::write(&workspace_path, SEMANTIC_ZOOM_FIXTURE).unwrap();

        let broad = semantic_zoom_output(
            &workspace_path,
            "current",
            "semantic-zoom-evaluation",
            "0.5",
        );
        let repeated_broad = semantic_zoom_output(
            &workspace_path,
            "current",
            "semantic-zoom-evaluation",
            "0.5",
        );
        let overlap = semantic_zoom_output(
            &workspace_path,
            "current",
            "semantic-zoom-evaluation",
            "0.8",
        );
        let close = semantic_zoom_output(
            &workspace_path,
            "current",
            "semantic-zoom-evaluation",
            "1.6",
        );

        assert_eq!(broad, repeated_broad);

        let broad: serde_json::Value = serde_json::from_str(&broad).unwrap();
        assert_eq!(broad["contract_version"], 1);
        assert_eq!(broad["version_id"], "current");
        assert_eq!(broad["lens_id"], "semantic-zoom-evaluation");
        assert_eq!(concept_ids(&broad), vec![10, 20]);
        assert_eq!(relationship_ids(&broad), vec!["context-to-working"]);

        let overlap: serde_json::Value = serde_json::from_str(&overlap).unwrap();
        assert_eq!(concept_ids(&overlap), vec![10, 20, 30]);
        assert_eq!(
            relationship_ids(&overlap),
            vec!["context-to-working", "working-to-detail"]
        );

        let close: serde_json::Value = serde_json::from_str(&close).unwrap();
        assert_eq!(concept_ids(&close), vec![10, 20, 30]);
        assert_eq!(
            relationship_ids(&close),
            vec!["context-to-working", "working-to-detail"]
        );
        let context_card = close["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|card| card["id"] == 10)
            .unwrap();
        assert_eq!(context_card["card_scale"].as_f64(), Some(0.5));
        assert_eq!(context_card["bounds"]["size"]["x"].as_f64(), Some(120.0));

        let missing_version = run(vec![
            "semantic-zoom".to_string(),
            "--workspace".to_string(),
            workspace_path.to_string_lossy().into_owned(),
            "--version".to_string(),
            "absent".to_string(),
            "--lens".to_string(),
            "semantic-zoom-evaluation".to_string(),
            "--scale".to_string(),
            "0.5".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .unwrap_err();
        assert_eq!(
            missing_version.to_string(),
            "workspace version was not found: absent"
        );

        let missing_lens = run(vec![
            "semantic-zoom".to_string(),
            "--workspace".to_string(),
            workspace_path.to_string_lossy().into_owned(),
            "--lens".to_string(),
            "absent".to_string(),
            "--scale".to_string(),
            "0.5".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .unwrap_err();
        assert_eq!(
            missing_lens.to_string(),
            "workspace version current has no lens named absent"
        );

        let non_finite_scale = run(vec![
            "semantic-zoom".to_string(),
            "--workspace".to_string(),
            workspace_path.to_string_lossy().into_owned(),
            "--lens".to_string(),
            "semantic-zoom-evaluation".to_string(),
            "--scale".to_string(),
            "NaN".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .unwrap_err();
        assert_eq!(
            non_finite_scale.to_string(),
            "--scale must be a finite positive number"
        );

        assert_eq!(
            fs::read_to_string(&workspace_path).unwrap(),
            SEMANTIC_ZOOM_FIXTURE
        );
        fs::remove_file(workspace_path).unwrap();
    }

    fn semantic_zoom_output(
        workspace_path: &std::path::Path,
        version_id: &str,
        lens_id: &str,
        scale: &str,
    ) -> String {
        run(vec![
            "semantic-zoom".to_string(),
            "--workspace".to_string(),
            workspace_path.to_string_lossy().into_owned(),
            "--version".to_string(),
            version_id.to_string(),
            "--lens".to_string(),
            lens_id.to_string(),
            "--scale".to_string(),
            scale.to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .unwrap()
    }

    fn concept_ids(output: &serde_json::Value) -> Vec<u64> {
        output["cards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|card| card["id"].as_u64().unwrap())
            .collect()
    }

    fn relationship_ids(output: &serde_json::Value) -> Vec<&str> {
        output["relationships"]
            .as_array()
            .unwrap()
            .iter()
            .map(|relationship| relationship["id"].as_str().unwrap())
            .collect()
    }
}
