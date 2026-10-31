//! A local, stdio-only MCP adapter for one explicitly selected workspace.
//!
//! The adapter is deliberately headless. It shares the typed staged actions
//! and safe local persistence path used by the application, but it does not
//! expose a network listener or control a running application window.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, BufRead, Write},
    path::PathBuf,
};

use infinite_note_canvas_canvas::{
    Point, Rect, RelationshipKind, SemanticZoomCardLayerMode, SemanticZoomLayerPercentSettings,
    TypedRelationship, VisualBoundary, WorkspaceConcept, WorkspaceDocument, WorkspaceLens,
};
use infinite_note_canvas_workspace_session::{
    ConceptDraft, WorkspaceSession, evaluation_actions::ConceptInspectorEditDraft,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const MCP_PROTOCOL_VERSION: &str = "2025-06-18";
const AGENT_CONTRACT_VERSION: u32 = 1;
const MAXIMUM_LENS_READ_LIMIT: usize = 250;

/// Starts the local MCP server. The server has access to exactly one workspace
/// file for the duration of its standard-input/output session.
pub(crate) fn run(arguments: &[String]) -> Result<(), String> {
    let options = McpOptions::parse(arguments)?;
    let server = AgentServer::open(options)?;
    server.serve_stdio()
}

struct McpOptions {
    workspace_path: PathBuf,
    allow_write: bool,
}

impl McpOptions {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut workspace_path = None;
        let mut allow_write = false;
        let mut index = 0;
        while index < arguments.len() {
            match arguments[index].as_str() {
                "--workspace" => {
                    let path = arguments.get(index + 1).ok_or_else(|| {
                        "mcp --workspace requires a workspace file path".to_string()
                    })?;
                    workspace_path = Some(PathBuf::from(path));
                    index += 2;
                }
                "--allow-write" => {
                    allow_write = true;
                    index += 1;
                }
                "--help" | "-h" => return Err(mcp_usage()),
                option => return Err(format!("unknown mcp option: {option}\n\n{}", mcp_usage())),
            }
        }
        let workspace_path = workspace_path.ok_or_else(|| {
            "mcp requires --workspace PATH. The server never selects a workspace itself."
                .to_string()
        })?;
        let workspace_path = resolve_workspace_path(workspace_path)?;
        Ok(Self {
            workspace_path,
            allow_write,
        })
    }
}

fn mcp_usage() -> String {
    [
        "Usage:",
        "  infinite-note-canvas-cli mcp --workspace PATH [--allow-write]",
        "",
        "The MCP server uses standard input/output and has access only to PATH.",
        "Without --allow-write it exposes read tools only.",
        "With --allow-write, create_workspace can create PATH when it does not exist.",
    ]
    .join("\n")
}

fn resolve_workspace_path(path: PathBuf) -> Result<PathBuf, String> {
    if path.exists() {
        let path = fs::canonicalize(&path).map_err(|error| {
            format!(
                "could not resolve the MCP workspace path {}: {error}",
                path.display()
            )
        })?;
        if !path.is_file() {
            return Err(format!(
                "MCP workspace path must name a file: {}",
                path.display()
            ));
        }
        return Ok(path);
    }

    let parent = path.parent().ok_or_else(|| {
        format!(
            "MCP workspace path must include an existing parent folder: {}",
            path.display()
        )
    })?;
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("MCP workspace path must name a file: {}", path.display()))?;
    let parent = fs::canonicalize(parent).map_err(|error| {
        format!(
            "could not resolve the parent folder for MCP workspace {}: {error}",
            path.display()
        )
    })?;
    if !parent.is_dir() {
        return Err(format!(
            "MCP workspace parent must be a folder: {}",
            parent.display()
        ));
    }
    Ok(parent.join(file_name))
}

struct AgentServer {
    workspace_path: PathBuf,
    allow_write: bool,
}

impl AgentServer {
    fn open(options: McpOptions) -> Result<Self, String> {
        if options.workspace_path.exists() {
            let source = fs::read_to_string(&options.workspace_path).map_err(|error| {
                format!(
                    "could not read MCP workspace {}: {error}",
                    options.workspace_path.display()
                )
            })?;
            WorkspaceDocument::parse_json(&source).map_err(|error| {
                format!(
                    "could not validate MCP workspace {}: {error}",
                    options.workspace_path.display()
                )
            })?;
        } else if !options.allow_write {
            return Err(format!(
                "MCP workspace does not exist: {}. Start with --allow-write to create it.",
                options.workspace_path.display()
            ));
        }
        Ok(Self {
            workspace_path: options.workspace_path,
            allow_write: options.allow_write,
        })
    }

    fn workspace_exists(&self) -> bool {
        self.workspace_path.is_file()
    }

