//! The C ABI.
//!
//! Every returned string is allocated with `CString::into_raw` and must be released with
//! [`wb_string_free`]. Errors are stored per thread and read with [`wb_last_error`].

use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_int};
use std::sync::Mutex;

use egui_workbench_host::{Host, HostSpec, Patch};

/// The opaque C host handle.
pub struct WbHost {
    host: Mutex<Option<Host>>,
}

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::new("").unwrap_or_default());
}

fn set_error(message: impl Into<String>) {
    let text = message.into();
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = CString::new(text).unwrap_or_default();
    });
}

fn text(value: &str) -> *mut c_char {
    CString::new(value)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// Create a host from a spec.
///
/// # Safety
/// `spec_json` must be a valid, null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wb_host_new(spec_json: *const c_char) -> *mut WbHost {
    if spec_json.is_null() {
        set_error("spec_json is null");
        return std::ptr::null_mut();
    }
    let json = unsafe { CStr::from_ptr(spec_json) }.to_string_lossy();
    let spec = match HostSpec::from_json(&json) {
        Ok(spec) => spec,
        Err(error) => {
            set_error(error);
            return std::ptr::null_mut();
        }
    };
    match Host::new(spec) {
        Ok(host) => Box::into_raw(Box::new(WbHost {
            host: Mutex::new(Some(host)),
        })),
        Err(error) => {
            set_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Free a host.
///
/// # Safety
/// `host` must come from [`wb_host_new`] and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wb_host_free(host: *mut WbHost) {
    if !host.is_null() {
        drop(unsafe { Box::from_raw(host) });
    }
}

/// Apply a patch.
///
/// # Safety
/// `host` must be a live handle and `patch_json` a null-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wb_host_apply_patch(host: *mut WbHost, patch_json: *const c_char) -> c_int {
    let Some(host) = (unsafe { host.as_ref() }) else {
        set_error("host is null");
        return -1;
    };
    if patch_json.is_null() {
        set_error("patch_json is null");
        return -1;
    }
    let json = unsafe { CStr::from_ptr(patch_json) }.to_string_lossy();
    let patch: Patch = match serde_json::from_str(&json) {
        Ok(patch) => patch,
        Err(error) => {
            set_error(error.to_string());
            return -1;
        }
    };
    let mut guard = host.host.lock().unwrap_or_else(|error| error.into_inner());
    let Some(host) = guard.as_mut() else {
        set_error("the host was consumed by wb_host_run");
        return -1;
    };
    match host.apply(&patch) {
        Ok(()) => 0,
        Err(error) => {
            set_error(error.to_string());
            -1
        }
    }
}

/// The current spec.
///
/// # Safety
/// `host` must be a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wb_host_spec(host: *mut WbHost) -> *mut c_char {
    let Some(host) = (unsafe { host.as_ref() }) else {
        set_error("host is null");
        return std::ptr::null_mut();
    };
    let guard = host.host.lock().unwrap_or_else(|error| error.into_inner());
    let Some(host) = guard.as_ref() else {
        set_error("the host was consumed by wb_host_run");
        return std::ptr::null_mut();
    };
    match serde_json::to_string(host.json()) {
        Ok(json) => text(&json),
        Err(error) => {
            set_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Drain events as a JSON array.
///
/// # Safety
/// `host` must be a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wb_host_poll_events(host: *mut WbHost) -> *mut c_char {
    let Some(host) = (unsafe { host.as_ref() }) else {
        set_error("host is null");
        return std::ptr::null_mut();
    };
    let mut guard = host.host.lock().unwrap_or_else(|error| error.into_inner());
    let Some(host) = guard.as_mut() else {
        set_error("the host was consumed by wb_host_run");
        return std::ptr::null_mut();
    };
    match serde_json::to_string(&host.poll_events()) {
        Ok(json) => text(&json),
        Err(error) => {
            set_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Run one headless frame.
///
/// # Safety
/// `host` must be a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wb_host_frame(host: *mut WbHost) -> *mut c_char {
    let Some(host) = (unsafe { host.as_ref() }) else {
        set_error("host is null");
        return std::ptr::null_mut();
    };
    let mut guard = host.host.lock().unwrap_or_else(|error| error.into_inner());
    let Some(host) = guard.as_mut() else {
        set_error("the host was consumed by wb_host_run");
        return std::ptr::null_mut();
    };
    match serde_json::to_string(&host.frame()) {
        Ok(json) => text(&json),
        Err(error) => {
            set_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Run the native window. Consumes the host state.
///
/// # Safety
/// `host` must be a live handle; it can only be run once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wb_host_run(host: *mut WbHost) -> c_int {
    let Some(host) = (unsafe { host.as_ref() }) else {
        set_error("host is null");
        return -1;
    };
    let mut guard = host.host.lock().unwrap_or_else(|error| error.into_inner());
    let Some(host) = guard.take() else {
        set_error("the host was already run");
        return -1;
    };
    drop(guard);
    match host.run() {
        Ok(()) => 0,
        Err(error) => {
            set_error(error.to_string());
            -1
        }
    }
}

/// The HostSpec JSON Schema.
#[unsafe(no_mangle)]
pub extern "C" fn wb_host_schema() -> *mut c_char {
    match serde_json::to_string_pretty(&HostSpec::schema()) {
        Ok(json) => text(&json),
        Err(error) => {
            set_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// The ABI version.
#[unsafe(no_mangle)]
pub extern "C" fn wb_host_version() -> *const c_char {
    c"0.13.0".as_ptr()
}

/// Free a string returned by this library.
///
/// # Safety
/// `text` must come from one of the `wb_*` functions that returns a string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wb_string_free(text: *mut c_char) {
    if !text.is_null() {
        drop(unsafe { CString::from_raw(text) });
    }
}

/// The last error for this thread.
#[unsafe(no_mangle)]
pub extern "C" fn wb_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| slot.borrow().as_ptr())
}
