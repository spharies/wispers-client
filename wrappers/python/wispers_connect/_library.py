"""Native library loader for wispers-connect."""

from __future__ import annotations

import ctypes
import ctypes.util
import os
import sys
from pathlib import Path

_lib: ctypes.CDLL | None = None


def _lib_filename() -> str:
    if sys.platform == "darwin":
        return "libwispers_connect.dylib"
    elif sys.platform == "win32":
        return "wispers_connect.dll"
    return "libwispers_connect.so"


def _load_lib() -> ctypes.CDLL:
    # 1. Explicit env var
    env_path = os.environ.get("WISPERS_CONNECT_LIB")
    if env_path:
        return ctypes.CDLL(env_path)

    # 2. Dev build path (relative to this file)
    dev_path = Path(__file__).resolve().parent.parent.parent.parent / "target" / "debug" / _lib_filename()
    if dev_path.exists():
        return ctypes.CDLL(str(dev_path))

    # 3. System search
    found = ctypes.util.find_library("wispers_connect")
    if found:
        return ctypes.CDLL(found)

    raise OSError(
        f"Could not find {_lib_filename()}. "
        "Set WISPERS_CONNECT_LIB or build with `cargo build` in connect/client/."
    )


def get_lib() -> ctypes.CDLL:
    """Return the shared library, loading and declaring FFI signatures on first call."""
    global _lib
    if _lib is None:
        _lib = _load_lib()
        from . import _ffi
        _ffi.declare_functions(_lib)
    return _lib