    fn serve_stdio(&self) -> Result<(), String> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut output = stdout.lock();
        for line in stdin.lock().lines() {
            let line = line.map_err(|error| format!("could not read MCP input: {error}"))?;
            if line.trim().is_empty() {
                continue;
            }
            let response = match serde_json::from_str(&line) {
                Ok(request) => self.handle(request),
                Err(error) => Some(json_rpc_error(
                    Value::Null,
                    -32700,
                    format!("invalid JSON-RPC request: {error}"),
                )),
            };
            if let Some(response) = response {
                serde_json::to_writer(&mut output, &response)
                    .map_err(|error| format!("could not write MCP response: {error}"))?;
                output
                    .write_all(b"\n")
                    .map_err(|error| format!("could not finish MCP response: {error}"))?;
                output
                    .flush()
                    .map_err(|error| format!("could not flush MCP response: {error}"))?;
            }
        }
        Ok(())
    }

    fn handle(&self, request: Value) -> Option<Value> {
        let object = match request.as_object() {
            Some(object) => object,
            None => {
                return Some(json_rpc_error(
                    Value::Null,
                    -32600,
                    "JSON-RPC requests must be objects.",
                ));
            }
        };
        let id = object.get("id").cloned();
        // MCP notifications do not receive replies. They cannot invoke a tool,
        // which keeps a missing request identifier from causing a write.
        let id = id?;
        if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Some(json_rpc_error(
                id,
                -32600,
                "MCP requests must use JSON-RPC version 2.0.",
            ));
        }
        let Some(method) = object.get("method").and_then(Value::as_str) else {
            return Some(json_rpc_error(
                id,
                -32600,
                "MCP requests must include a method.",
            ));
        };
        let params = object.get("params").cloned().unwrap_or_else(|| json!({}));
        match method {
            "initialize" => Some(json_rpc_result(
                id,
                json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": {
                        "name": "infinite-note-canvas",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                    "instructions": "The server is restricted to one local workspace file. When that file does not exist, call create_workspace first. Otherwise use workspace_overview before reading or changing a map. Apply one validated change at a time.",
                }),
            )),
            "tools/list" => Some(json_rpc_result(id, json!({ "tools": self.tools() }))),
            "tools/call" => Some(self.handle_tool_call(id, params)),
            _ => Some(json_rpc_error(
                id,
                -32601,
                format!("unsupported MCP method: {method}"),
            )),
        }
    }

    fn tools(&self) -> Vec<Value> {
        if !self.workspace_exists() {
            return if self.allow_write {
                vec![tool_definition(
                    "create_workspace",
                    "Create an empty workspace at the server's selected --workspace path. This is available only when that exact file does not exist and never replaces an existing file.",
                    json!({
                        "type": "object",
                        "properties": {
                            "title": { "type": "string", "minLength": 1, "description": "Workspace title." },
                            "workspace_id": { "type": "string", "minLength": 1, "description": "Optional stable workspace identifier. Defaults to a safe form of the title." }
                        },
                        "required": ["title"],
                        "additionalProperties": false
                    }),
                )]
            } else {
                Vec::new()
            };
        }
        let mut tools = vec![
            tool_definition(
                "workspace_overview",
                "Read workspace identity, types, versions, Lens summaries, boundaries, and zoom-layer summaries. Call this before selecting a Lens or making a change.",
                json!({ "type": "object", "additionalProperties": false }),
            ),
            tool_definition(
                "read_lens",
                "Read a bounded page of cards in one Lens, their positions, direct relationships, boundaries, and semantic zoom settings. The default limit is 100 cards; the maximum is 250.",
                json!({
                    "type": "object",
                    "properties": {
                        "version_id": { "type": "string", "description": "Defaults to the current version." },
                        "lens_id": { "type": "string", "description": "Defaults to the version's first Lens." },
                        "offset": { "type": "integer", "minimum": 0, "description": "Zero-based card offset." },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 250, "description": "Cards to return." }
                    },
                    "additionalProperties": false
                }),
            ),
            tool_definition(
                "query_workspace",
                "Traverse concepts and typed relationships from one concept. Use this instead of reading large maps when answering a dependency or ownership question.",
                json!({
                    "type": "object",
                    "properties": {
                        "version_id": { "type": "string", "description": "Defaults to the current version." },
                        "concept_id": { "type": "integer", "description": "Starting concept identifier." },
                        "direction": { "type": "string", "enum": ["in", "out", "both"], "default": "both" },
                        "relationship_kind": { "type": "string", "enum": ["contains", "owns", "operates", "depends_on", "implemented_in", "delivers", "deployed_to"] },
                        "depth": { "type": "integer", "minimum": 1, "default": 1 }
                    },
                    "required": ["concept_id"],
                    "additionalProperties": false
                }),
            ),
            tool_definition(
                "preview_semantic_zoom",
                "Read the cards, relationships, and layers visible at a selected zoom scale without changing the workspace.",
                json!({
                    "type": "object",
                    "properties": {
                        "version_id": { "type": "string", "description": "Defaults to the current version." },
                        "lens_id": { "type": "string", "description": "Defaults to the version's first Lens." },
                        "scale": { "type": "number", "exclusiveMinimum": 0, "description": "Canvas scale. 1.0 means 100%." }
                    },
                    "required": ["scale"],
                    "additionalProperties": false
                }),
            ),
        ];
        if self.allow_write {
            tools.push(tool_definition(
                "apply_workspace_change",
                "Apply one validated change through the same local save, backup, and verification path as the app. The change kind may be create_card, update_card, delete_card, move_cards, resize_card, create_relationship, change_relationship_kind, remove_relationship, create_boundary, update_boundary, move_boundary, resize_boundary, remove_boundary, create_zoom_layer, update_zoom_layer, reorder_zoom_layer, delete_zoom_layer, or set_card_layer_mode. Read the target Lens before changing it. A change has the form {kind: ..., ...}.",
                json!({
                    "type": "object",
                    "properties": {
                        "version_id": { "type": "string", "description": "Defaults to the current version." },
                        "lens_id": { "type": "string", "description": "Defaults to the version's first Lens." },
                        "change": {
                            "type": "object",
                            "description": "A single typed change. Card and boundary bounds use {origin:{x,y},size:{x,y}}. Relationship kinds use contains, owns, operates, depends_on, implemented_in, delivers, or deployed_to. create_zoom_layer uses title, start_percent, end_percent, and fade_percent. set_card_layer_mode uses mode:{mode:'automatic'} or mode:{mode:'manual',layer_ids:[...]}.",
                            "required": ["kind"]
                        }
                    },
                    "required": ["change"],
                    "additionalProperties": false
                }),
            ));
        }
        tools
    }

    fn handle_tool_call(&self, id: Value, params: Value) -> Value {
        let call: ToolCall = match serde_json::from_value(params) {
            Ok(call) => call,
            Err(error) => {
                return json_rpc_error(
                    id,
                    -32602,
                    format!("invalid tools/call parameters: {error}"),
                );
            }
        };
        let arguments = call.arguments.unwrap_or_else(|| json!({}));
        let result = match call.name.as_str() {
            "create_workspace" if self.allow_write => deserialize_arguments(arguments)
                .and_then(|arguments| self.create_workspace(arguments)),
            "create_workspace" => Err(
                "write access was not granted. Restart this MCP server with --allow-write to create a workspace."
                    .to_string(),
            ),
            "workspace_overview" => self.workspace_overview(),
            "read_lens" => deserialize_arguments(arguments).and_then(|arguments| self.read_lens(arguments)),
            "query_workspace" => {
                deserialize_arguments(arguments).and_then(|arguments| self.query_workspace(arguments))
            }
            "preview_semantic_zoom" => deserialize_arguments(arguments)
                .and_then(|arguments| self.preview_semantic_zoom(arguments)),
            "apply_workspace_change" if self.allow_write => deserialize_arguments(arguments)
                .and_then(|arguments| self.apply_workspace_change(arguments)),
            "apply_workspace_change" => Err(
                "write access was not granted. Restart this MCP server with --allow-write to offer workspace changes."
                    .to_string(),
            ),
            name => return json_rpc_error(id, -32602, format!("unknown MCP tool: {name}")),
        };
        json_rpc_result(
            id,
            match result {
                Ok(value) => tool_success(value),
                Err(error) => tool_failure(error),
            },
        )
    }

    fn create_workspace(&self, arguments: CreateWorkspaceArguments) -> Result<Value, String> {
        if self.workspace_exists() {
            return Err(format!(
                "workspace already exists at {}. create_workspace never replaces an existing file.",
                self.workspace_path.display()
            ));
        }
        let title = arguments.title.trim();
        if title.is_empty() {
            return Err("workspace title must not be empty".to_string());
        }
        let workspace_id = arguments
            .workspace_id
            .filter(|workspace_id| !workspace_id.trim().is_empty())
            .unwrap_or_else(|| workspace_id_from_title(title));
        let document = WorkspaceDocument::blank(workspace_id, title.to_string())
            .map_err(|error| error.to_string())?;
        let session = WorkspaceSession::create_new(self.workspace_path.clone(), document)
            .map_err(|error| error.to_string())?;
        let source = fs::read_to_string(&self.workspace_path).map_err(|error| {
            format!(
                "could not read newly created workspace {}: {error}",
                self.workspace_path.display()
            )
        })?;
        serializable_value(&WorkspaceCreateOutput {
            contract_version: AGENT_CONTRACT_VERSION,
            workspace_path: self.workspace_path.display().to_string(),
            workspace_digest: workspace_digest(&source),
            workspace_id: session.document().workspace_id.clone(),
            workspace_title: session.document().title.clone(),
            version_id: session.active_version_id().to_string(),
            lens_id: session.active_lens_id().to_string(),
        })
    }

    fn read_document(&self) -> Result<(String, WorkspaceDocument), String> {
        let source = fs::read_to_string(&self.workspace_path).map_err(|error| {
            format!(
                "could not read workspace {}: {error}",
                self.workspace_path.display()
            )
        })?;
        let document = WorkspaceDocument::parse_json(&source).map_err(|error| {
            format!(
                "could not validate workspace {}: {error}",
                self.workspace_path.display()
            )
        })?;
        Ok((source, document))
    }

    fn load_session(&self, context: &WorkspaceContext) -> Result<WorkspaceSession, String> {
        let (_, document) = self.read_document()?;
        let version_id = context
            .version_id
            .clone()
            .unwrap_or_else(|| document.current_state_id().to_string());
        let lens_id = match &context.lens_id {
            Some(lens_id) => lens_id.clone(),
            None => document
                .default_lens_id(&version_id)
                .map_err(|error| error.to_string())?
                .to_string(),
        };
        WorkspaceSession::load(self.workspace_path.clone(), version_id, lens_id)
            .map_err(|error| error.to_string())
    }

    fn workspace_overview(&self) -> Result<Value, String> {
        let (source, document) = self.read_document()?;
        let versions = document
            .version_ids()
            .into_iter()
            .map(|version_id| {
                let version = document.version(version_id).expect("validated version id");
                WorkspaceVersionOverview::from(version)
            })
            .collect();
        let current_version_id = document.current_state_id().to_string();
        serializable_value(&WorkspaceOverview {
            contract_version: AGENT_CONTRACT_VERSION,
            workspace_path: self.workspace_path.display().to_string(),
            workspace_digest: workspace_digest(&source),
            workspace_id: document.workspace_id,
            workspace_title: document.title,
            schema_version: document.schema_version,
            current_version_id,
            custom_types: document.custom_types,
            versions,
            write_enabled: self.allow_write,
        })
    }

    fn read_lens(&self, arguments: LensReadArguments) -> Result<Value, String> {
        if arguments.limit == 0 || arguments.limit > MAXIMUM_LENS_READ_LIMIT {
            return Err(format!(
                "read_lens limit must be between 1 and {MAXIMUM_LENS_READ_LIMIT}"
            ));
        }
        let session = self.load_session(&arguments.context())?;
        let version = session
            .document()
            .version(session.active_version_id())
            .map_err(|error| error.to_string())?;
        let lens = lens_for_session(&session)?;
        let mut placements = lens.placements.clone();
        placements.sort_by_key(|placement| placement.concept_id);
        let total_card_count = placements.len();
        let page = placements
            .into_iter()
            .skip(arguments.offset)
            .take(arguments.limit)
            .collect::<Vec<_>>();
        let page_card_ids: BTreeSet<_> =
            page.iter().map(|placement| placement.concept_id).collect();
        let cards = page
            .into_iter()
            .map(|placement| {
                let concept = session
                    .document()
                    .concept(session.active_version_id(), placement.concept_id)
                    .map_err(|error| error.to_string())?
                    .clone();
                let layer_visibility = session
                    .document()
                    .semantic_zoom_card_layer_visibility(
                        session.active_version_id(),
                        session.active_lens_id(),
                        placement.concept_id,
                    )
                    .map_err(|error| error.to_string())?;
                Ok(LensCard {
                    concept,
                    bounds: placement.bounds,
                    layer_mode: layer_visibility.mode,
                    visible_layer_ids: layer_visibility.visible_layer_ids,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let relationships = version
            .relationships
            .iter()
            .filter(|relationship| {
                page_card_ids.contains(&relationship.source_id)
                    || page_card_ids.contains(&relationship.target_id)
            })
            .cloned()
            .collect();
        let next_offset = (arguments.offset + cards.len() < total_card_count)
            .then_some(arguments.offset + cards.len());
        serializable_value(&LensReadOutput {
            contract_version: AGENT_CONTRACT_VERSION,
            version_id: session.active_version_id().to_string(),
            lens_id: session.active_lens_id().to_string(),
            total_card_count,
            offset: arguments.offset,
            next_offset,
            cards,
            relationships,
            visual_boundaries: lens.visual_boundaries.clone(),
            semantic_zoom_layers: lens.semantic_zoom_layers.clone(),
        })
    }

    fn query_workspace(&self, arguments: WorkspaceQueryArguments) -> Result<Value, String> {
        if arguments.depth == 0 {
            return Err("query_workspace depth must be greater than zero".to_string());
        }
        let (_, document) = self.read_document()?;
        let version_id = arguments
            .version_id
            .unwrap_or_else(|| document.current_state_id().to_string());
        let version = document
            .version(&version_id)
            .map_err(|error| error.to_string())?;
        if !version
            .concepts
            .iter()
            .any(|concept| concept.id == arguments.concept_id)
        {
            return Err(format!(
                "concept {} does not exist in version {version_id}",
                arguments.concept_id
            ));
        }

        let mut result_ids = BTreeSet::from([arguments.concept_id]);
        let mut frontier = BTreeSet::from([arguments.concept_id]);
        let mut relationship_ids = BTreeSet::new();
        for _ in 0..arguments.depth {
            let mut next_frontier = BTreeSet::new();
            for relationship in &version.relationships {
                if arguments
                    .relationship_kind
                    .is_some_and(|kind| relationship.kind != kind)
                {
                    continue;
                }
                let connected = match arguments.direction {
                    QueryDirection::Out if frontier.contains(&relationship.source_id) => {
                        Some(relationship.target_id)
                    }
                    QueryDirection::In if frontier.contains(&relationship.target_id) => {
                        Some(relationship.source_id)
                    }
                    QueryDirection::Both if frontier.contains(&relationship.source_id) => {
                        Some(relationship.target_id)
                    }
                    QueryDirection::Both if frontier.contains(&relationship.target_id) => {
                        Some(relationship.source_id)
                    }
                    _ => None,
                };
                if let Some(concept_id) = connected {
                    relationship_ids.insert(relationship.id.clone());
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
        serializable_value(&WorkspaceQueryOutput {
            contract_version: AGENT_CONTRACT_VERSION,
            version_id,
            root_concept_id: arguments.concept_id,
            direction: arguments.direction,
            depth: arguments.depth,
            concepts: version
                .concepts
                .iter()
                .filter(|concept| result_ids.contains(&concept.id))
                .cloned()
                .collect(),
            relationships: version
                .relationships
                .iter()
                .filter(|relationship| relationship_ids.contains(&relationship.id))
                .cloned()
                .collect(),
        })
    }

    fn preview_semantic_zoom(&self, arguments: SemanticZoomArguments) -> Result<Value, String> {
        if !arguments.scale.is_finite() || arguments.scale <= 0.0 {
            return Err(
                "preview_semantic_zoom scale must be finite and greater than zero".to_string(),
            );
        }
        let (_, document) = self.read_document()?;
        let context = arguments.context();
        let version_id = context
            .version_id
            .unwrap_or_else(|| document.current_state_id().to_string());
        let lens_id = match context.lens_id {
            Some(lens_id) => lens_id,
            None => document
                .default_lens_id(&version_id)
                .map_err(|error| error.to_string())?
                .to_string(),
        };
        let frame = document
            .semantic_zoom_frame(&version_id, &lens_id, arguments.scale)
            .map_err(|error| error.to_string())?;
        serializable_value(&SemanticZoomPreview {
            contract_version: AGENT_CONTRACT_VERSION,
            version_id,
            lens_id,
            scale: arguments.scale,
            cards: frame
                .cards
                .into_iter()
                .map(|card| SemanticZoomCardOutput {
                    concept_id: card.card.id,
                    bounds: card.card.bounds,
                    opacity: card.opacity,
                    card_scale: card.card_scale,
                })
                .collect(),
            relationships: frame
                .relationships
                .into_iter()
                .map(|relationship| SemanticZoomRelationshipOutput {
                    id: relationship.id,
                    source_id: relationship.source_id,
                    target_id: relationship.target_id,
                    opacity: relationship.opacity,
                })
                .collect(),
            layers: frame
                .layers
                .into_iter()
                .map(|layer| SemanticZoomLayerOutput {
                    id: layer.id,
                    title: layer.title,
                    opacity: layer.opacity,
                })
                .collect(),
        })
    }

    fn apply_workspace_change(&self, arguments: ApplyArguments) -> Result<Value, String> {
        let context = arguments.context();
        let mut session = self.load_session(&context)?;
        let change_kind = arguments.change.label();
        let staged = stage_change(&session, arguments.change)?;
        session.commit(staged).map_err(|error| error.to_string())?;
        let (source, document) = self.read_document()?;
        serializable_value(&WorkspaceChangeOutput {
            contract_version: AGENT_CONTRACT_VERSION,
            change_kind,
            workspace_path: self.workspace_path.display().to_string(),
            workspace_digest: workspace_digest(&source),
            version_id: session.active_version_id().to_string(),
            lens_id: session.active_lens_id().to_string(),
            selected_concept_ids: session.selection_ids().to_vec(),
            concept_count: document
                .version(session.active_version_id())
                .map_err(|error| error.to_string())?
                .concepts
                .len(),
        })
    }
}

fn tool_definition(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn json_rpc_error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message.into() },
    })
}

fn tool_success(value: Value) -> Value {
    let text = serde_json::to_string_pretty(&value)
        .unwrap_or_else(|error| format!("could not encode tool result: {error}"));
    json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": value,
    })
}

fn tool_failure(error: String) -> Value {
    json!({
        "content": [{ "type": "text", "text": error }],
        "isError": true,
    })
}

fn deserialize_arguments<T: for<'de> Deserialize<'de>>(arguments: Value) -> Result<T, String> {
    serde_json::from_value(arguments).map_err(|error| format!("invalid tool arguments: {error}"))
}

fn serializable_value(value: &impl Serialize) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| format!("could not encode tool result: {error}"))
}

