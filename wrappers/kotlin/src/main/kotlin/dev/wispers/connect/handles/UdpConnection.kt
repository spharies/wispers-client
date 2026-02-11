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
 * Handle to a UDP P2P connection.
 *
 * UDP provides fast, low-latency communication without reliability guarantees.
 * Messages may be lost, duplicated, or arrive out of order.
 *
 * Use for real-time data where occasional loss is acceptable (e.g., game state,
 * sensor readings, voice/video).
 *
 * Typical usage:
 * ```kotlin
 * val conn = node.connectUdp(peerNodeNumber)
 *
 * // Send data (synchronous, non-blocking)
 * conn.send("PING".toByteArray())
 *
 * // Receive data (suspends until data arrives)
 * val response = conn.recv()
 *
 * conn.close()
 * ```
 */
class UdpConnection internal constructor(
    pointer: Pointer,
    private val lib: NativeLibrary = NativeLibrary.INSTANCE
) : Handle(pointer) {

    /**
     * Send data over the connection.
     *
     * This is a synchronous, non-blocking operation. The data is queued
     * for sending but delivery is not guaranteed (UDP semantics).
     *
     * @param data The data to send
     * @throws WispersException.ConnectionFailed if the connection is broken
     */
    fun send(data: ByteArray) {
        val ptr = requireOpen()

        val mem = Memory(data.size.toLong())
        mem.write(0, data, 0, data.size)

        val status = lib.wispers_udp_connection_send(ptr, mem, data.size.toLong())
        if (status != WispersStatus.SUCCESS.code) {
            throw WispersException.fromStatus(status)
        }
    }

    /**
     * Receive data from the connection.
     *
     * Suspends until data is available.
     *
     * @return The received data
     * @throws WispersException.ConnectionFailed if the connection is broken
     */
    suspend fun recv(): ByteArray = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_udp_connection_recv_async(ptr, ctx, Callbacks.data)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    /**
     * Close the connection.
     *
     * **This consumes the handle** - it cannot be used afterward.
     */
    override fun close() {
        val ptr = consume() ?: return
        lib.wispers_udp_connection_close(ptr)
    }

    override fun doClose(pointer: Pointer) {
        lib.wispers_udp_connection_close(pointer)
    }
}
