#![feature(proc_macro_hygiene)]

use once_cell::sync::Lazy;
use skyline::hooks::InlineCtx;
use skyline::{hook, install_hook};
use smash_arc::*;
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Mutex;
use std::thread;

#[macro_use]
extern crate lazy_static;

mod bntx;
mod ffi;
mod offsets;
mod resource;

const BIND_ADDR: &str = "0.0.0.0:7878";
const SCAN_DIR: &str = "sd:/ultimate/mods/Auto-Refresh/";

static mut FILES_INFO: Lazy<Vec<String>> = Lazy::new(|| vec![]);

pub fn refresh_file(path: &String) {
    unsafe {
        let file_hash = smash_arc::hash40(&path.to_owned());
        if path.ends_with("bntx") {
            if let Ok(data) = std::fs::read(Path::new(SCAN_DIR).join(path)) {
                bntx::handle_file_replace(file_hash, &data);
            }
        } else {
            let is_loaded = arcropolis_api::is_file_loaded(file_hash.as_u64());
            println!("[auto-refresh] Updating file contents...");
            if is_loaded {
                let fs = resource::filesystem_info();
                let loaded_arc = &fs.path_info.arc;

                let file_info = loaded_arc.get_file_info_from_hash(file_hash).unwrap();
                let loaded_data =
                    &fs.get_loaded_datas()[file_info.file_info_indice_index.0 as usize];

                if !loaded_data.data.is_null() {
                    let decompressed_size = loaded_arc
                        .get_file_data(file_info, Region::UsEnglish)
                        .decomp_size;
                    let slice = std::slice::from_raw_parts_mut(
                        loaded_data.data as *mut u8,
                        decompressed_size as usize,
                    );
                    println!("[auto-refresh] Overwriting buffer...");
                    match std::fs::read(Path::new(SCAN_DIR).join(path)) {
                        Ok(data) => slice.copy_from_slice(&data),
                        Err(err) => println!("[auto-refresh] Error: {:?}", err),
                    }
                }
            }
        }
    }
}

pub fn refresh_files() {
    unsafe {
        for file_path in FILES_INFO.iter() {
            refresh_file(&file_path);
        }
    }
}

pub fn handle_buffer(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(size) => {
            let data = String::from_utf8_lossy(&buffer[0..size]).into_owned();
            let data = data.trim();
            let mut response: String = "Refreshed!".to_string();
            println!("{}", data);
            let lines: Vec<&str> = data.split("\n").collect();
            if lines.len() >= 1 {
                for line in lines.iter() {
                    refresh_file(&line.to_string());
                }
            } else {
                refresh_files();
            }

            match stream.write(response.as_bytes()) {
                Ok(_ok) => {}
                Err(err) => {
                    println!("[auto-refresh] Stream Write Error: {:?}", err);
                }
            }
        }
        Err(err) => {
            println!("[auto-refresh] Network Error: {:?}", err);
            stream.flush().unwrap();
        }
    }

    stream.shutdown(std::net::Shutdown::Both);
}

pub fn scan_path_for_files(path: &Path) {
    match std::fs::read_dir(&path) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.unwrap();
                let real_path = format!("{}", entry.path().display());
                let path = Path::new(&real_path);
                if path.is_dir() {
                    scan_path_for_files(&path);
                } else {
                    let arc_path = &real_path[SCAN_DIR.len()..];
                    unsafe {
                        println!("{:?}", arc_path);
                        FILES_INFO.push(arc_path.to_string());
                    }
                }
            }
        }
        Err(err) => println!("[auto-refresh] Error: {:?}", err),
    }
}

#[skyline::main(name = "auto-refresh")]
pub fn main() {
    bntx::install();

    scan_path_for_files(Path::new(SCAN_DIR));

    thread::spawn(|| {
        let listener = TcpListener::bind(BIND_ADDR).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            println!("Connection established!");
            handle_buffer(stream);
        }
    });
}
