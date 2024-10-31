#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

use std::{
    ffi::{c_long, c_ulong, c_void},
    ptr::null_mut,
};

use windows_sys::{
    core::{GUID, HRESULT, PCSTR, PCWSTR},
    Win32::Foundation::{BOOL, HMODULE},
};

pub const DETOUR_INSTRUCTION_TARGET_NONE: *mut c_void = null_mut();
pub const DETOUR_INSTRUCTION_TARGET_DYNAMIC: *mut c_void = -1i64 as *mut c_void;

extern "system" {
    pub fn SlimDetoursTransactionBegin() -> HRESULT;
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
}

#[repr(C)]
pub struct DETOUR_INLINE_HOOK {
    pszFuncName: PCSTR,
    ppPointer: *mut *mut c_void,
    pDetour: *mut c_void,
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
        hModule: HMODULE,
        ulCount: c_ulong,
        pHooks: *mut DETOUR_INLINE_HOOK,
    ) -> HRESULT;
}

#[repr(C)]
pub struct DETOUR_FUNC_TABLE_HOOK {
    ulOffset: c_ulong,
    ppOldFunc: *mut *mut c_void,
    pNewFunc: *mut c_void,
}

extern "system" {
    pub fn SlimDetoursFuncTableHook(
        pFuncTable: *mut *mut c_void,
        ulOffset: c_ulong,
        ppOldFunc: *mut *mut c_void,
        pNewFunc: *mut c_void,
    ) -> HRESULT;

    pub fn SlimDetoursFuncTableHooks(
        bEnable: BOOL,
        pFuncTable: *mut *mut c_void,
        ulCount: c_ulong,
        pHooks: *mut DETOUR_FUNC_TABLE_HOOK,
    ) -> HRESULT;

    pub fn SlimDetoursCOMHooks(
        bEnable: BOOL,
        rCLSID: &GUID,
        rIID: &GUID,
        ulCount: c_ulong,
        pHooks: *mut DETOUR_FUNC_TABLE_HOOK,
    ) -> HRESULT;
}

#[inline(always)]
pub unsafe fn SlimDetoursCOMHook(
    rCLSID: &GUID,
    rIID: &GUID,
    ulOffset: c_ulong,
    ppOldFunc: *mut *mut c_void,
    pNewFunc: *mut c_void,
) -> HRESULT {
    let mut hook = DETOUR_FUNC_TABLE_HOOK {
        ulOffset,
        ppOldFunc,
        pNewFunc,
    };
    let enable = if !ppOldFunc.is_null() { 1 } else { 0 };
    SlimDetoursCOMHooks(enable, rCLSID, rIID, 1, &mut hook)
}

pub type DETOUR_DELAY_ATTACH_CALLBACK = Option<
    unsafe extern "system" fn(
        Result: HRESULT,
        ppPointer: *mut *mut c_void,
        DllName: PCWSTR,
        Function: PCSTR,
        Context: *mut c_void,
    ),
>;
extern "system" {
    pub fn SlimDetoursDelayAttach(
        ppPointer: *mut *mut c_void,
        pDetour: *mut c_void,
        DllName: PCWSTR,
        Function: PCSTR,
        Callback: DETOUR_DELAY_ATTACH_CALLBACK,
        Context: *mut c_void,
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
