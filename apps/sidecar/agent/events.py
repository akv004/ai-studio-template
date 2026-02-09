"""
Event Bus
=========
Async broadcast system for real-time event streaming.

Events emitted during chat/tool execution are broadcast to all
connected WebSocket clients (typically just Tauri).
"""

import asyncio
import json
import uuid
from datetime import datetime, timezone
from typing import Optional


class EventBus:
    """Broadcasts events to all subscribed WebSocket queues."""

    def __init__(self):
        self._subscribers: set[asyncio.Queue] = set()
        self._seq_counters: dict[str, int] = {}

    def subscribe(self) -> asyncio.Queue:
        queue: asyncio.Queue = asyncio.Queue(maxsize=1000)
        self._subscribers.add(queue)
        return queue

    def unsubscribe(self, queue: asyncio.Queue):
        self._subscribers.discard(queue)

    def _next_seq(self, session_id: str) -> int:
        self._seq_counters[session_id] = self._seq_counters.get(session_id, 0) + 1
        return self._seq_counters[session_id]

    async def emit(
        self,
        event_type: str,
        session_id: str,
        source: str,
        payload: dict,
        cost_usd: Optional[float] = None,
    ) -> dict:
        """Emit an event to all subscribers. Returns the event dict."""
        event = {
            "event_id": str(uuid.uuid4()),
            "type": event_type,
            "ts": datetime.now(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z"),
            "session_id": session_id,
            "source": source,
            "seq": self._next_seq(session_id),
            "payload": payload,
            "cost_usd": cost_usd,
        }

        msg = json.dumps(event)
        dead: list[asyncio.Queue] = []
        for queue in self._subscribers:
            try:
                queue.put_nowait(msg)
            except asyncio.QueueFull:
                dead.append(queue)

        for q in dead:
            self._subscribers.discard(q)

        return event
