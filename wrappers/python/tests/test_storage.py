"""Tests for NodeStorage using the in-memory backend and the native library."""

import pytest

from wispers_connect import NodeState, NodeStorage, NotFoundError


class TestInMemoryStorage:
    def test_restore_or_init_returns_pending(self) -> None:
        storage = NodeStorage.in_memory()
        try:
            node, state = storage.restore_or_init()
            assert state == NodeState.PENDING
            node.close()
        finally:
            storage.close()

    def test_read_registration_not_found(self) -> None:
        storage = NodeStorage.in_memory()
        try:
            with pytest.raises(NotFoundError):
                storage.read_registration()
        finally:
            storage.close()

    def test_context_manager(self) -> None:
        with NodeStorage.in_memory() as storage:
            node, state = storage.restore_or_init()
            assert state == NodeState.PENDING
            node.close()

    def test_close_is_idempotent(self) -> None:
        storage = NodeStorage.in_memory()
        storage.close()
        storage.close()  # should not raise

    def test_use_after_close_raises(self) -> None:
        storage = NodeStorage.in_memory()
        storage.close()
        with pytest.raises(RuntimeError, match="closed"):
            storage.restore_or_init()
