# External Agent Support

## Decision

External agents use a local Model Context Protocol (MCP) server over standard
input and output. The server owns one explicitly selected workspace file. It
does not open a network port, discover files, or control an application window.

This is the initial adapter for external agents. It preserves the local-first
product model. It gives an agent one small interface for reading a map and a
separate, explicitly enabled interface for one validated workspace change.

## Why This Shape

PhotoCraft offers a headless MCP server and can optionally bridge to a running
desktop process. Its UI, command line, and MCP tools share command handling.
It also scopes headless file access to configured roots.

Reason Room follows the useful parts of that design:

- MCP uses standard input and output.
- The agent receives access to one stated local workspace path.
- Read and write operations reuse the application's typed workspace actions.
- A change uses the same local save, backup, validation, and recovery path as
  interactive authoring.

The desktop bridge is intentionally not included. It would require a network
listener or another live-process transport. The current product decision is
local files without network services.

Sources: [PhotoCraft MCP command](https://github.com/storytold/photocraft/blob/952c328b3d849eaa110cff9c7ab9977389590ccc/apps/photocraft-cli/src/automation.rs) and [its automation arguments](https://github.com/storytold/photocraft/blob/952c328b3d849eaa110cff9c7ab9977389590ccc/apps/photocraft-cli/src/args.rs).

## Start an MCP Server

Build the agent CLI:

```sh
cargo build --release -p infinite-note-canvas-cli
```

Configure an MCP client with an absolute CLI path and an absolute workspace
path. This example gives the agent read and write access to only one file.

```json
{
  "mcpServers": {
    "infinite-note-canvas": {
      "command": "/absolute/path/to/target/release/infinite-note-canvas-cli",
      "args": [
        "mcp",
        "--workspace",
        "/absolute/path/to/workspace.json",
        "--allow-write"
      ]
    }
  }
}
```

Remove `--allow-write` to offer read-only tools. The server rejects write tools
unless that option is present when the process starts.

When a write-enabled server starts with a path that does not yet exist, its
only initial tool is `create_workspace`. That tool can create an empty document
at that exact configured path. It never selects another location and never
replaces an existing file. After creation, the normal workspace tools become
available in the same MCP session.

For Codex, add this table to `~/.codex/config.toml` or to a project-scoped
`.codex/config.toml`:

```toml
[mcp_servers.infinite_note_canvas]
command = "/absolute/path/to/target/release/infinite-note-canvas-cli"
args = [
  "mcp",
  "--workspace",
  "/absolute/path/to/workspace.json",
  "--allow-write",
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

The write tool supports create, read, update, and delete workflows for cards,
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

## Current Limitation

The desktop app does not yet reload a workspace that an agent changes. Do not
edit the same workspace in the desktop app while an agent has write access.
Close and reopen the desktop app after the agent finishes its changes.
