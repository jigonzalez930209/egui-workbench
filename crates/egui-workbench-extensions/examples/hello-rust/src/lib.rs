//! A no_std Rust extension. Build with `cargo build --target wasm32-unknown-unknown --release`.
#![no_std]

#[link(wasm_import_module = "workbench")]
unsafe extern "C" {
    #[link_name = "host_register"]
    fn host_register(kind: i32, ptr: *const u8, len: i32) -> i32;
    #[link_name = "host_log"]
    fn host_log(ptr: *const u8, len: i32);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

static COMMAND: &[u8] = br#"{"id":"rust.run","title":"Hello from Rust"}"#;
static PANEL: &[u8] = br#"{"id":"rust.panel","title":"Rust panel"}"#;

#[unsafe(no_mangle)]
pub extern "C" fn init() -> i32 {
    unsafe {
        host_register(0, COMMAND.as_ptr(), COMMAND.len() as i32);
        host_register(1, PANEL.as_ptr(), PANEL.len() as i32);
        host_log(b"init".as_ptr(), 4);
    }
    0
}

static mut INBOX: [u8; 256] = [0; 256];

/// The host copies the command id into this buffer instead of guessing a free offset.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: i32) -> i32 {
    if len < 0 || len as usize > 256 {
        return -1;
    }
    core::ptr::addr_of_mut!(INBOX) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn on_command(ptr: *const u8, len: i32) -> i32 {
    unsafe {
        host_log(ptr, len);
    }
    0
}
