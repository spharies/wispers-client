use super::handles::{ManagerImpl, WispersNodeStorageHandle};
use super::helpers::{c_str_to_string, WispersRegistrationInfo};
use crate::errors::WispersStatus;
use crate::state::NodeStorage;
use crate::storage::foreign::WispersNodeStateStoreCallbacks;
use crate::storage::{ForeignNodeStateStore, InMemoryNodeStateStore};
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn wispers_storage_new_in_memory() -> *mut WispersNodeStorageHandle {
    let storage = NodeStorage::new(InMemoryNodeStateStore::new());
    Box::into_raw(Box::new(WispersNodeStorageHandle(ManagerImpl::InMemory(
        storage,
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn wispers_storage_new_with_callbacks(
    callbacks: *const WispersNodeStateStoreCallbacks,
) -> *mut WispersNodeStorageHandle {
    if callbacks.is_null() {
        return std::ptr::null_mut();
    }

    let callbacks = unsafe { *callbacks };
    let store = match ForeignNodeStateStore::new(callbacks) {
        Ok(store) => store,
        Err(_) => return std::ptr::null_mut(),
    };
    let storage = NodeStorage::new(store);
    Box::into_raw(Box::new(WispersNodeStorageHandle(ManagerImpl::Foreign(
        storage,
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn wispers_storage_free(handle: *mut WispersNodeStorageHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wispers_storage_read_registration(
    handle: *mut WispersNodeStorageHandle,
    out_info: *mut WispersRegistrationInfo,
) -> WispersStatus {
    if handle.is_null() || out_info.is_null() {
        return WispersStatus::NullPointer;
    }

    let wrapper = unsafe { &*handle };

    // Handle each variant separately to avoid type mismatch
    let maybe_reg: Result<Option<crate::types::NodeRegistration>, WispersStatus> = match &wrapper.0
    {
        ManagerImpl::InMemory(storage) => storage
            .read_registration()
            .map_err(|_| WispersStatus::StoreError),
        ManagerImpl::Foreign(storage) => storage
            .read_registration()
            .map_err(|_| WispersStatus::StoreError),
    };

    match maybe_reg {
        Ok(Some(reg)) => match WispersRegistrationInfo::from_registration(&reg) {
            Ok(info) => {
                unsafe { *out_info = info };
                WispersStatus::Success
            }
            Err(status) => status,
        },
        Ok(None) => {
            unsafe { *out_info = WispersRegistrationInfo::null() };
            WispersStatus::NotFound
        }
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wispers_storage_override_hub_addr(
    handle: *mut WispersNodeStorageHandle,
    hub_addr: *const c_char,
) -> WispersStatus {
    if handle.is_null() {
        return WispersStatus::NullPointer;
    }

    let addr = match c_str_to_string(hub_addr) {
        Ok(s) => s,
        Err(status) => return status,
    };

    let wrapper = unsafe { &*handle };
    match &wrapper.0 {
        ManagerImpl::InMemory(storage) => storage.override_hub_addr(addr),
        ManagerImpl::Foreign(storage) => storage.override_hub_addr(addr),
    }

    WispersStatus::Success
}

// TODO: wispers_storage_restore_or_init_async with callback-based API
// See discussion: FFI will use callbacks, Swift/Kotlin wrappers convert to native async
