# Security Policy

## Supported Versions

The project has no stable release yet. Before the first release, security fixes
apply to the latest commit on the default branch.

| Version | Supported |
| --- | --- |
| Latest default-branch source | Yes |
| Earlier commits and forks | No |

## Security Boundary

Reason Room is a local-first application. It does not provide user
accounts, cloud synchronization, telemetry, or a network API. The local browser
build serves generated files on `127.0.0.1`.

Desktop workspaces are ordinary JSON files. Browser workspaces are stored in
the browser's local storage for the site origin and can be downloaded as JSON.
The application does not encrypt these files or browser data. Protect the
device, browser profile, and workspace directory according to the sensitivity
of the data they contain.

The desktop save path verifies candidate data and keeps a `.bak` backup. These
controls help recover from an interrupted or failed save. They are not a
replacement for a separate backup strategy.

## Report A Vulnerability

After this repository is published on GitHub, use the repository's **Security**
tab and select **Report a vulnerability**. Do not open a public issue,
discussion, or pull request for a suspected vulnerability.

If private vulnerability reporting is not enabled, contact the repository owner
through an agreed private channel. Do not disclose the details in public while
you wait for a response.

Include these details in a report:

- A clear description of the issue and its possible impact.
- Steps to reproduce the issue or a small proof of concept.
- The affected commit, release, operating system, and browser when relevant.
- Any suggested mitigation or fix.

Remove private workspace data, tokens, credentials, and personal information
from reports and proof-of-concept files.

## Disclosure

Please allow maintainers time to investigate and prepare a fix before public
disclosure. The maintainer should publish an advisory, affected-version details,
and remediation guidance when a fix is available.

## Before Publishing

Enable GitHub private vulnerability reporting for this repository. Add a private
security contact if the repository owner does not intend to use GitHub reports.
Update this policy when the project publishes its first supported release.
