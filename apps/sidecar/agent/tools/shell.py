"""
Shell Tool
==========
Execute shell commands with safety controls.
"""

import asyncio
import os
import shlex
from typing import Optional
from dataclasses import dataclass


@dataclass
class ShellResult:
    """Result of a shell command execution"""
    command: str
    stdout: str
    stderr: str
    return_code: int
    timed_out: bool = False


class ShellTool:
    """
    Execute shell commands with configurable safety.
    
    Safety levels:
    - sandboxed: Only allow whitelisted commands
    - restricted: Block dangerous commands (rm -rf, sudo, etc.)
    - full: Allow all commands (use with caution)
    
    Example:
        shell = ShellTool(mode="restricted")
        result = await shell.run("ls -la")
        print(result.stdout)
    """
    
    # Commands blocked in restricted mode
    DANGEROUS_PATTERNS = [
        "rm -rf /",
        "rm -rf ~",
        "sudo rm",
        "mkfs",
        "> /dev/",
        "dd if=",
        ":(){ :|:& };:",  # fork bomb
        "chmod -R 777 /",
        "chown -R",
    ]
    
    # Commands allowed in sandboxed mode
    SAFE_COMMANDS = [
        "ls", "cat", "head", "tail", "grep", "find", "pwd", "echo",
        "date", "whoami", "uname", "env", "which", "wc", "sort",
        "uniq", "diff", "file", "stat", "du", "df", "ps", "top",
        "curl", "wget", "python", "python3", "node", "npm", "git",
    ]
    
    def __init__(
        self,
        mode: str = "restricted",  # sandboxed | restricted | full
        timeout: float = 30.0,
        working_dir: Optional[str] = None,
        env: Optional[dict] = None,
    ):
        self.mode = mode
        self.timeout = timeout
        self.working_dir = working_dir or os.getcwd()
        self.env = {**os.environ, **(env or {})}
    
    def _is_safe(self, command: str) -> tuple[bool, str]:
        """Check if command is safe to execute"""
        if self.mode == "full":
            return True, ""
        
        # Check for dangerous patterns
        if self.mode in ("restricted", "sandboxed"):
            for pattern in self.DANGEROUS_PATTERNS:
                if pattern in command:
                    return False, f"Blocked dangerous pattern: {pattern}"
        
        # In sandboxed mode, only allow whitelisted commands
        if self.mode == "sandboxed":
            try:
                parts = shlex.split(command)
                base_cmd = os.path.basename(parts[0]) if parts else ""
                if base_cmd not in self.SAFE_COMMANDS:
                    return False, f"Command '{base_cmd}' not in allowed list"
            except ValueError:
                return False, "Invalid command syntax"
        
        return True, ""
    
    async def run(
        self,
        command: str,
        timeout: Optional[float] = None,
        cwd: Optional[str] = None,
    ) -> ShellResult:
        """
        Execute a shell command.
        
        Args:
            command: Shell command to execute
            timeout: Override default timeout
            cwd: Override working directory
            
        Returns:
            ShellResult with stdout, stderr, and return code
        """
        # Safety check
        is_safe, reason = self._is_safe(command)
        if not is_safe:
            return ShellResult(
                command=command,
                stdout="",
                stderr=f"Command blocked: {reason}",
                return_code=-1,
            )
        
        timeout = timeout or self.timeout
        cwd = cwd or self.working_dir
        
        try:
            process = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=cwd,
                env=self.env,
            )
            
            try:
                stdout, stderr = await asyncio.wait_for(
                    process.communicate(),
                    timeout=timeout,
                )
                return ShellResult(
                    command=command,
                    stdout=stdout.decode("utf-8", errors="replace"),
                    stderr=stderr.decode("utf-8", errors="replace"),
                    return_code=process.returncode or 0,
                )
            except asyncio.TimeoutError:
                process.kill()
                await process.wait()
                return ShellResult(
                    command=command,
                    stdout="",
                    stderr=f"Command timed out after {timeout}s",
                    return_code=-1,
                    timed_out=True,
                )
        
        except Exception as e:
            return ShellResult(
                command=command,
                stdout="",
                stderr=str(e),
                return_code=-1,
            )
    
    async def run_script(
        self,
        script: str,
        interpreter: str = "bash",
        timeout: Optional[float] = None,
    ) -> ShellResult:
        """Execute a multi-line script"""
        import tempfile
        
        with tempfile.NamedTemporaryFile(mode="w", suffix=".sh", delete=False) as f:
            f.write(script)
            script_path = f.name
        
        try:
            result = await self.run(f"{interpreter} {script_path}", timeout=timeout)
            return result
        finally:
            os.unlink(script_path)
