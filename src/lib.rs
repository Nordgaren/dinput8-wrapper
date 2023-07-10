#![feature(naked_functions)]
#![allow(non_snake_case)]

mod dinput8;
mod util;

use crate::dinput8::init_dinput8;
use windows::Win32::Foundation::{HMODULE, MAX_PATH};
#[cfg(feature = "Console")]
use windows::Win32::System::Console::{AllocConsole, AttachConsole};
use windows::Win32::System::LibraryLoader::GetModuleFileNameA;

#[no_mangle]
#[allow(unused)]
pub extern "stdcall" fn DllMain(hinstDLL: isize, dwReason: u32, lpReserved: *mut usize) -> i32 {
    match dwReason {
        DLL_PROCESS_ATTACH => unsafe {
            #[cfg(feature = "Console")]
            {
                AllocConsole();
                AttachConsole(u32::MAX);
            }
            init(hinstDLL);
            1
        },
        _ => 0,
    }
}

unsafe fn init(hinstDLL: isize) -> String {
    let mut buffer = [0u8; MAX_PATH as usize + 1];
    let name_size = GetModuleFileNameA(
        HMODULE(hinstDLL),
        &mut buffer
    ) as usize;
    let name = &buffer[..name_size];
    let name_str = std::str::from_utf8(name).unwrap_or_default();
    if name_str.to_lowercase().ends_with("dinput8.dll") {
        init_dinput8();
    }

    name_str.to_string()
}
