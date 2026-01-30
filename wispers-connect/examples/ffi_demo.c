/**
 * FFI Demo Program
 *
 * Demonstrates using the wispers-connect C API to:
 * - Initialize/restore node state
 * - Register with hub (if needed)
 * - Activate with a pairing code (if needed)
 * - Ping another node
 *
 * This is a minimal C equivalent of what `wconnect` does.
 *
 * Usage:
 *   ./ffi_demo status              - Show current node state
 *   ./ffi_demo register <token>    - Register with the given token
 *   ./ffi_demo activate <code>     - Activate with pairing code
 *   ./ffi_demo ping <node_number>  - Ping another node
 */

#include "wispers_connect.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s status              - Show current node state\n", program);
    fprintf(stderr, "  %s register <token>    - Register with the given token\n", program);
    fprintf(stderr, "  %s activate <code>     - Activate with pairing code\n", program);
    fprintf(stderr, "  %s ping <node_number>  - Ping another node\n", program);
}

//------------------------------------------------------------------------------
// Commands (to be implemented as FFI phases are completed)
//------------------------------------------------------------------------------

static int cmd_status(void) {
    // TODO: Phase 3 - wispers_storage_restore_or_init_async
    fprintf(stderr, "status: not yet implemented (requires Phase 3)\n");
    return 1;
}

static int cmd_register(const char *token) {
    (void)token;
    // TODO: Phase 3 - wispers_pending_state_register_async
    fprintf(stderr, "register: not yet implemented (requires Phase 3)\n");
    return 1;
}

static int cmd_activate(const char *pairing_code) {
    (void)pairing_code;
    // TODO: Phase 5 - wispers_registered_state_activate_async
    fprintf(stderr, "activate: not yet implemented (requires Phase 5)\n");
    return 1;
}

static int cmd_ping(int node_number) {
    (void)node_number;
    // TODO: Phase 8 - wispers_activated_node_connect_quic_async
    fprintf(stderr, "ping: not yet implemented (requires Phase 8)\n");
    return 1;
}

//------------------------------------------------------------------------------
// Main
//------------------------------------------------------------------------------

int main(int argc, char *argv[]) {
    if (argc < 2) {
        print_usage(argv[0]);
        return 1;
    }

    const char *command = argv[1];

    if (strcmp(command, "status") == 0) {
        return cmd_status();
    } else if (strcmp(command, "register") == 0) {
        if (argc < 3) {
            fprintf(stderr, "register: missing token argument\n");
            return 1;
        }
        return cmd_register(argv[2]);
    } else if (strcmp(command, "activate") == 0) {
        if (argc < 3) {
            fprintf(stderr, "activate: missing pairing code argument\n");
            return 1;
        }
        return cmd_activate(argv[2]);
    } else if (strcmp(command, "ping") == 0) {
        if (argc < 3) {
            fprintf(stderr, "ping: missing node number argument\n");
            return 1;
        }
        int node_number = atoi(argv[2]);
        return cmd_ping(node_number);
    } else {
        fprintf(stderr, "Unknown command: %s\n", command);
        print_usage(argv[0]);
        return 1;
    }
}
