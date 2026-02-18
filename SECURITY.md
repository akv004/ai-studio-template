# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Architecture Security

AI Studio is designed with security as a core architectural principle:

- **Local-first**: All data stays on your machine in SQLite. No cloud accounts, no telemetry.
- **Tauri security boundary**: The UI (React) never communicates with the Python sidecar directly. All requests go through Tauri IPC, where the Rust layer adds authentication tokens and enforces CORS.
- **Sidecar authentication**: The Python sidecar is spawned by Tauri with a random auth token. Every non-health request requires an `x-ai-studio-token` header. This prevents localhost port-scanning attacks.
- **Tool approval**: Every tool execution (shell commands, file access, MCP tools) goes through an approval workflow. Users can configure per-tool approval rules (always ask, auto-approve, deny).
- **Plugin permissions**: Plugins declare required permissions in their manifest (`plugin.json`). The app shows these before enabling.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT open a public GitHub issue** for security vulnerabilities
2. Email: Open a private security advisory via GitHub's [Security Advisories](https://github.com/akv004/ai-studio-template/security/advisories/new)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will acknowledge receipt within 48 hours and aim to release a fix within 7 days for critical vulnerabilities.

## Scope

The following are in scope for security reports:

- Authentication bypass (sidecar token)
- Command injection via tool execution
- Path traversal in file operations
- XSS in the Tauri webview
- MCP server connection security
- Plugin sandbox escapes
- SQLite injection

The following are **not** in scope:

- Vulnerabilities in third-party LLM provider APIs (report to the provider)
- Issues requiring physical access to the machine
- Social engineering attacks
- Denial of service on localhost
