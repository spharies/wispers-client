# C FFI Test Makefile
#
# Targets:
#   make           - Build FFI tests
#   make test      - Build and run FFI tests
#   make clean     - Remove built files

# Detect platform and set library extension
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    LIB_EXT := dylib
else
    LIB_EXT := so
endif

# Use debug build by default, override with: make LIB=../../../target/release/libwispers_connect.$(LIB_EXT)
LIB ?= ../../../target/debug/libwispers_connect.$(LIB_EXT)
INCLUDE ?= ../../include
CC ?= cc
CFLAGS ?= -Wall -Wextra -pedantic

.PHONY: all test clean

all: ffi_test

ffi_test: ffi_test.c $(LIB) $(INCLUDE)/wispers_connect.h
	$(CC) $(CFLAGS) -I$(INCLUDE) ffi_test.c $(LIB) -o ffi_test

test: ffi_test
	./ffi_test

clean:
	rm -f ffi_test
