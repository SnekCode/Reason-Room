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

Official setup references: [Codex skills](https://developers.openai.com/codex/skills/)
and [Codex MCP configuration](https://developers.openai.com/codex/mcp/).