fn workspace_digest(source: &str) -> String {
    let hash = source
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("fnv1a64:{hash:016x}")
}

fn workspace_id_from_title(title: &str) -> String {
    let mut identifier = String::new();
    let mut previous_was_separator = false;
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            identifier.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !identifier.is_empty() && !previous_was_separator {
            identifier.push('-');
            previous_was_separator = true;
        }
    }
    if previous_was_separator {
        identifier.pop();
    }
    if identifier.is_empty() {
        "workspace".to_string()
    } else {
        identifier
    }
}

fn lens_for_session(session: &WorkspaceSession) -> Result<&WorkspaceLens, String> {
    session
        .document()
        .version(session.active_version_id())
        .map_err(|error| error.to_string())?
        .lenses
        .iter()
        .find(|lens| lens.id == session.active_lens_id())
        .ok_or_else(|| {
            format!(
                "version {} does not contain Lens {}",
                session.active_version_id(),
                session.active_lens_id()
            )
        })
}

fn stage_change(
    session: &WorkspaceSession,
    change: WorkspaceChange,
) -> Result<infinite_note_canvas_workspace_session::StagedWorkspaceMutation, String> {
    match change {
        WorkspaceChange::CreateCard {
            type_id,
            title,
            summary,
            bounds,
        } => session
            .stage_create_active_concept(ConceptDraft {
                type_id,
                title,
                summary,
                bounds,
            })
            .map_err(|error| error.to_string()),
        WorkspaceChange::UpdateCard {
            concept_id,
            type_id,
            title,
            summary,
            details,
            attributes,
        } => {
            let current = session
                .document()
                .concept(session.active_version_id(), concept_id)
                .map_err(|error| error.to_string())?;
            session
                .stage_edit_active_concept_inspector(ConceptInspectorEditDraft {
                    concept_id,
                    type_id: type_id.unwrap_or_else(|| current.type_id.clone()),
                    title: title.unwrap_or_else(|| current.title.clone()),
                    summary: summary.unwrap_or_else(|| current.summary.clone()),
                    details: details.unwrap_or_else(|| current.details.clone()),
                    assertion_attributes: attributes.unwrap_or_else(|| current.attributes.clone()),
                })
                .map_err(|error| error.to_string())
        }
        WorkspaceChange::DeleteCard { concept_id } => session
            .stage_delete_active_concept(concept_id)
            .map_err(|error| error.to_string()),
        WorkspaceChange::MoveCards { concept_ids, delta } => session
            .stage_move_active_concepts(&concept_ids, delta)
            .map_err(|error| error.to_string()),
        WorkspaceChange::ResizeCard { concept_id, bounds } => session
            .stage_resize_active_concept(concept_id, bounds)
            .map_err(|error| error.to_string()),
        WorkspaceChange::CreateRelationship {
            source_id,
            target_id,
            relationship_kind,
        } => session
            .stage_create_relationship(source_id, target_id, relationship_kind)
            .map_err(|error| error.to_string()),
        WorkspaceChange::ChangeRelationshipKind {
            relationship_id,
            relationship_kind,
        } => session
            .stage_change_relationship_kind(&relationship_id, relationship_kind)
            .map_err(|error| error.to_string()),
        WorkspaceChange::RemoveRelationship { relationship_id } => session
            .stage_remove_relationship(&relationship_id)
            .map_err(|error| error.to_string()),
        WorkspaceChange::CreateBoundary {
            title,
            summary,
            bounds,
        } => session
            .stage_create_visual_boundary(title, summary, bounds)
            .map_err(|error| error.to_string()),
        WorkspaceChange::UpdateBoundary {
            boundary_id,
            title,
            summary,
        } => session
            .stage_update_visual_boundary(&boundary_id, title, summary)
            .map_err(|error| error.to_string()),
        WorkspaceChange::MoveBoundary { boundary_id, delta } => session
            .stage_move_visual_boundary(&boundary_id, delta)
            .map_err(|error| error.to_string()),
        WorkspaceChange::ResizeBoundary {
            boundary_id,
            bounds,
        } => session
            .stage_resize_visual_boundary(&boundary_id, bounds)
            .map_err(|error| error.to_string()),
        WorkspaceChange::RemoveBoundary { boundary_id } => session
            .stage_remove_visual_boundary(&boundary_id)
            .map_err(|error| error.to_string()),
        WorkspaceChange::CreateZoomLayer {
            title,
            start_percent,
            end_percent,
            fade_percent,
        } => {
            let settings = SemanticZoomLayerPercentSettings {
                active_zoom_range: infinite_note_canvas_canvas::ZoomPercentRange {
                    start_percent,
                    end_percent,
                },
                fade_percent,
            };
            if session
                .available_semantic_zoom_layers()
                .map_err(|error| error.to_string())?
                .is_empty()
            {
                session
                    .stage_create_semantic_zoom_base_layer_with_percent_settings(title, settings)
                    .map_err(|error| error.to_string())
            } else {
                session
                    .stage_create_semantic_zoom_layer_with_percent_settings(title, settings)
                    .map_err(|error| error.to_string())
            }
        }
        WorkspaceChange::UpdateZoomLayer {
            layer_id,
            title,
            start_percent,
            end_percent,
            fade_percent,
            card_scale,
        } => session
            .stage_update_semantic_zoom_layer_percent_settings(
                &layer_id,
                title,
                SemanticZoomLayerPercentSettings {
                    active_zoom_range: infinite_note_canvas_canvas::ZoomPercentRange {
                        start_percent,
                        end_percent,
                    },
                    fade_percent,
                },
                card_scale,
            )
            .map_err(|error| error.to_string()),
        WorkspaceChange::ReorderZoomLayer {
            layer_id,
            destination_index,
        } => session
            .stage_reorder_semantic_zoom_layer(&layer_id, destination_index)
            .map_err(|error| error.to_string()),
        WorkspaceChange::DeleteZoomLayer { layer_id } => session
            .stage_delete_semantic_zoom_layer(&layer_id)
            .map_err(|error| error.to_string()),
        WorkspaceChange::SetCardLayerMode { concept_id, mode } => session
            .stage_set_semantic_zoom_card_layer_mode(concept_id, mode.into_domain())
            .map_err(|error| error.to_string()),
    }
}

