#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

use std::{
    ffi::{c_long, c_ulong, c_void},
    ptr::null_mut,
};

use windows_sys::{
    core::{BOOL, HRESULT, PCSTR},
    Win32::Foundation::HMODULE,
};

pub const DETOUR_INSTRUCTION_TARGET_NONE: *mut c_void = null_mut();
pub const DETOUR_INSTRUCTION_TARGET_DYNAMIC: *mut c_void = -1i64 as *mut c_void;

#[repr(C)]
pub struct DETOUR_TRANSACTION_OPTIONS {
    pub fSuspendThreads: BOOL,
}

extern "system" {
    pub fn SlimDetoursTransactionBeginEx(pOptions: *mut DETOUR_TRANSACTION_OPTIONS) -> HRESULT;
}

#[inline(always)]
pub unsafe fn SlimDetoursTransactionBegin() -> HRESULT {
    let mut options = DETOUR_TRANSACTION_OPTIONS { fSuspendThreads: 1 };
    SlimDetoursTransactionBeginEx(&mut options)
}

extern "system" {
    pub fn SlimDetoursTransactionAbort() -> HRESULT;
    pub fn SlimDetoursTransactionCommit() -> HRESULT;

    pub fn SlimDetoursAttach(ppPointer: *mut *mut c_void, pDetour: *mut c_void) -> HRESULT;
    pub fn SlimDetoursDetach(ppPointer: *mut *mut c_void, pDetour: *mut c_void) -> HRESULT;

    pub fn SlimDetoursCodeFromPointer(pPointer: *mut c_void) -> *mut c_void;
    pub fn SlimDetoursCopyInstruction(
        pDst: *mut c_void,
        pSrc: *mut c_void,
        ppTarget: *mut *mut c_void,
        plExtra: *mut c_long,
    ) -> *mut c_void;

    pub fn SlimDetoursUninitialize() -> HRESULT;
}

#[repr(C)]
pub struct DETOUR_INLINE_HOOK {
    pub pszFuncName: PCSTR,
    pub ppPointer: *mut *mut c_void,
    pub pDetour: *mut c_void,
}

extern "system" {
    pub fn SlimDetoursInlineHook(
        bEnable: BOOL,
        ppPointer: *mut *mut c_void,
        pDetour: *mut c_void,
    ) -> HRESULT;

    pub fn SlimDetoursInitInlineHooks(
        hModule: HMODULE,
        ulCount: c_ulong,
        pHooks: *mut DETOUR_INLINE_HOOK,
    ) -> HRESULT;

    pub fn SlimDetoursInlineHooks(
        bEnable: BOOL,
        ulCount: c_ulong,
        pHooks: *mut DETOUR_INLINE_HOOK,
    ) -> HRESULT;
}

// Modified from detours-sys crate.
#[cfg(test)]
mod tests {
    use sync_unsafe_cell::SyncUnsafeCell;
    use windows_sys::Win32::System::{SystemInformation::GetTickCount, Threading::Sleep};

    use super::*;
    use std::{
        ffi::c_void,
        sync::atomic::{AtomicI32, Ordering},
    };

    static TRUE_SLEEP: SyncUnsafeCell<unsafe extern "system" fn(u32)> = SyncUnsafeCell::new(Sleep);
    static SLEPT: AtomicI32 = AtomicI32::new(0);

    // Detour function that replaces the Sleep API.
    unsafe extern "system" fn TimedSleep(dwMilliseconds: u32) {
        // Save the before and after times around calling the Sleep API.
        let dwBeg: u32 = GetTickCount();
        (*TRUE_SLEEP.get())(dwMilliseconds);
        let dwEnd: u32 = GetTickCount();

        SLEPT.store((dwEnd - dwBeg) as i32, Ordering::Release);
    }

    #[test]
    fn hook_self() {
        unsafe {
            let tru = TRUE_SLEEP.get() as *mut *mut c_void;
            let new = TimedSleep as *mut c_void;

            SlimDetoursInlineHook(1, tru, new);

            Sleep(500);
            let slept = SLEPT.load(Ordering::Acquire);
            assert_ne!(SLEPT.load(Ordering::Acquire), 0);

            SlimDetoursInlineHook(0, tru, new);
            Sleep(500);
            assert_eq!(slept, SLEPT.load(Ordering::Acquire));
        }
    }
}
