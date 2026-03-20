"""Tests for exception hierarchy and raise_for_status."""

import pytest

from wispers_connect.exceptions import (
    HubError,
    NotFoundError,
    WispersError,
    raise_for_status,
)
from wispers_connect.types import Status


class TestRaiseForStatus:
    def test_success_is_noop(self) -> None:
        raise_for_status(Status.SUCCESS)

    def test_error_raises_typed_exception(self) -> None:
        with pytest.raises(NotFoundError) as exc_info:
            raise_for_status(Status.NOT_FOUND)
        assert exc_info.value.status == Status.NOT_FOUND
        assert exc_info.value.detail is None

    def test_error_with_detail(self) -> None:
        with pytest.raises(HubError) as exc_info:
            raise_for_status(Status.HUB_ERROR, "server returned 500")
        assert exc_info.value.status == Status.HUB_ERROR
        assert exc_info.value.detail == "server returned 500"
        assert "server returned 500" in str(exc_info.value)

    def test_all_subclass_wispers_error(self) -> None:
        for status in Status:
            if status == Status.SUCCESS:
                continue
            with pytest.raises(WispersError):
                raise_for_status(status)

    def test_isinstance_hierarchy(self) -> None:
        try:
            raise_for_status(Status.NOT_FOUND)
        except NotFoundError as e:
            assert isinstance(e, WispersError)
            assert isinstance(e, NotFoundError)
        except Exception:
            pytest.fail("expected NotFoundError")