#[derive(Deserialize)]
struct ToolCall {
    name: String,
    #[serde(default)]
    arguments: Option<Value>,
}

#[derive(Deserialize)]
struct CreateWorkspaceArguments {
    title: String,
    #[serde(default)]
    workspace_id: Option<String>,
}

#[derive(Clone, Default, Deserialize)]
struct WorkspaceContext {
    #[serde(default)]
    version_id: Option<String>,
    #[serde(default)]
    lens_id: Option<String>,
}

#[derive(Deserialize)]
struct LensReadArguments {
    #[serde(flatten)]
    context: WorkspaceContext,
    #[serde(default)]
    offset: usize,
    #[serde(default = "default_lens_read_limit")]
    limit: usize,
}

impl LensReadArguments {
    fn context(&self) -> WorkspaceContext {
        self.context.clone()
    }
}

fn default_lens_read_limit() -> usize {
    100
}

#[derive(Deserialize)]
struct WorkspaceQueryArguments {
    #[serde(default)]
    version_id: Option<String>,
    concept_id: u32,
    #[serde(default)]
    direction: QueryDirection,
    #[serde(default)]
    relationship_kind: Option<RelationshipKind>,
    #[serde(default = "default_query_depth")]
    depth: u32,
}

fn default_query_depth() -> u32 {
    1
}

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum QueryDirection {
    In,
    Out,
    #[default]
    Both,
}

