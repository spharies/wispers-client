package dev.wispers.connect.handles

import com.sun.jna.Pointer
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Base class for all native handle wrappers.
 *
 * Provides resource management with:
 * - [requireOpen] to ensure handle is still valid before use
 * - [consume] to take ownership (for operations that invalidate the handle)
 * - [close] to release the native resource
 *
 * Implements [AutoCloseable] for use with `use {}` blocks.
 */
abstract class Handle internal constructor(
    @Volatile protected var pointer: Pointer?
) : AutoCloseable {

    private val closed = AtomicBoolean(false)

    /**
     * Whether this handle has been closed or consumed.
     */
    val isClosed: Boolean
        get() = closed.get()

    /**
     * Get the pointer, throwing if the handle has been closed.
     *
     * @throws IllegalStateException if the handle has been closed or consumed
     */
    protected fun requireOpen(): Pointer {
        if (closed.get()) {
            throw IllegalStateException("Handle has been closed or consumed")
        }
        return pointer ?: throw IllegalStateException("Handle pointer is null")
    }

    /**
     * Consume the handle, returning the pointer and marking as closed.
     *
     * Use this for operations that take ownership of the native handle
     * (e.g., logout, which frees the handle internally).
     *
     * @return The pointer, or null if already consumed
     */
    protected fun consume(): Pointer? {
        if (closed.getAndSet(true)) {
            return null
        }
        val ptr = pointer
        pointer = null
        return ptr
    }

    /**
     * Close the handle and release native resources.
     *
     * Safe to call multiple times - subsequent calls are no-ops.
     */
    override fun close() {
        val ptr = consume() ?: return
        doClose(ptr)
    }

    /**
     * Subclass-specific cleanup. Called exactly once with a valid pointer.
     */
    protected abstract fun doClose(pointer: Pointer)
}
