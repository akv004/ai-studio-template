"""
AI Studio - Python Sidecar Server (Mock)
========================================

This is a mock HTTP server that simulates AI model inference.
In production, this would interface with actual ML frameworks
like PyTorch, TensorFlow, or ONNX Runtime.

Currently returns mock JSON responses for all endpoints.
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import time
import random
from pathlib import Path

# Mock response directory
MOCK_DIR = Path(__file__).parent / "mock_responses"


class MockAIHandler(BaseHTTPRequestHandler):
    """HTTP request handler with mock AI responses"""

    def _send_json(self, data: dict, status: int = 200):
        """Send JSON response"""
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def _load_mock(self, name: str) -> dict:
        """Load mock response from file"""
        mock_file = MOCK_DIR / f"{name}.json"
        if mock_file.exists():
            return json.loads(mock_file.read_text())
        return {"error": f"Mock not found: {name}"}

    def do_GET(self):
        """Handle GET requests"""
        # Simulate network delay
        time.sleep(random.uniform(0.1, 0.3))

        if self.path == "/health":
            self._send_json({"status": "healthy", "version": "0.1.0"})

        elif self.path == "/models":
            self._send_json(self._load_mock("models"))

        elif self.path == "/status":
            self._send_json({
                "gpu_available": True,
                "gpu_memory_used": random.randint(2000, 6000),
                "gpu_memory_total": 8192,
                "active_models": ["yolov8", "whisper-base"],
                "queue_length": random.randint(0, 5)
            })

        else:
            self._send_json({"error": "Not found"}, 404)

    def do_POST(self):
        """Handle POST requests"""
        # Simulate inference delay
        time.sleep(random.uniform(0.2, 0.5))

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)
        
        try:
            request_data = json.loads(body) if body else {}
        except json.JSONDecodeError:
            request_data = {}

        if self.path == "/inference/vision":
            self._send_json(self._load_mock("vision_inference"))

        elif self.path == "/inference/audio":
            self._send_json(self._load_mock("audio_inference"))

        elif self.path == "/inference/text":
            self._send_json(self._load_mock("text_inference"))

        elif self.path == "/train/start":
            self._send_json({
                "run_id": f"run_{int(time.time())}",
                "status": "started",
                "message": "Training job queued successfully"
            })

        elif self.path == "/train/status":
            run_id = request_data.get("run_id", "unknown")
            self._send_json({
                "run_id": run_id,
                "status": "running",
                "epoch": random.randint(1, 100),
                "loss": round(random.uniform(0.01, 0.5), 4),
                "accuracy": round(random.uniform(0.8, 0.99), 4)
            })

        else:
            self._send_json({"error": "Not found"}, 404)

    def do_OPTIONS(self):
        """Handle CORS preflight"""
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def log_message(self, format, *args):
        """Custom logging format"""
        print(f"[AI Sidecar] {self.address_string()} - {args[0]}")


def main():
    """Run the mock AI server"""
    host = "127.0.0.1"
    port = 8765

    # Ensure mock responses directory exists
    MOCK_DIR.mkdir(exist_ok=True)

    server = HTTPServer((host, port), MockAIHandler)
    print(f"🤖 AI Studio Sidecar (Mock) running at http://{host}:{port}")
    print("   Endpoints:")
    print("   - GET  /health        - Health check")
    print("   - GET  /models        - List available models")
    print("   - GET  /status        - GPU/queue status")
    print("   - POST /inference/*   - Run inference")
    print("   - POST /train/*       - Training operations")
    print("\n   Press Ctrl+C to stop\n")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n👋 Shutting down...")
        server.shutdown()


if __name__ == "__main__":
    main()
