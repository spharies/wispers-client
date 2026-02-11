package dev.wispers.connect

import android.content.Context
import dev.wispers.connect.handles.StorageHandle
import dev.wispers.connect.internal.NativeLibrary
import dev.wispers.connect.storage.EncryptedStorage
import dev.wispers.connect.storage.NodeStorageCallbacks
import dev.wispers.connect.storage.toNativeCallbacks
import dev.wispers.connect.types.WispersException

/**
 * Entry point for wispers-connect.
 *
 * Use this object to create storage handles:
 *
 * ```kotlin
 * // Recommended: secure encrypted storage
 * val storage = WispersConnect.createStorage(context)
 *
 * // Initialize or restore node
 * val (node, state) = storage.restoreOrInit()
 *
 * // For testing only: in-memory storage (data lost on process exit)
 * val testStorage = WispersConnect.createInMemoryStorage()
 * ```
 */
object WispersConnect {

    private val lib: NativeLibrary by lazy { NativeLibrary.INSTANCE }

    /**
     * Create storage with secure encrypted persistence.
     *
     * This is the recommended method for production use. Data is encrypted using
     * AES256-GCM with keys stored in Android Keystore (hardware-backed if available).
     *
     * @param context Application or activity context
     * @return A new storage handle
     * @throws WispersException if storage creation fails
     */
    fun createStorage(context: Context): StorageHandle {
        val callbacks = EncryptedStorage.create(context)
        return createStorage(callbacks)
    }

    /**
     * Create storage with custom persistence callbacks.
     *
     * Use this if you need custom storage behavior (e.g., different encryption,
     * remote backup, multi-profile support). For most apps, prefer [createStorage]
     * with a Context parameter.
     *
     * @param callbacks Implementation of storage callbacks
     * @return A new storage handle
     * @throws WispersException if storage creation fails
     */
    fun createStorage(callbacks: NodeStorageCallbacks): StorageHandle {
        val nativeCallbacks = callbacks.toNativeCallbacks()
        val ptr = lib.wispers_storage_new_with_callbacks(nativeCallbacks)
            ?: throw WispersException.NullPointer("Failed to create storage with callbacks")
        return StorageHandle(ptr, lib)
    }

    /**
     * Create storage using in-memory backing store.
     *
     * **For testing only.** Data will be lost when the process exits.
     * For production use, use [createStorage] with a Context.
     *
     * @return A new storage handle
     * @throws WispersException if storage creation fails
     */
    fun createInMemoryStorage(): StorageHandle {
        val ptr = lib.wispers_storage_new_in_memory()
            ?: throw WispersException.NullPointer("Failed to create in-memory storage")
        return StorageHandle(ptr, lib)
    }
}
