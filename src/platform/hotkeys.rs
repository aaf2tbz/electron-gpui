//! System-wide shortcuts, the equivalent of Electron's `globalShortcut`. They fire
//! even when another app is frontmost. Carbon hot keys are delivered through the
//! application event loop and need no Accessibility permission. The handler only
//! sets a flag per shortcut; the UI polls `take` on a timer and acts on it.
//!
//! macOS only. On Linux, X11 could grab keys with `XGrabKey`, but Wayland has no
//! app-side global shortcuts (only the xdg-desktop-portal GlobalShortcuts portal,
//! which not every desktop implements), so `register` returns an error there.
//!
//! For shortcuts that only matter while your window is focused, use GPUI key
//! bindings (`cx.bind_keys`) instead; see `main.rs`.
use std::sync::atomic::{AtomicU64, Ordering};

/// Bit per registered shortcut id (0..64), set by the Carbon handler.
static PRESSED: AtomicU64 = AtomicU64::new(0);
#[cfg(target_os = "macos")]
static INSTALLED: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

/// Whether `register` can work on this platform.
pub const SUPPORTED: bool = cfg!(target_os = "macos");

#[allow(dead_code)]
pub mod modifiers {
    pub const CMD: u32 = 1 << 8;
    pub const SHIFT: u32 = 1 << 9;
    pub const OPTION: u32 = 1 << 11;
    pub const CONTROL: u32 = 1 << 12;
}

/// A few virtual key codes (Carbon `kVK_*`); see HIToolbox/Events.h for the rest.
#[allow(dead_code)]
pub mod keys {
    pub const A: u32 = 0x00;
    pub const S: u32 = 0x01;
    pub const D: u32 = 0x02;
    pub const G: u32 = 0x05;
    pub const K: u32 = 0x28;
    pub const P: u32 = 0x23;
    pub const Q: u32 = 0x0C;
    pub const SPACE: u32 = 0x31;
}

/// True once per press of the shortcut registered with `id`.
pub fn take(id: u32) -> bool {
    let bit = 1u64 << (id % 64);
    PRESSED.fetch_and(!bit, Ordering::Relaxed) & bit != 0
}

#[cfg(target_os = "macos")]
mod carbon {
    use std::ffi::c_void;

    #[repr(C)]
    pub struct EventTypeSpec {
        pub event_class: u32,
        pub event_kind: u32,
    }
    #[repr(C)]
    #[derive(Default)]
    pub struct EventHotKeyId {
        pub signature: u32,
        pub id: u32,
    }

    pub type Handler = extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> i32;

    #[link(name = "Carbon", kind = "framework")]
    unsafe extern "C" {
        pub fn GetApplicationEventTarget() -> *mut c_void;
        pub fn InstallEventHandler(
            target: *mut c_void,
            handler: Handler,
            num_types: usize,
            list: *const EventTypeSpec,
            user_data: *mut c_void,
            out_ref: *mut *mut c_void,
        ) -> i32;
        pub fn RegisterEventHotKey(
            key_code: u32,
            modifiers: u32,
            id: EventHotKeyId,
            target: *mut c_void,
            options: u32,
            out_ref: *mut *mut c_void,
        ) -> i32;
        pub fn GetEventParameter(
            event: *mut c_void,
            name: u32,
            desired_type: u32,
            actual_type: *mut u32,
            buffer_size: usize,
            actual_size: *mut usize,
            data: *mut c_void,
        ) -> i32;
    }

    pub const K_EVENT_CLASS_KEYBOARD: u32 = u32::from_be_bytes(*b"keyb");
    pub const K_EVENT_HOT_KEY_PRESSED: u32 = 5;
    pub const K_EVENT_PARAM_DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"----");
    pub const TYPE_EVENT_HOT_KEY_ID: u32 = u32::from_be_bytes(*b"hkid");
    /// Four-character tag for this app's hot keys; change it per app if you like.
    pub const SIGNATURE: u32 = u32::from_be_bytes(*b"EGPU");
}

#[cfg(target_os = "macos")]
extern "C" fn on_hot_key(
    _: *mut std::ffi::c_void,
    event: *mut std::ffi::c_void,
    _: *mut std::ffi::c_void,
) -> i32 {
    use carbon::*;
    let mut id = EventHotKeyId::default();
    let status = unsafe {
        GetEventParameter(
            event,
            K_EVENT_PARAM_DIRECT_OBJECT,
            TYPE_EVENT_HOT_KEY_ID,
            std::ptr::null_mut(),
            std::mem::size_of::<EventHotKeyId>(),
            std::ptr::null_mut(),
            (&mut id as *mut EventHotKeyId).cast(),
        )
    };
    if status == 0 && id.signature == SIGNATURE {
        PRESSED.fetch_or(1u64 << (id.id % 64), Ordering::Relaxed);
    }
    0
}

/// Register a global shortcut: `key_code` from `keys`, `modifiers` OR-ed from
/// `modifiers`, and an `id` (0..64) you later pass to `take`.
pub fn register(key_code: u32, modifiers: u32, id: u32) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    unsafe {
        use carbon::*;
        let target = GetApplicationEventTarget();
        let mut installed = INSTALLED.lock().unwrap();
        if !*installed {
            let spec = EventTypeSpec {
                event_class: K_EVENT_CLASS_KEYBOARD,
                event_kind: K_EVENT_HOT_KEY_PRESSED,
            };
            let mut handler = std::ptr::null_mut();
            if InstallEventHandler(
                target,
                on_hot_key,
                1,
                &spec,
                std::ptr::null_mut(),
                &mut handler,
            ) != 0
            {
                anyhow::bail!("Global shortcut handler not installed");
            }
            *installed = true;
        }
        let mut hot_key = std::ptr::null_mut();
        let status = RegisterEventHotKey(
            key_code,
            modifiers,
            EventHotKeyId {
                signature: SIGNATURE,
                id,
            },
            target,
            0,
            &mut hot_key,
        );
        if status != 0 {
            anyhow::bail!("Global shortcut {id} not registered (status {status}); another app may own it");
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (key_code, modifiers, id);
        anyhow::bail!("Global shortcuts are only implemented on macOS");
    }
    #[cfg(target_os = "macos")]
    Ok(())
}
