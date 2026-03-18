# C FFI Demo Makefile
#
# Targets:
#   make           - Build demo program
#   make clean     - Remove built files

# Detect platform and set library extension
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    LIB_EXT := dylib
else
    LIB_EXT := so
endif

# Use debug build by default, override with: make LIB=../../target/release/libwispers_connect.$(LIB_EXT)
LIB ?= ../../target/debug/libwispers_connect.$(LIB_EXT)
INCLUDE ?= ../../wispers-connect/include
CC ?= cc
CFLAGS ?= -Wall -Wextra -pedantic

.PHONY: all demo clean

all: ffi_demo

ffi_demo: ffi_demo.c $(LIB) $(INCLUDE)/wispers_connect.h
	$(CC) $(CFLAGS) -I$(INCLUDE) ffi_demo.c $(LIB) -o ffi_demo

demo: ffi_demo
	@echo "Demo built. Run with: ./ffi_demo [--hub <addr>] <command>"
	@echo "Commands: status, register <token>, activate <code>, serve [--activation-code], ping <node>"

clean:
	rm -f ffi_demo
