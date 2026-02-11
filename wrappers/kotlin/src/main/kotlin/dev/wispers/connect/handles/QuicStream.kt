package dev.wispers.connect.handles

import com.sun.jna.Memory
import com.sun.jna.Pointer
import dev.wispers.connect.internal.CallbackBridge
import dev.wispers.connect.internal.Callbacks
import dev.wispers.connect.internal.NativeLibrary
import dev.wispers.connect.types.WispersException
import dev.wispers.connect.types.WispersStatus
import kotlinx.coroutines.suspendCancellableCoroutine

/**
 * Handle to a QUIC stream.
 *
 * Streams are bidirectional and independent - data flow on one stream
 * doesn't affect others on the same connection.
 *
 * Stream lifecycle:
 * 1. Write data with [write]
 * 2. Call [finish] to signal end of writes (sends FIN)
 * 3. Read data with [read] until it returns empty (peer sent FIN)
 * 4. Call [close] to release resources
 *
 * Typical usage:
 * ```kotlin
 * // Request-response pattern
 * val stream = conn.openStream()
 * stream.write("REQUEST".toByteArray())
 * stream.finish()
 *
 * val response = buildString {
 *     while (true) {
 *         val chunk = stream.read(1024)
 *         if (chunk.isEmpty()) break
 *         append(chunk.decodeToString())
 *     }
 * }
 * stream.close()
 * ```
 */
class QuicStream internal constructor(
    pointer: Pointer,
    private val lib: NativeLibrary = NativeLibrary.INSTANCE
) : Handle(pointer) {

    /**
     * Write data to the stream.
     *
     * Data is buffered and sent reliably. Call [finish] after the last write
     * to signal end of data.
     *
     * @param data The data to write
     * @throws WispersException.ConnectionFailed if the stream/connection is broken
     */
    suspend fun write(data: ByteArray): Unit = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val mem = Memory(data.size.toLong())
        mem.write(0, data, 0, data.size)

        val status = lib.wispers_quic_stream_write_async(ptr, mem, data.size.toLong(), ctx, Callbacks.basic)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    /**
     * Read data from the stream.
     *
     * Returns up to [maxLen] bytes. Returns an empty array when the peer
     * has finished writing (sent FIN).
     *
     * @param maxLen Maximum number of bytes to read
     * @return The received data, or empty array if stream ended
     * @throws WispersException.ConnectionFailed if the stream/connection is broken
     */
    suspend fun read(maxLen: Int): ByteArray = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_quic_stream_read_async(ptr, maxLen.toLong(), ctx, Callbacks.data)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    /**
     * Signal end of writes (send FIN).
     *
     * Call this after the last [write] to let the peer know no more data
     * is coming. You can still [read] from the stream after finishing.
     *
     * @throws WispersException.ConnectionFailed if the stream/connection is broken
     */
    suspend fun finish(): Unit = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_quic_stream_finish_async(ptr, ctx, Callbacks.basic)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    /**
     * Shutdown the stream (stop sending and receiving).
     *
     * Use this to abruptly terminate the stream. For graceful shutdown,
     * use [finish] and wait for reads to complete.
     *
     * @throws WispersException.ConnectionFailed if the stream/connection is broken
     */
    suspend fun shutdown(): Unit = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_quic_stream_shutdown_async(ptr, ctx, Callbacks.basic)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    override fun doClose(pointer: Pointer) {
        lib.wispers_quic_stream_free(pointer)
    }
}
