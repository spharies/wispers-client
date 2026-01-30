use crate::errors::WispersStatus;
use crate::types::NodeRegistration;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

pub fn c_str_to_string(ptr: *const c_char) -> Result<String, WispersStatus> {
    if ptr.is_null() {
        return Err(WispersStatus::NullPointer);
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_owned())
            .map_err(|_| WispersStatus::InvalidUtf8)
    }
}

pub fn optional_c_str(ptr: *const c_char) -> Result<Option<String>, WispersStatus> {
    if ptr.is_null() {
        Ok(None)
    } else {
        c_str_to_string(ptr).map(Some)
    }
}

pub unsafe fn reset_out_ptr<T>(out: *mut *mut T) {
    if !out.is_null() {
        unsafe {
            *out = ptr::null_mut();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wispers_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

/// Registration info returned to C callers.
#[repr(C)]
pub struct WispersRegistrationInfo {
    pub connectivity_group_id: *mut c_char,
    pub node_number: c_int,
    pub auth_token: *mut c_char,
}

impl WispersRegistrationInfo {
    /// Create from a NodeRegistration, allocating C strings.
    pub fn from_registration(reg: &NodeRegistration) -> Result<Self, WispersStatus> {
        let cg_id = CString::new(reg.connectivity_group_id.to_string())
            .map_err(|_| WispersStatus::InvalidUtf8)?;
        let token_str = reg
            .auth_token()
            .map(|t| t.as_str())
            .unwrap_or("");
        let token = CString::new(token_str).map_err(|_| WispersStatus::InvalidUtf8)?;

        Ok(Self {
            connectivity_group_id: cg_id.into_raw(),
            node_number: reg.node_number,
            auth_token: token.into_raw(),
        })
    }

    /// Create a zeroed/null instance.
    pub fn null() -> Self {
        Self {
            connectivity_group_id: ptr::null_mut(),
            node_number: 0,
            auth_token: ptr::null_mut(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wispers_registration_info_free(info: *mut WispersRegistrationInfo) {
    if info.is_null() {
        return;
    }
    unsafe {
        let info = &mut *info;
        if !info.connectivity_group_id.is_null() {
            drop(CString::from_raw(info.connectivity_group_id));
            info.connectivity_group_id = ptr::null_mut();
        }
        if !info.auth_token.is_null() {
            drop(CString::from_raw(info.auth_token));
            info.auth_token = ptr::null_mut();
        }
    }
}
