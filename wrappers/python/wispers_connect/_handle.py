"""Base handle class for opaque C pointers."""

from __future__ import annotations

import threading
from typing import Any


class Handle:
    """Base class for opaque C handle wrappers.

    Mirrors Go handle.go / Kotlin Handle.kt: tracks whether the handle has
    been closed/consumed, preventing use-after-free.
    """

    __slots__ = ("_ptr", "_closed", "_lock")

    def __init__(self, ptr: Any) -> None:
        self._ptr = ptr
        self._closed = False
        self._lock = threading.Lock()

    def _require_open(self) -> Any:
        """Return the raw pointer, raising if closed."""
        with self._lock:
            if self._closed:
                raise RuntimeError("wispers: use of closed handle")
            return self._ptr

    def _consume(self) -> Any:
        """Return the raw pointer and mark closed (for ownership-consuming calls)."""
        with self._lock:
            if self._closed:
                raise RuntimeError("wispers: use of closed handle")
            self._closed = True
            return self._ptr

    def _do_close(self, ptr: Any) -> None:
        """Subclass hook: free the C handle. Called at most once."""
        raise NotImplementedError

    def close(self) -> None:
        """Close the handle, freeing the C resource. Idempotent."""
        with self._lock:
            if self._closed:
                return
            self._closed = True
            ptr = self._ptr
        self._do_close(ptr)

    def __enter__(self) -> Handle:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            if not self._closed:
                self.close()
        except Exception:
            pass
