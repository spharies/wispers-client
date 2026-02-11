package dev.wispers.connect.handles

import com.sun.jna.Pointer
import dev.wispers.connect.internal.CallbackBridge
import dev.wispers.connect.internal.Callbacks
import dev.wispers.connect.internal.NativeLibrary
import dev.wispers.connect.types.WispersException
import dev.wispers.connect.types.WispersStatus
import kotlinx.coroutines.suspendCancellableCoroutine

/**
 * Handle to a QUIC P2P connection.
 *
 * QUIC provides reliable, multiplexed streams over UDP. Each stream is
 * independent - data on one stream doesn't block others.
 *
 * Use for data that requires delivery guarantees (file transfer, RPC)
 * or when you need multiple independent communication channels.
 *
 * Typical usage:
 * ```kotlin
 * val conn = node.connectQuic(peerNodeNumber)
 *
 * // Open a stream and send request
 * val stream = conn.openStream()
 * stream.write("REQUEST".toByteArray())
 * stream.finish()  // Signal end of write
 *
 * // Read response
 * val response = stream.read(1024)
 * stream.close()
 *
 * // Accept streams from peer
 * launch {
 *     while (isActive) {
 *         val inStream = conn.acceptStream()
 *         launch { handleStream(inStream) }
 *     }
 * }
 *
 * conn.close()
 * ```
 */
class QuicConnection internal constructor(
    pointer: Pointer,
    private val lib: NativeLibrary = NativeLibrary.INSTANCE
) : Handle(pointer) {

    /**
     * Open a new bidirectional stream.
     *
     * @return A new QUIC stream handle
     * @throws WispersException.ConnectionFailed if the connection is broken
     */
    suspend fun openStream(): QuicStream = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_quic_connection_open_stream_async(ptr, ctx, Callbacks.quicStream)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }.let { streamPtr ->
        streamPtr as Pointer? ?: throw WispersException.NullPointer("QUIC stream is null")
        QuicStream(streamPtr, lib)
    }

    /**
     * Accept an incoming stream from the peer.
     *
     * Suspends until the peer opens a new stream.
     *
     * @return The incoming QUIC stream handle
     * @throws WispersException.ConnectionFailed if the connection is broken
     */
    suspend fun acceptStream(): QuicStream = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_quic_connection_accept_stream_async(ptr, ctx, Callbacks.quicStream)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }.let { streamPtr ->
        streamPtr as Pointer? ?: throw WispersException.NullPointer("QUIC stream is null")
        QuicStream(streamPtr, lib)
    }

    /**
     * Close the connection.
     *
     * **This consumes the handle** - it cannot be used afterward.
     * All open streams will be terminated.
     *
     * @throws WispersException.ConnectionFailed on error
     */
    suspend fun closeAsync(): Unit = suspendCancellableCoroutine { cont ->
        val ptr = consume() ?: throw IllegalStateException("Handle already consumed")
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_quic_connection_close_async(ptr, ctx, Callbacks.basic)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    /**
     * Close the connection synchronously.
     *
     * Prefer [closeAsync] when in a coroutine context for proper cleanup.
     */
    override fun close() {
        val ptr = consume() ?: return
        // Can't do async close from close(), just free the handle
        lib.wispers_quic_connection_free(ptr)
    }

    override fun doClose(pointer: Pointer) {
        lib.wispers_quic_connection_free(pointer)
    }
}
