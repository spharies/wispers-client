package dev.wispers.connect.handles

import com.sun.jna.Pointer
import dev.wispers.connect.internal.CallbackBridge
import dev.wispers.connect.internal.Callbacks
import dev.wispers.connect.internal.NativeLibrary
import dev.wispers.connect.internal.NativeTypes
import dev.wispers.connect.types.ActivationStatus
import dev.wispers.connect.types.NodeInfo
import dev.wispers.connect.types.NodeState
import dev.wispers.connect.types.WispersException
import dev.wispers.connect.types.WispersStatus
import kotlinx.coroutines.suspendCancellableCoroutine

/**
 * A wispers node.
 *
 * The node progresses through states: [NodeState.Pending] -> [NodeState.Registered] -> [NodeState.Activated]
 *
 * Available operations depend on the current state:
 * - **Pending**: [register] with a registration token
 * - **Registered**: [activate] with a pairing code, [listNodes], [startServing] (for bootstrap)
 * - **Activated**: [listNodes], [startServing], [connectUdp], [connectQuic]
 *
 * The node remains valid across state transitions. Call [close] when done,
 * or use [logout] to deregister and invalidate the node.
 */
class Node internal constructor(
    pointer: Pointer,
    private val lib: NativeLibrary = NativeLibrary.INSTANCE
) : Handle(pointer) {

    /**
     * Current state of the node.
     */
    val state: NodeState
        get() = NodeState.fromCode(lib.wispers_node_state(requireOpen()))

    /**
     * Register the node with the hub using a registration token.
     *
     * Requires [NodeState.Pending]. On success, transitions to [NodeState.Registered].
     *
     * @param token Registration token from the hub
     * @throws WispersException.InvalidState if not in Pending state
     * @throws WispersException.AlreadyRegistered if already registered
     * @throws WispersException.HubError on hub communication failure
     */
    suspend fun register(token: String): Unit = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_node_register_async(ptr, token, ctx, Callbacks.basic)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    /**
     * Activate the node by pairing with an endorser.
     *
     * The pairing code format is "node_number-secret" (e.g., "1-abc123xyz0").
     * Get this from an already-activated node via [ServingSession.generatePairingCode].
     *
     * Requires [NodeState.Registered]. On success, transitions to [NodeState.Activated].
     *
     * @param pairingCode Pairing code from an endorser node
     * @throws WispersException.InvalidState if not in Registered state
     * @throws WispersException.InvalidPairingCode if the code format is invalid
     * @throws WispersException.ActivationFailed if pairing verification fails
     */
    suspend fun activate(pairingCode: String): Unit = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_node_activate_async(ptr, pairingCode, ctx, Callbacks.basic)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    /**
     * Logout the node.
     *
     * This deletes local state and deregisters from the hub. If activated,
     * the node is also revoked from the roster.
     *
     * **This consumes the node** - it cannot be used after logout completes.
     *
     * @throws WispersException.HubError on hub communication failure
     */
    suspend fun logout(): Unit = suspendCancellableCoroutine { cont ->
        val ptr = consume() ?: throw IllegalStateException("Node already consumed")
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_node_logout_async(ptr, ctx, Callbacks.basic)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }

    /**
     * List all nodes in the connectivity group.
     *
     * Requires [NodeState.Registered] or [NodeState.Activated].
     *
     * @return List of nodes in the connectivity group
     * @throws WispersException.InvalidState if in Pending state
     * @throws WispersException.HubError on hub communication failure
     */
    suspend fun listNodes(): List<NodeInfo> {
        val listPtr: Pointer? = suspendCancellableCoroutine { cont ->
            val ptr = requireOpen()
            val ctx = CallbackBridge.register(cont)

            val status = lib.wispers_node_list_nodes_async(ptr, ctx, Callbacks.nodeList)
            if (status != WispersStatus.SUCCESS.code) {
                CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
            }
        }

        if (listPtr == null) {
            return emptyList()
        }

        try {
            val nodeList = NativeTypes.WispersNodeList()
            nodeList.useMemory(listPtr)
            nodeList.read()

            val count = nodeList.count.toInt()
            if (count == 0 || nodeList.nodes == null) {
                return emptyList()
            }

            val nodeSize = NativeTypes.WispersNode().size()
            return (0 until count).map { i ->
                val nodePtr = nodeList.nodes!!.share((i * nodeSize).toLong())
                val node = NativeTypes.WispersNode()
                node.useMemory(nodePtr)
                node.read()

                NodeInfo(
                    nodeNumber = node.nodeNumber,
                    name = node.name?.getString(0, "UTF-8") ?: "",
                    isSelf = node.isSelf,
                    activationStatus = ActivationStatus.fromCode(node.activationStatus),
                    lastSeenAtMillis = if (node.lastSeenAtMillis > 0) node.lastSeenAtMillis else null
                )
            }
        } finally {
            lib.wispers_node_list_free(listPtr)
        }
    }

    /**
     * Start a serving session.
     *
     * Serving connects to the hub and makes the node reachable by peers.
     *
     * - **Registered nodes** can serve for bootstrapping (generate pairing codes)
     *   but cannot accept P2P connections.
     * - **Activated nodes** can also accept incoming P2P connections.
     *
     * Requires [NodeState.Registered] or [NodeState.Activated].
     *
     * After calling this, you must run the event loop:
     * ```kotlin
     * val session = node.startServing()
     * scope.launch { session.runEventLoop() }
     * ```
     *
     * @return A serving session
     * @throws WispersException.InvalidState if in Pending state
     * @throws WispersException.HubError on hub communication failure
     */
    suspend fun startServing(): ServingSession = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_node_start_serving_async(ptr, ctx, Callbacks.startServing)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }.let { result ->
        @Suppress("UNCHECKED_CAST")
        val (servingPtr, sessionPtr, incomingPtr) = result as Triple<Pointer?, Pointer?, Pointer?>

        if (servingPtr == null || sessionPtr == null) {
            throw WispersException.NullPointer("Serving session is null")
        }

        ServingSession(
            servingHandle = servingPtr,
            sessionHandle = sessionPtr,
            incomingHandle = incomingPtr,
            lib = lib
        )
    }

    /**
     * Connect to a peer node using UDP transport.
     *
     * UDP provides fast, low-latency communication but without reliability guarantees.
     * Use for real-time data where occasional packet loss is acceptable.
     *
     * Requires [NodeState.Activated].
     *
     * @param peerNodeNumber The node number to connect to
     * @return A UDP connection
     * @throws WispersException.InvalidState if not in Activated state
     * @throws WispersException.ConnectionFailed if connection fails
     */
    suspend fun connectUdp(peerNodeNumber: Int): UdpConnection = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_node_connect_udp_async(ptr, peerNodeNumber, ctx, Callbacks.udpConnection)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }.let { connPtr ->
        connPtr as Pointer? ?: throw WispersException.NullPointer("UDP connection is null")
        UdpConnection(connPtr, lib)
    }

    /**
     * Connect to a peer node using QUIC transport.
     *
     * QUIC provides reliable, multiplexed streams over UDP. Use for data that
     * requires delivery guarantees or when you need multiple independent streams.
     *
     * Requires [NodeState.Activated].
     *
     * @param peerNodeNumber The node number to connect to
     * @return A QUIC connection
     * @throws WispersException.InvalidState if not in Activated state
     * @throws WispersException.ConnectionFailed if connection fails
     */
    suspend fun connectQuic(peerNodeNumber: Int): QuicConnection = suspendCancellableCoroutine { cont ->
        val ptr = requireOpen()
        val ctx = CallbackBridge.register(cont)

        val status = lib.wispers_node_connect_quic_async(ptr, peerNodeNumber, ctx, Callbacks.quicConnection)
        if (status != WispersStatus.SUCCESS.code) {
            CallbackBridge.resumeException(ctx, WispersException.fromStatus(status))
        }
    }.let { connPtr ->
        connPtr as Pointer? ?: throw WispersException.NullPointer("QUIC connection is null")
        QuicConnection(connPtr, lib)
    }

    override fun doClose(pointer: Pointer) {
        lib.wispers_node_free(pointer)
    }
}
