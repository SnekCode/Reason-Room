# Connect Reason Room to an Agent

The skill supplies workflow instructions. The separately configured local MCP
server supplies the tools and access to one workspace. No API key, account,
cloud service, or open network port is required by Reason Room's MCP server.

## Build the CLI

Clone `https://github.com/SnekCode/Reason-Room.git` and follow the repository's
`docs/installation.md` for Rust 1.97.1 and developer tools. From that checkout:

```sh
cargo build --locked --release --package infinite-note-canvas-cli
```

The executable is `target/release/infinite-note-canvas-cli` (`.exe` on Windows).
The legacy binary name is intentional during the Reason Room rebrand. The
supported desktop launcher is `./start.sh`; the CLI's `mcp` subcommand is a
headless agent adapter and does not open the app.

## Select a Workspace and Connect

Use absolute paths to the executable and the user's intended workspace JSON.
Keep writable workspaces outside the source checkout. For read-only use, the
file must already exist and validate. For a new document, select a path in an
existing directory and explicitly enable writes.

Codex configuration, using a read-only connection by default:

```toml
[mcp_servers.reason_room]
command = "/absolute/path/to/Reason-Room/target/release/infinite-note-canvas-cli"
args = ["mcp", "--workspace", "/absolute/path/to/workspace.json"]
```

To enable requested editing, add `"--allow-write"` to `args` and reconnect the
server. Use `~/.codex/config.toml` for user configuration or a trusted project's
`.codex/config.toml` for a project-specific connection. Preserve unrelated
configuration. Another option is the client's MCP settings UI: choose STDIO,
then enter the command and arguments separately. This server has no HTTP URL.

Other MCP clients commonly use this JSON configuration shape; follow the
client's own configuration-file location:

```json
{
  "mcpServers": {
    "reason_room": {
      "command": "/absolute/path/to/Reason-Room/target/release/infinite-note-canvas-cli",
      "args": ["mcp", "--workspace", "/absolute/path/to/workspace.json"]
    }
  }
}
```

After connecting, discover tools and call `workspace_overview`. Confirm the
returned `workspace_path` and `write_enabled`. For a missing write-enabled
workspace, use `create_workspace` first, then refresh discovery. The server
currently changes its tool list after creation without emitting a list-change
notification, so some clients need a reconnect to discover the normal tools.

Use separate named MCP connections for multiple canvases. A tool call cannot
switch this server to another file. Do not add `--allow-write` simply to make
an existing read-only inspection work.

## Install the Skill Folder

In the webview, **File > Download usage skill** downloads `reason-room-skill.zip`.
Extract it to get the complete `reason-room/` folder, then install that folder
as described below. The ZIP includes this setup guide and the MCP reference.

The portable bundle is the `reason-room/` folder containing `SKILL.md`,
`agents/openai.yaml`, `references/`, and `LICENSE`. Keep those together.

For current Codex user-scope discovery, copy that folder into
`~/.agents/skills/reason-room/`. From the Reason Room checkout:

```sh
mkdir -p "$HOME/.agents/skills"
# If reason-room already exists there, review it before replacing anything.
test ! -e "$HOME/.agents/skills/reason-room" && cp -R skills/reason-room "$HOME/.agents/skills/"
```

For a particular project, use that project's `.agents/skills/reason-room/`
instead. Some Codex installations use `$CODEX_HOME/skills` or
`~/.codex/skills`; use the local skill directory your installation exposes and
install only one copy to avoid duplicate discovery. Other skill-capable
clients can use the same folder in their documented skills location.

Ask the agent to use `$reason-room` after installing. If it is not discovered,
restart the client. Installing the skill does not automatically configure MCP
or select a workspace.

Examples:

- “Use $reason-room to explain the outgoing dependencies of the Billing API.”
- “Use $reason-room to rename the selected service card and retain its details.”
- “Use $reason-room to check what is visible at 25% zoom.”

## Use MCP with the Webview

The webview runs the browser application. Its hosted page does not expose an
MCP endpoint: configure the local stdio server above on the machine holding
the workspace JSON and running the agent's MCP client. The server cannot
access browser storage or control the page.

1. For an existing browser canvas, use **File > Save** to download its JSON.
   Keep the file in a writable folder outside the source checkout. If the
   intended canvas is already a local JSON file, use that file instead.
2. Configure MCP with that file's absolute path. Discover the tools and call
   `workspace_overview` to confirm `workspace_path` and `write_enabled` before
   applying requested changes. Enable `--allow-write` only for requested edits.
3. In the webview, use **File > Open and refresh from file...** and select the
   same JSON file. On browsers supporting file handles, the app polls the
   selected file and defers incoming changes while editing is active. After
   each MCP edit, read back the affected Lens to verify the saved result.
4. If the picker falls back to ordinary **Open canvas...**, it imports a copy
   without live refresh. Reopen the file after MCP edits to see the result.
   Re-importing replaces the active browser canvas; save browser work first.

Browser **Save** and **Save As...** download copies; they do not write back to
the watched file. Avoid simultaneous browser and agent edits. To hand browser
edits back to MCP, save a copy, deliberately replace the selected local file
or configure MCP to use the new path, then confirm the overview again. Do not
assume that a browser download overwrote the MCP workspace: browsers may add
a filename suffix. Browser-local storage and a local MCP file are separate.

Official setup references: [Codex skills](https://developers.openai.com/codex/skills/)
and [Codex MCP configuration](https://developers.openai.com/codex/mcp/).
