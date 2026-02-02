"""
Filesystem Tool
===============
File and directory operations with safety controls.
"""

import os
import shutil
from pathlib import Path
from typing import Optional, Union
from dataclasses import dataclass


@dataclass
class FileResult:
    """Result of a filesystem operation"""
    success: bool
    action: str
    path: str
    data: Optional[str] = None
    error: Optional[str] = None


class FilesystemTool:
    """
    Filesystem operations with configurable safety.
    
    Safety levels:
    - sandboxed: Only allow operations within workspace
    - restricted: Block sensitive paths (/etc, /usr, ~/.ssh, etc.)
    - full: Allow all paths (use with caution)
    
    Example:
        fs = FilesystemTool(workspace="/home/user/projects")
        result = await fs.read("config.json")
        result = await fs.write("output.txt", "Hello, world!")
        result = await fs.list_dir(".")
    """
    
    # Paths blocked in restricted mode
    SENSITIVE_PATHS = [
        "/etc", "/usr", "/bin", "/sbin", "/boot", "/sys", "/proc",
        "~/.ssh", "~/.gnupg", "~/.aws", "~/.config/gcloud",
        "/var/log", "/var/lib",
    ]
    
    def __init__(
        self,
        mode: str = "restricted",  # sandboxed | restricted | full
        workspace: Optional[str] = None,
        max_file_size: int = 10 * 1024 * 1024,  # 10MB
    ):
        self.mode = mode
        self.workspace = Path(workspace or os.getcwd()).resolve()
        self.max_file_size = max_file_size
    
    def _resolve_path(self, path: str) -> Path:
        """Resolve path relative to workspace"""
        p = Path(path)
        if not p.is_absolute():
            p = self.workspace / p
        return p.resolve()
    
    def _is_allowed(self, path: Path) -> tuple[bool, str]:
        """Check if path is allowed"""
        path_str = str(path)
        
        if self.mode == "full":
            return True, ""
        
        # In sandboxed mode, must be within workspace
        if self.mode == "sandboxed":
            try:
                path.relative_to(self.workspace)
                return True, ""
            except ValueError:
                return False, f"Path '{path}' is outside workspace"
        
        # In restricted mode, block sensitive paths
        if self.mode == "restricted":
            for sensitive in self.SENSITIVE_PATHS:
                expanded = os.path.expanduser(sensitive)
                if path_str.startswith(expanded):
                    return False, f"Access to '{sensitive}' is blocked"
        
        return True, ""
    
    def read(self, path: str, encoding: str = "utf-8") -> FileResult:
        """Read file contents"""
        resolved = self._resolve_path(path)
        
        allowed, reason = self._is_allowed(resolved)
        if not allowed:
            return FileResult(success=False, action="read", path=str(resolved), error=reason)
        
        try:
            if not resolved.exists():
                return FileResult(success=False, action="read", path=str(resolved), error="File not found")
            
            if resolved.stat().st_size > self.max_file_size:
                return FileResult(
                    success=False, action="read", path=str(resolved),
                    error=f"File too large (max {self.max_file_size} bytes)"
                )
            
            content = resolved.read_text(encoding=encoding)
            return FileResult(success=True, action="read", path=str(resolved), data=content)
        
        except Exception as e:
            return FileResult(success=False, action="read", path=str(resolved), error=str(e))
    
    def write(self, path: str, content: str, encoding: str = "utf-8") -> FileResult:
        """Write content to file"""
        resolved = self._resolve_path(path)
        
        allowed, reason = self._is_allowed(resolved)
        if not allowed:
            return FileResult(success=False, action="write", path=str(resolved), error=reason)
        
        try:
            # Create parent directories if needed
            resolved.parent.mkdir(parents=True, exist_ok=True)
            resolved.write_text(content, encoding=encoding)
            return FileResult(success=True, action="write", path=str(resolved))
        
        except Exception as e:
            return FileResult(success=False, action="write", path=str(resolved), error=str(e))
    
    def append(self, path: str, content: str, encoding: str = "utf-8") -> FileResult:
        """Append content to file"""
        resolved = self._resolve_path(path)
        
        allowed, reason = self._is_allowed(resolved)
        if not allowed:
            return FileResult(success=False, action="append", path=str(resolved), error=reason)
        
        try:
            resolved.parent.mkdir(parents=True, exist_ok=True)
            with open(resolved, "a", encoding=encoding) as f:
                f.write(content)
            return FileResult(success=True, action="append", path=str(resolved))
        
        except Exception as e:
            return FileResult(success=False, action="append", path=str(resolved), error=str(e))
    
    def delete(self, path: str) -> FileResult:
        """Delete file or directory"""
        resolved = self._resolve_path(path)
        
        allowed, reason = self._is_allowed(resolved)
        if not allowed:
            return FileResult(success=False, action="delete", path=str(resolved), error=reason)
        
        try:
            if not resolved.exists():
                return FileResult(success=False, action="delete", path=str(resolved), error="Path not found")
            
            if resolved.is_dir():
                shutil.rmtree(resolved)
            else:
                resolved.unlink()
            
            return FileResult(success=True, action="delete", path=str(resolved))
        
        except Exception as e:
            return FileResult(success=False, action="delete", path=str(resolved), error=str(e))
    
    def list_dir(self, path: str = ".", include_hidden: bool = False) -> FileResult:
        """List directory contents"""
        resolved = self._resolve_path(path)
        
        allowed, reason = self._is_allowed(resolved)
        if not allowed:
            return FileResult(success=False, action="list_dir", path=str(resolved), error=reason)
        
        try:
            if not resolved.exists():
                return FileResult(success=False, action="list_dir", path=str(resolved), error="Directory not found")
            
            if not resolved.is_dir():
                return FileResult(success=False, action="list_dir", path=str(resolved), error="Not a directory")
            
            entries = []
            for entry in sorted(resolved.iterdir()):
                if not include_hidden and entry.name.startswith("."):
                    continue
                
                entries.append({
                    "name": entry.name,
                    "type": "directory" if entry.is_dir() else "file",
                    "size": entry.stat().st_size if entry.is_file() else None,
                })
            
            return FileResult(
                success=True,
                action="list_dir",
                path=str(resolved),
                data=str(entries),  # JSON would be better
            )
        
        except Exception as e:
            return FileResult(success=False, action="list_dir", path=str(resolved), error=str(e))
    
    def mkdir(self, path: str) -> FileResult:
        """Create directory"""
        resolved = self._resolve_path(path)
        
        allowed, reason = self._is_allowed(resolved)
        if not allowed:
            return FileResult(success=False, action="mkdir", path=str(resolved), error=reason)
        
        try:
            resolved.mkdir(parents=True, exist_ok=True)
            return FileResult(success=True, action="mkdir", path=str(resolved))
        
        except Exception as e:
            return FileResult(success=False, action="mkdir", path=str(resolved), error=str(e))
    
    def exists(self, path: str) -> FileResult:
        """Check if path exists"""
        resolved = self._resolve_path(path)
        
        allowed, reason = self._is_allowed(resolved)
        if not allowed:
            return FileResult(success=False, action="exists", path=str(resolved), error=reason)
        
        exists = resolved.exists()
        return FileResult(
            success=True,
            action="exists",
            path=str(resolved),
            data=str(exists),
        )
    
    def copy(self, source: str, dest: str) -> FileResult:
        """Copy file or directory"""
        src_resolved = self._resolve_path(source)
        dst_resolved = self._resolve_path(dest)
        
        for resolved in [src_resolved, dst_resolved]:
            allowed, reason = self._is_allowed(resolved)
            if not allowed:
                return FileResult(success=False, action="copy", path=str(resolved), error=reason)
        
        try:
            if not src_resolved.exists():
                return FileResult(success=False, action="copy", path=str(src_resolved), error="Source not found")
            
            if src_resolved.is_dir():
                shutil.copytree(src_resolved, dst_resolved)
            else:
                shutil.copy2(src_resolved, dst_resolved)
            
            return FileResult(success=True, action="copy", path=str(dst_resolved))
        
        except Exception as e:
            return FileResult(success=False, action="copy", path=str(src_resolved), error=str(e))
    
    def move(self, source: str, dest: str) -> FileResult:
        """Move file or directory"""
        src_resolved = self._resolve_path(source)
        dst_resolved = self._resolve_path(dest)
        
        for resolved in [src_resolved, dst_resolved]:
            allowed, reason = self._is_allowed(resolved)
            if not allowed:
                return FileResult(success=False, action="move", path=str(resolved), error=reason)
        
        try:
            if not src_resolved.exists():
                return FileResult(success=False, action="move", path=str(src_resolved), error="Source not found")
            
            shutil.move(src_resolved, dst_resolved)
            return FileResult(success=True, action="move", path=str(dst_resolved))
        
        except Exception as e:
            return FileResult(success=False, action="move", path=str(src_resolved), error=str(e))
