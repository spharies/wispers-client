"""ServingSession + IncomingConnections handles."""

from __future__ import annotations

from typing import Any

from ._bridge import (
    ACTIVATION_CODE_CB,
    BASIC_CB,
    QUIC_CONNECTION_CB,
    UDP_CONNECTION_CB,
    call_async,
)
from ._handle import Handle


class IncomingConnections(Handle):
    """Wraps WispersIncomingConnections for accepting P2P connections."""

    def accept_udp(self) -> Any:
        """Accept an incoming UDP connection from a peer."""
        from ._library import get_lib
        from .udp import UdpConnection
        ptr = self._require_open()
        conn_ptr = call_async(
            get_lib().wispers_incoming_accept_udp_async, ptr, cb=UDP_CONNECTION_CB,
        )
        return UdpConnection(conn_ptr)

    def accept_quic(self) -> Any:
        """Accept an incoming QUIC connection from a peer."""
        from ._library import get_lib
        from .quic import QuicConnection
        ptr = self._require_open()
        conn_ptr = call_async(
            get_lib().wispers_incoming_accept_quic_async, ptr, cb=QUIC_CONNECTION_CB,
        )
        return QuicConnection(conn_ptr)

    def _do_close(self, ptr: Any) -> None:
        from ._library import get_lib
        get_lib().wispers_incoming_connections_free(ptr)


class ServingSession:
    """Composite handle: serving handle + session + optional incoming connections.

    Not a Handle subclass — owns multiple C handles with different lifetimes.
    """

    def __init__(
        self,
        serving_ptr: Any,
        session_ptr: Any,
        incoming_ptr: Any | None,
    ) -> None:
        self._serving_ptr = serving_ptr
        self._session_ptr = session_ptr
        self._session_consumed = False
        self.incoming: IncomingConnections | None = (
            IncomingConnections(incoming_ptr) if incoming_ptr else None
        )

    def generate_activation_code(self) -> str:
        """Generate an activation code for endorsing a new node."""
        from ._library import get_lib
        if self._serving_ptr is None:
            raise RuntimeError("wispers: serving handle already closed")
        result: str = call_async(
            get_lib().wispers_serving_handle_generate_activation_code_async,
            self._serving_ptr, cb=ACTIVATION_CODE_CB,
        )
        return result

    def run(self) -> None:
        """Run the serving session event loop. Blocks until shutdown or error.

        The session handle is consumed by this call.
        """
        from ._library import get_lib
        if self._session_consumed:
            raise RuntimeError("wispers: session already consumed")
        self._session_consumed = True
        call_async(
            get_lib().wispers_serving_session_run_async,
            self._session_ptr, cb=BASIC_CB,
        )

    def shutdown(self) -> None:
        """Request the serving session to shut down."""
        from ._library import get_lib
        if self._serving_ptr is None:
            raise RuntimeError("wispers: serving handle already closed")
        call_async(
            get_lib().wispers_serving_handle_shutdown_async,
            self._serving_ptr, cb=BASIC_CB,
        )

    def close(self) -> None:
        """Free all handles. Idempotent."""
        from ._library import get_lib
        lib = get_lib()
        if self.incoming is not None:
            self.incoming.close()
            self.incoming = None
        if not self._session_consumed and self._session_ptr is not None:
            lib.wispers_serving_session_free(self._session_ptr)
            self._session_consumed = True
        if self._serving_ptr is not None:
            lib.wispers_serving_handle_free(self._serving_ptr)
            self._serving_ptr = None

    def __enter__(self) -> ServingSession:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        self.close()
