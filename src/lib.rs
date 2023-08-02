#![feature(naked_functions)]
#![allow(non_snake_case)]

mod dinput8;
mod util;
mod HashableString;
mod hooks;
mod dl_string;
mod path_processor;

use std::collections::HashMap;
use std::{mem, thread};
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::time::Duration;
use fisherman::hook::builder::HookBuilder;
use fisherman::scanner::signature::Signature;
use crate::dinput8::init_dinput8;
use windows::Win32::Foundation::{HMODULE, MAX_PATH};
#[cfg(feature = "Console")]
use windows::Win32::System::Console::{AllocConsole, AttachConsole};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameA, GetModuleHandleA};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use crate::hooks::{create_file_hook, CREATE_FILE_ORIGINAL, FnCreateFileW, get_file_attributes_hook, GET_FILE_ATTRIBUTES_ORIGINAL, get_file_hook, GET_FILE_ORIGINAL, open_file_hook, OPEN_FILE_ORIGINAL};
use log::*;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::*;
use windows::core::PCSTR;
use windows::imp::GetProcAddress;
use crate::path_processor::{FILES, save_dump};

static mut DIR: String = String::new();

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
            init_logs("log/file-logger.log");
            //init_file_processor();
            let path = init(hinstDLL);
            init_hooks(&path);
            1
        },
        DLL_PROCESS_DETACH => {
            unsafe {
                save_dump();
            }
            1
        }
        _ => 0,
    }
}


pub fn init_logs(file: &str) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}",
        )))
        .build();

    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}",
        )))
        .build(file)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(Logger::builder().build("bingo", LevelFilter::Trace))
        .logger(Logger::builder().build("file", LevelFilter::Trace))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    init_config(config).unwrap();

    log_panics::init();
}


unsafe fn init(hinstDLL: isize) -> String {
    let mut buffer = [0u8; MAX_PATH as usize + 1];
    let name_size = GetModuleFileNameA(
        HMODULE(hinstDLL),
        &mut buffer,
    ) as usize;
    let name = &buffer[..name_size];
    let name_str = std::str::from_utf8(name).unwrap_or_default();
    if name_str.to_lowercase().ends_with("dinput8.dll") {
        init_dinput8();
    }

    name_str.to_string()
}

unsafe fn init_hooks(name: &str) {
    let mut end = name.rfind("\\");
    if end == None {
        end = name.rfind("/");
    }

    let i = end.expect("Could not find parent directory.");
    let dir = &name[..i + 1];
    DIR = dir.to_string();

    let base = GetModuleHandleA(None).unwrap().0 as usize;
    FILES = Some(HashMap::new());
    //VEC = Some(Vec::new());

    let kernel32_handle = GetModuleHandleA(PCSTR::from_raw("KERNEL32.dll".as_ptr())).unwrap();
    let create_file_addr = GetProcAddress(kernel32_handle.0, PCSTR::from_raw("CreateFileW\0".as_ptr()));
    println!("{}", create_file_addr as usize);
    let get_file_attributes_addr = GetProcAddress(kernel32_handle.0, PCSTR::from_raw("GetFileAttributesW\0".as_ptr()));
    //CREATE_FILE_ORIGINAL = mem::transmute(create_file_addr);
    //GET_FILE_ORIGINAL = mem::transmute(GetProcAddress(handle.0, PCSTR::from_raw("CreateFileW".as_ptr())));
    let signature = Signature::from_ida_pattern("e8 ?? ?? ?? ?? 48 83 7b 20 08 48 8d 4b 08 72 03 48 8b 09 4c 8b 4b 18 41 b8 05 00 00 00 4d 3b c8").unwrap();
    HookBuilder::new()
        .add_inline_hook(
            base + 0x51b8f96,
            get_file_hook as usize,
            &mut GET_FILE_ORIGINAL,
            Some(base),
        )
        //     .add_inline_hook(
        //     base + 0x123340,
        //     insert_res_cap_hook as usize,
        //     &mut INSERT_RES_CAP_ORIGINAL,
        //     Some(base),
        // )
        //     .add_inline_hook(
        //         create_file_addr as usize,
        //     create_file_hook as usize,
        //     &mut CREATE_FILE_ORIGINAL,
        //     Some(kernel32_handle.0 as usize),
        // )
        // .add_inline_hook(
        //     base + 0x1f7a370,
        //     open_file_hook as usize,
        //     &mut OPEN_FILE_ORIGINAL,
        //     Some(kernel32_handle.0 as usize),
        // )
        //     .add_inline_hook(
        //         get_file_attributes_addr as usize,
        //         get_file_attributes_hook as usize,
        //         &mut GET_FILE_ATTRIBUTES_ORIGINAL,
        //         Some(kernel32_handle.0 as usize),
        // )
        // .add_iat_hook(
        //     "KERNEL32.dll",
        //     "CreateFileW",
        //     create_file_hook as usize,
        // )
        // .add_iat_hook(
        //     "KERNEL32.dll",
        //     "GetFileAttributesW",
        //     get_file_attributes_hook as usize,
        // )
        .build();
}

