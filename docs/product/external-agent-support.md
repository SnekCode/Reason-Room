# External Agent Support

## Current Interface

External agents use a local Model Context Protocol (MCP) server over standard
input and output. The server owns one explicitly selected workspace file. It
does not open a network port, discover files, or control an application window.

This is the initial adapter for external agents. It preserves the local-first
product model. It gives an agent one small interface for reading a map and a
separate, explicitly enabled interface for one validated workspace change.

## Persistence Boundary

The server reuses the workspace-session crate's typed actions and local
save/backup/verification path. It opens and validates the selected file for
each call. The desktop app can observe resulting file changes; the server has
no live UI bridge and does not share the desktop undo history.

## Installable Agent Skill

The portable [Reason Room skill](../../skills/reason-room/SKILL.md) teaches an
agent how to select context, inspect a map, and verify requested MCP edits.
Its [tool reference](../../skills/reason-room/references/mcp-contract.md)
covers all six tools and all 18 typed change kinds. Its
[setup guide](../../skills/reason-room/references/setup.md) covers skill
installation and separate MCP configuration for Codex and other MCP clients.

The skill folder is `skills/reason-room/`. Copy the complete folder into the
client's skill directory, or ask a compatible skill installer to install that
path from `https://github.com/SnekCode/Reason-Room`. Installing the skill does
not build the CLI, select a workspace, or enable write access.

## Start an MCP Server

Build the agent CLI:

```sh
cargo build --locked --release --package infinite-note-canvas-cli
```

Configure an MCP client with an absolute CLI path and an absolute workspace
path. This example gives the agent read-only access to one existing file.

```json
{
  "mcpServers": {
    "reason_room": {
      "command": "/absolute/path/to/target/release/infinite-note-canvas-cli",
      "args": [
        "mcp",
        "--workspace",
        "/absolute/path/to/workspace.json"
      ]
    }
  }
}
```

Add `"--allow-write"` to the argument list when the user wants editing, then
reconnect. The server rejects write tools unless that option is present when
the process starts. The selected file's parent directory must already exist.

When a write-enabled server starts with a path that does not yet exist, its
only initial tool is `create_workspace`. That tool can create an empty document
at that exact configured path. It never selects another location and never
replaces an existing file. After creation, the normal workspace tools become
available in the same MCP session.

For Codex, add this table to `~/.codex/config.toml` or to a project-scoped
`.codex/config.toml`:

```toml
[mcp_servers.reason_room]
command = "/absolute/path/to/target/release/infinite-note-canvas-cli"
args = [
  "mcp",
  "--workspace",
  "/absolute/path/to/workspace.json",
]
```

Codex supports standard-input/output servers through `command` and `args` in
an `[mcp_servers.<name>]` table. See the [official Codex MCP documentation](https://developers.openai.com/codex/mcp/).

`./start.sh` remains the only supported command to start the desktop
application. The MCP command is an agent adapter. It does not start the app.

## Tools

- `create_workspace` creates an empty document at the configured path only
  when it does not already exist. It is available only with `--allow-write`.
- `workspace_overview` shows workspace identity, versions, Lenses, types,
  boundary counts, and layer summaries.
- `read_lens` returns a page of up to 250 cards with positions, direct
  relationships, boundaries, and semantic zoom settings.
- `query_workspace` traverses typed relationships from one card.
- `preview_semantic_zoom` reports a Lens projection for one zoom scale.
- `apply_workspace_change` is available only with `--allow-write`. It applies
  one validated change at a time.

The write tool supports create, update, and delete workflows for cards,
relationships, visual boundaries, and semantic-zoom layers. It reports the
saved workspace digest after every successful change.

## Agent Workflow

1. Call `workspace_overview`.
2. Call `read_lens` or `query_workspace` for the relevant context.
3. Use `preview_semantic_zoom` if the request involves a zoom layer.
4. Call one `apply_workspace_change`.
5. Read the affected Lens again before the next change.

The server opens and validates the workspace for each tool call. This avoids
using a stale in-memory document after another local process changes the file.

## Refresh and Editing Boundaries

The desktop app polls its open workspace for external changes and defers
refresh while a dialog, gesture, or editor draft is active. It can display
agent changes after editing becomes idle. Avoid concurrent human and agent
authoring: per-call file reload and save verification are not a shared lock or
a multi-operation transaction.

The browser's ordinary Open action imports a separate copy. For external file
updates, use its live-refresh connection where supported, or explicitly import
the saved JSON again. Changing a desktop JSON file does not automatically
update a hosted browser canvas.

A server for a missing path initially advertises only `create_workspace`.
After creation, refresh tool discovery. Clients that cache discovery may need
to reconnect because this server does not emit a tool-list change notification.

MCP has no UI controls, undo/redo, automatic layout, version/Lens creation, or
custom-type creation tools. A multi-step edit can succeed in part; report saved
changes and any failure accurately rather than implying an automatic rollback.
