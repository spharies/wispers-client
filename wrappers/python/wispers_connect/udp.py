"""UdpConnection handle."""

from __future__ import annotations

import ctypes
from typing import Any

from ._bridge import DATA_CB, call_async
from ._handle import Handle
from .exceptions import raise_for_status


class UdpConnection(Handle):
    """Wraps a WispersUdpConnectionHandle."""

    def send(self, data: bytes) -> None:
        """Send data over the UDP connection (sync, non-blocking)."""
        from ._library import get_lib
        ptr = self._require_open()
        buf = (ctypes.c_uint8 * len(data)).from_buffer_copy(data)
        status = get_lib().wispers_udp_connection_send(ptr, buf, len(data))
        raise_for_status(status)

    def recv(self) -> bytes:
        """Receive data from the UDP connection. Blocks until data arrives."""
        from ._library import get_lib
        ptr = self._require_open()
        result: bytes = call_async(
            get_lib().wispers_udp_connection_recv_async, ptr, cb=DATA_CB,
        )
        return result

    def _do_close(self, ptr: Any) -> None:
        from ._library import get_lib
        get_lib().wispers_udp_connection_close(ptr)
