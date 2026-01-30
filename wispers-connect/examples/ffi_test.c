/**
 * FFI Test Program
 *
 * Tests the wispers-connect C API. Extended with each implementation phase.
 *
 * Phase 1: Storage lifecycle, handle types, callback types compile check
 */

#include "wispers_connect.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TEST(name) printf("Testing: %s... ", name)
#define PASS() printf("PASS\n")
#define FAIL(msg) do { printf("FAIL: %s\n", msg); return 1; } while(0)

//------------------------------------------------------------------------------
// Phase 1: Infrastructure tests
//------------------------------------------------------------------------------

// Test that callback types compile correctly (not invoked yet)
static void dummy_callback(void *ctx, WispersStatus status) {
    (void)ctx;
    (void)status;
}

static void dummy_init_callback(
    void *ctx,
    WispersStatus status,
    WispersStage stage,
    WispersPendingNodeStateHandle *pending,
    WispersRegisteredNodeStateHandle *registered,
    WispersActivatedNodeHandle *activated
) {
    (void)ctx;
    (void)status;
    (void)stage;
    (void)pending;
    (void)registered;
    (void)activated;
}

static void dummy_registered_callback(
    void *ctx,
    WispersStatus status,
    WispersRegisteredNodeStateHandle *handle
) {
    (void)ctx;
    (void)status;
    (void)handle;
}

static void dummy_activated_callback(
    void *ctx,
    WispersStatus status,
    WispersActivatedNodeHandle *handle
) {
    (void)ctx;
    (void)status;
    (void)handle;
}

static int test_callback_types_compile(void) {
    TEST("callback types compile");

    // Just verify these assignments compile - proves the types match
    WispersCallback cb1 = dummy_callback;
    WispersInitCallback cb2 = dummy_init_callback;
    WispersRegisteredCallback cb3 = dummy_registered_callback;
    WispersActivatedCallback cb4 = dummy_activated_callback;

    (void)cb1;
    (void)cb2;
    (void)cb3;
    (void)cb4;

    PASS();
    return 0;
}

static int test_status_codes(void) {
    TEST("status codes");

    // Verify new status codes exist and have expected values
    if (WISPERS_STATUS_SUCCESS != 0) FAIL("SUCCESS != 0");
    if (WISPERS_STATUS_HUB_ERROR != 12) FAIL("HUB_ERROR != 12");
    if (WISPERS_STATUS_CONNECTION_FAILED != 13) FAIL("CONNECTION_FAILED != 13");
    if (WISPERS_STATUS_TIMEOUT != 14) FAIL("TIMEOUT != 14");

    PASS();
    return 0;
}

static int test_stage_enum(void) {
    TEST("stage enum");

    if (WISPERS_STAGE_PENDING != 0) FAIL("PENDING != 0");
    if (WISPERS_STAGE_REGISTERED != 1) FAIL("REGISTERED != 1");
    if (WISPERS_STAGE_ACTIVATED != 2) FAIL("ACTIVATED != 2");

    PASS();
    return 0;
}

static int test_storage_in_memory(void) {
    TEST("storage in-memory create/free");

    WispersNodeStorageHandle *storage = wispers_storage_new_in_memory();
    if (!storage) FAIL("wispers_storage_new_in_memory returned NULL");

    wispers_storage_free(storage);
    PASS();
    return 0;
}

static int test_storage_free_null(void) {
    TEST("storage free NULL is safe");

    // Should not crash
    wispers_storage_free(NULL);

    PASS();
    return 0;
}

static int test_handle_free_null(void) {
    TEST("handle free NULL is safe");

    // All free functions should handle NULL safely
    wispers_pending_state_free(NULL);
    wispers_registered_state_free(NULL);
    wispers_activated_node_free(NULL);
    wispers_string_free(NULL);

    PASS();
    return 0;
}

//------------------------------------------------------------------------------
// Main
//------------------------------------------------------------------------------

int main(void) {
    printf("=== Wispers Connect FFI Tests ===\n\n");

    int failures = 0;

    // Phase 1 tests
    printf("-- Phase 1: Infrastructure --\n");
    failures += test_callback_types_compile();
    failures += test_status_codes();
    failures += test_stage_enum();
    failures += test_storage_in_memory();
    failures += test_storage_free_null();
    failures += test_handle_free_null();

    printf("\n");
    if (failures == 0) {
        printf("All tests passed!\n");
        return 0;
    } else {
        printf("%d test(s) failed.\n", failures);
        return 1;
    }
}
