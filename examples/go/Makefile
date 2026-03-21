# Go example Makefile
#
# Targets:
#   make           - Build the wconnect-go binary
#   make clean     - Remove built files

# Detect platform and set library extension
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    LIB_EXT := dylib
else
    LIB_EXT := so
endif

# Resolve to absolute paths so CGo can find headers from any working directory.
ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
LIB_DIR ?= $(abspath $(ROOT)/../../target/debug)
INCLUDE ?= $(abspath $(ROOT)/../../wispers-connect/include)

export CGO_CFLAGS=-I$(INCLUDE)
export CGO_LDFLAGS=-L$(LIB_DIR) -lwispers_connect

.PHONY: all clean

all: wconnect-go

wconnect-go: main.go go.mod $(LIB_DIR)/libwispers_connect.$(LIB_EXT)
	go build -o wconnect-go .

clean:
	rm -f wconnect-go
