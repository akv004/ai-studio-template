# Tools API Reference

## Overview

The AI Agent provides tools for shell commands, filesystem access, and browser automation.

**Base URL:** `http://localhost:8765`

**Safety Modes:** Controlled by `TOOLS_MODE` environment variable:
- `sandboxed` - Only whitelisted commands, workspace-only filesystem
- `restricted` (default) - Block dangerous commands, block sensitive paths
- `full` - No restrictions (use with caution)

---

## Shell Tool

### POST `/tools/shell`

Execute shell commands.

**Request:**
```json
{
  "command": "ls -la",
  "timeout": 30.0,
  "cwd": "/path/to/dir"
}
```

**Response:**
```json
{
  "command": "ls -la",
  "stdout": "total 24\ndrwxr-xr-x ...",
  "stderr": "",
  "return_code": 0,
  "timed_out": false
}
```

**Blocked in restricted mode:**
- `rm -rf /`, `sudo rm`, `mkfs`, fork bombs, etc.

**Allowed in sandboxed mode:**
- `ls`, `cat`, `grep`, `find`, `pwd`, `echo`, `git`, `python`, `node`, `curl`, etc.

---

## Filesystem Tool

### POST `/tools/filesystem`

Perform file operations.

**Actions:** `read`, `write`, `append`, `delete`, `list_dir`, `mkdir`, `exists`, `copy`, `move`

**Request (read):**
```json
{
  "action": "read",
  "path": "config.json"
}
```

**Request (write):**
```json
{
  "action": "write",
  "path": "output.txt",
  "content": "Hello, world!"
}
```

**Request (copy/move):**
```json
{
  "action": "copy",
  "path": "source.txt",
  "dest": "destination.txt"
}
```

**Response:**
```json
{
  "success": true,
  "action": "read",
  "path": "/full/path/to/file",
  "data": "file contents...",
  "error": null
}
```

**Blocked paths in restricted mode:**
- `/etc`, `/usr`, `/bin`, `~/.ssh`, `~/.aws`, etc.

---

## Browser Tool

Playwright-based browser automation.

### POST `/tools/browser/start`

Start browser instance.

### POST `/tools/browser/stop`

Stop browser instance.

### POST `/tools/browser`

Perform browser actions.

**Actions:** `navigate`, `screenshot`, `click`, `fill`, `extract_text`, `get_html`, `evaluate`, `wait_for`

**Request (navigate):**
```json
{
  "action": "navigate",
  "url": "https://example.com"
}
```

**Request (fill form):**
```json
{
  "action": "fill",
  "selector": "#email",
  "value": "user@example.com"
}
```

**Request (extract text):**
```json
{
  "action": "extract_text",
  "selector": "h1"
}
```

**Request (evaluate JS):**
```json
{
  "action": "evaluate",
  "script": "document.title"
}
```

**Response (screenshot):**
```json
{
  "success": true,
  "action": "screenshot",
  "screenshot": "base64-encoded-image...",
  "error": null
}
```

---

## Examples

### Run a shell command
```bash
curl -X POST http://localhost:8765/tools/shell \
  -H "Content-Type: application/json" \
  -d '{"command": "echo Hello, World!"}'
```

### Read a file
```bash
curl -X POST http://localhost:8765/tools/filesystem \
  -H "Content-Type: application/json" \
  -d '{"action": "read", "path": "README.md"}'
```

### Take a screenshot
```bash
# Start browser
curl -X POST http://localhost:8765/tools/browser/start

# Navigate and screenshot
curl -X POST http://localhost:8765/tools/browser \
  -H "Content-Type: application/json" \
  -d '{"action": "navigate", "url": "https://example.com"}'

curl -X POST http://localhost:8765/tools/browser \
  -H "Content-Type: application/json" \
  -d '{"action": "screenshot"}'

# Stop browser
curl -X POST http://localhost:8765/tools/browser/stop
```