#[derive(Deserialize)]
struct SemanticZoomArguments {
    #[serde(flatten)]
    context: WorkspaceContext,
    scale: f32,
}

impl SemanticZoomArguments {
    fn context(&self) -> WorkspaceContext {
        self.context.clone()
    }
}

#[derive(Deserialize)]
struct ApplyArguments {
    #[serde(flatten)]
    context: WorkspaceContext,
    change: WorkspaceChange,
}

impl ApplyArguments {
    fn context(&self) -> WorkspaceContext {
        self.context.clone()
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WorkspaceChange {
    CreateCard {
        type_id: String,
        title: String,
        #[serde(default)]
        summary: String,
        bounds: Rect,
    },
    UpdateCard {
        concept_id: u32,
        #[serde(default)]
        type_id: Option<String>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        summary: Option<String>,
        #[serde(default)]
        details: Option<String>,
        #[serde(default)]
        attributes: Option<BTreeMap<String, String>>,
    },
    DeleteCard {
        concept_id: u32,
    },
    MoveCards {
        concept_ids: Vec<u32>,
        delta: Point,
    },
    ResizeCard {
        concept_id: u32,
        bounds: Rect,
    },
    CreateRelationship {
        source_id: u32,
        target_id: u32,
        relationship_kind: RelationshipKind,
    },
    ChangeRelationshipKind {
        relationship_id: String,
        relationship_kind: RelationshipKind,
    },
    RemoveRelationship {
        relationship_id: String,
    },
    CreateBoundary {
        title: String,
        #[serde(default)]
        summary: String,
        bounds: Rect,
    },
    UpdateBoundary {
        boundary_id: String,
        title: String,
        summary: String,
    },
    MoveBoundary {
        boundary_id: String,
        delta: Point,
    },
    ResizeBoundary {
        boundary_id: String,
        bounds: Rect,
    },
    RemoveBoundary {
        boundary_id: String,
    },
    CreateZoomLayer {
        title: String,
        start_percent: f32,
        end_percent: f32,
        fade_percent: f32,
    },
    UpdateZoomLayer {
        layer_id: String,
        title: String,
        start_percent: f32,
        end_percent: f32,
        fade_percent: f32,
        card_scale: f32,
    },
    ReorderZoomLayer {
        layer_id: String,
        destination_index: usize,
    },
    DeleteZoomLayer {
        layer_id: String,
    },
    SetCardLayerMode {
        concept_id: u32,
        mode: CardLayerMode,
    },
}

impl WorkspaceChange {
    fn label(&self) -> &'static str {
        match self {
            Self::CreateCard { .. } => "create_card",
            Self::UpdateCard { .. } => "update_card",
            Self::DeleteCard { .. } => "delete_card",
            Self::MoveCards { .. } => "move_cards",
            Self::ResizeCard { .. } => "resize_card",
            Self::CreateRelationship { .. } => "create_relationship",
            Self::ChangeRelationshipKind { .. } => "change_relationship_kind",
            Self::RemoveRelationship { .. } => "remove_relationship",
            Self::CreateBoundary { .. } => "create_boundary",
            Self::UpdateBoundary { .. } => "update_boundary",
            Self::MoveBoundary { .. } => "move_boundary",
            Self::ResizeBoundary { .. } => "resize_boundary",
            Self::RemoveBoundary { .. } => "remove_boundary",
            Self::CreateZoomLayer { .. } => "create_zoom_layer",
            Self::UpdateZoomLayer { .. } => "update_zoom_layer",
            Self::ReorderZoomLayer { .. } => "reorder_zoom_layer",
            Self::DeleteZoomLayer { .. } => "delete_zoom_layer",
            Self::SetCardLayerMode { .. } => "set_card_layer_mode",
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
enum CardLayerMode {
    Automatic,
    Manual { layer_ids: Vec<String> },
}

impl CardLayerMode {
    fn into_domain(self) -> SemanticZoomCardLayerMode {
        match self {
            Self::Automatic => SemanticZoomCardLayerMode::Automatic,
            Self::Manual { layer_ids } => SemanticZoomCardLayerMode::Manual { layer_ids },
        }
    }
}

#[derive(Serialize)]
struct WorkspaceOverview {
    contract_version: u32,
    workspace_path: String,
    workspace_digest: String,
    workspace_id: String,
    workspace_title: String,
    schema_version: u32,
    current_version_id: String,
    custom_types: Vec<infinite_note_canvas_canvas::CustomType>,
    versions: Vec<WorkspaceVersionOverview>,
    write_enabled: bool,
}

#[derive(Serialize)]
struct WorkspaceVersionOverview {
    id: String,
    title: String,
    concept_count: usize,
    relationship_count: usize,
    lenses: Vec<LensOverview>,
}

impl From<&infinite_note_canvas_canvas::WorkspaceVersion> for WorkspaceVersionOverview {
    fn from(version: &infinite_note_canvas_canvas::WorkspaceVersion) -> Self {
        Self {
            id: version.id.clone(),
            title: version.title.clone(),
            concept_count: version.concepts.len(),
            relationship_count: version.relationships.len(),
            lenses: version.lenses.iter().map(LensOverview::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct LensOverview {
    id: String,
    title: String,
    placed_card_count: usize,
    visual_boundary_count: usize,
    semantic_zoom_layers: Vec<ZoomLayerOverview>,
}

impl From<&WorkspaceLens> for LensOverview {
    fn from(lens: &WorkspaceLens) -> Self {
        Self {
            id: lens.id.clone(),
            title: lens.title.clone(),
            placed_card_count: lens.placements.len(),
            visual_boundary_count: lens.visual_boundaries.len(),
            semantic_zoom_layers: lens
                .semantic_zoom_layers
                .iter()
                .map(|layer| ZoomLayerOverview {
                    id: layer.id.clone(),
                    title: layer.title.clone(),
                    concept_count: layer.concept_ids.len(),
                    relationship_count: layer.relationship_ids.len(),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ZoomLayerOverview {
    id: String,
    title: String,
    concept_count: usize,
    relationship_count: usize,
}

#[derive(Serialize)]
struct LensReadOutput {
    contract_version: u32,
    version_id: String,
    lens_id: String,
    total_card_count: usize,
    offset: usize,
    next_offset: Option<usize>,
    cards: Vec<LensCard>,
    relationships: Vec<TypedRelationship>,
    visual_boundaries: Vec<VisualBoundary>,
    semantic_zoom_layers: Vec<infinite_note_canvas_canvas::SemanticZoomLayer>,
}

#[derive(Serialize)]
struct LensCard {
    concept: WorkspaceConcept,
    bounds: Rect,
    layer_mode: SemanticZoomCardLayerMode,
    visible_layer_ids: Vec<String>,
}

#[derive(Serialize)]
struct WorkspaceQueryOutput {
    contract_version: u32,
    version_id: String,
    root_concept_id: u32,
    direction: QueryDirection,
    depth: u32,
    concepts: Vec<WorkspaceConcept>,
    relationships: Vec<TypedRelationship>,
}

#[derive(Serialize)]
struct SemanticZoomPreview {
    contract_version: u32,
    version_id: String,
    lens_id: String,
    scale: f32,
    cards: Vec<SemanticZoomCardOutput>,
    relationships: Vec<SemanticZoomRelationshipOutput>,
    layers: Vec<SemanticZoomLayerOutput>,
}

#[derive(Serialize)]
struct SemanticZoomCardOutput {
    concept_id: u32,
    bounds: Rect,
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

#[derive(Serialize)]
struct SemanticZoomLayerOutput {
    id: String,
    title: String,
    opacity: f32,
}

#[derive(Serialize)]
struct WorkspaceChangeOutput {
    contract_version: u32,
    change_kind: &'static str,
    workspace_path: String,
    workspace_digest: String,
    version_id: String,
    lens_id: String,
    selected_concept_ids: Vec<u32>,
    concept_count: usize,
}

#[derive(Serialize)]
struct WorkspaceCreateOutput {
    contract_version: u32,
    workspace_path: String,
    workspace_digest: String,
    workspace_id: String,
    workspace_title: String,
    version_id: String,
    lens_id: String,
}

#[cfg(test)]
mod tests {
    use super::{AgentServer, McpOptions};
    use infinite_note_canvas_canvas::WorkspaceDocument;
    use serde_json::{Value, json};
    use std::{env, fs};

    fn test_server(allow_write: bool) -> (AgentServer, std::path::PathBuf) {
        let workspace_path = env::temp_dir().join(format!(
            "infinite-note-canvas-mcp-{}-{}.json",
            std::process::id(),
            if allow_write { "write" } else { "read" }
        ));
        fs::write(
            &workspace_path,
            WorkspaceDocument::sample().to_pretty_json().unwrap(),
        )
        .unwrap();
        let server = AgentServer::open(McpOptions {
            workspace_path: fs::canonicalize(&workspace_path).unwrap(),
            allow_write,
        })
        .unwrap();
        (server, workspace_path)
    }

    fn tool_call(name: &str, arguments: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
        })
    }

    #[test]
    fn read_only_server_exposes_bounded_inspection_tools() {
        let (server, workspace_path) = test_server(false);
        let response = server
            .handle(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list",
            }))
            .unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        assert!(
            tools
                .iter()
                .any(|tool| tool["name"] == "workspace_overview")
        );
        assert!(
            !tools
                .iter()
                .any(|tool| tool["name"] == "apply_workspace_change")
        );

        let result = server
            .handle(tool_call("workspace_overview", json!({})))
            .unwrap();
        assert_eq!(
            result["result"]["structuredContent"]["write_enabled"],
            false
        );
        assert_eq!(
            result["result"]["structuredContent"]["workspace_title"],
            "Acme Platform Landscape"
        );
        fs::remove_file(workspace_path).unwrap();
    }

    #[test]
    fn write_enabled_server_creates_its_scoped_workspace_once() {
        let workspace_path = env::temp_dir().join(format!(
            "infinite-note-canvas-mcp-create-{}.json",
            std::process::id()
        ));
        let _ = fs::remove_file(&workspace_path);
        let server = AgentServer::open(McpOptions {
            workspace_path: workspace_path.clone(),
            allow_write: true,
        })
        .unwrap();

        let before_create = server
            .handle(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list",
            }))
            .unwrap();
        let tools = before_create["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "create_workspace");

        let created = server
            .handle(tool_call("create_workspace", json!({ "title": "jcwa" })))
            .unwrap();
        assert_eq!(
            created["result"]["structuredContent"]["workspace_title"],
            "jcwa"
        );
        assert_eq!(
            created["result"]["structuredContent"]["workspace_id"],
            "jcwa"
        );
        let document =
            WorkspaceDocument::parse_json(&fs::read_to_string(&workspace_path).unwrap()).unwrap();
        assert_eq!(document.title, "jcwa");
        assert_eq!(document.default_lens_id("current").unwrap(), "canvas");

        let second_create = server
            .handle(tool_call(
                "create_workspace",
                json!({ "title": "replacement" }),
            ))
            .unwrap();
        assert!(second_create["result"]["isError"].as_bool().unwrap());
        assert_eq!(
            WorkspaceDocument::parse_json(&fs::read_to_string(&workspace_path).unwrap())
                .unwrap()
                .title,
            "jcwa"
        );
        fs::remove_file(workspace_path).unwrap();
    }

    #[test]
    fn write_enabled_server_creates_cards_through_the_safe_session_path() {
        let (server, workspace_path) = test_server(true);
        let response = server
            .handle(tool_call(
                "apply_workspace_change",
                json!({
                    "change": {
                        "kind": "create_card",
                        "type_id": "service",
                        "title": "Agent-created service",
                        "summary": "Created through MCP.",
                        "bounds": {
                            "origin": { "x": 24.0, "y": 48.0 },
                            "size": { "x": 240.0, "y": 120.0 }
                        }
                    }
                }),
            ))
            .unwrap();
        let concept_id = response["result"]["structuredContent"]["selected_concept_ids"]
            .as_array()
            .and_then(|ids| ids.first())
            .and_then(Value::as_u64)
            .expect("created card id") as u32;

        let document =
            WorkspaceDocument::parse_json(&fs::read_to_string(&workspace_path).unwrap()).unwrap();
        assert!(document.current_state.concepts.iter().any(|concept| {
            concept.title == "Agent-created service" && concept.type_id == "service"
        }));

        let deletion = server
            .handle(tool_call(
                "apply_workspace_change",
                json!({ "change": { "kind": "delete_card", "concept_id": concept_id } }),
            ))
            .unwrap();
        assert_eq!(
            deletion["result"]["structuredContent"]["change_kind"],
            "delete_card"
        );
        let after_delete =
            WorkspaceDocument::parse_json(&fs::read_to_string(&workspace_path).unwrap()).unwrap();
        assert!(
            after_delete
                .current_state
                .concepts
                .iter()
                .all(|concept| concept.id != concept_id)
        );
        fs::remove_file(workspace_path).unwrap();
    }
}
