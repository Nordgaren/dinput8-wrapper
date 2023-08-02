use std::collections::HashMap;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::{fs, mem, thread};
use std::cell::{OnceCell, RefCell};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use log::warn;
use regex::Regex;

pub static mut FILES: Option<HashMap<String, Vec<String>>> = None;

pub unsafe fn init_file_processor() {
//     let (tx, rx) = mpsc::channel();
//
//     TX1 = Arc::new(MaybeUninit::new(tx.clone()));
//     TX2 = Arc::new(MaybeUninit::new(tx.clone()));
//
//     thread::spawn(|| {
//         file_processor_loop(rx);
//     });
}

pub unsafe fn file_processor_loop(rx: Receiver<String>) {
    for message in rx {
        if let Some(ref mut hashmap) = &mut FILES {
            let re = Regex::new(r"^([a-zA-Z0-9]*):([^:]*)$").unwrap();
            let string = &message;
            match re.captures(string) {
                Some(c) => {
                    if c.len() != 3 {
                        warn!("capture len incorrect. {}\n{}", c.len(), string);
                        return;
                    }
                    let key = c[1].to_string();
                    let val = c[2].to_string();
                    match hashmap.get_mut(&key) {
                        None => {
                            hashmap.insert(key, vec![val]);
                            ()
                        }
                        Some(v) => v.push(val),
                    }
                }
                None => return,
            };
        }
    }
}

pub unsafe fn file_processor_loop_manual(str: String) {
    println!("{}", str);
    if let Some(ref mut hashmap) = &mut FILES {
        let re = Regex::new(r"^([a-zA-Z0-9]*):([^:]*)$").unwrap();
        let string = &str;
        match re.captures(string) {
            Some(c) => {
                if c.len() != 3 {
                    warn!("capture len incorrect. {}\n{}", c.len(), string);
                    return;
                }
                let key = c[1].to_string();
                let val = c[2].to_string();
                match hashmap.get_mut(&key) {
                    None => {
                        hashmap.insert(key, vec![val]);
                        ()
                    }
                    Some(v) => v.push(val),
                }
            }
            None => return,
        };
    }
}

pub unsafe fn save_dump() {
    if let Some(files) = &mut FILES {
        let mut string = String::new();

        for (key, val) in files.iter_mut() {
            val.sort();
            val.dedup();

            string += &format!("#{}\n", key);
            string += &val.join("\n");
            string += "\n";
        }

        fs::write("./dump/file_paths.txt", string).unwrap();
    }
}
