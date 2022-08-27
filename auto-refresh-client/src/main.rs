#![allow(dead_code)]

mod gui;
use eframe::epaint::Vec2;
use gui::*;

use eframe::{run_native, NativeOptions, HardwareAcceleration, Renderer};
use ftp::FtpStream;
use notify::{watcher, DebouncedEvent::*, RecursiveMode, Watcher};
use nutexb::NutexbFile;
use std::io::{Cursor, Write};
use std::net::TcpStream;
use std::path::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
const WINDOW_SIZE: Vec2 = Vec2::new(430.0, 210.0);

enum ConversionType {
    Image,
    Invalid,
}

fn is_smash_extension(extension: &str) -> bool {
    matches!(
        extension,
        "prc"
            | "xmb"
            | "nuhlpb"
            | "numatb"
            | "numshb"
            | "numdlb"
            | "nusktb"
            | "numshexb"
            | "nusrcmdlb"
            | "nutexb"
            | "lc"
            | "arc"
            | "bntx"
            | "msbt"
            | "nuanmb"
            | "bin"
            | "shpcanim"
            | "stdat"
            | "lvd"
            | "stprm"
            | "shpc"
            | "nus3bank"
            | "tonelabel"
            | "nus3audio"
            | "sli"
            | "csb"
            | "svt"
            | "spt"
            | "fnv"
            | "nus3conf"
            | "sqb"
            | "eff"
            | "nushdb"
            | "nufxlb"
            | "webm"
            | "adjb"
            | "bfttf"
            | "bfotf"
            | "nro"
            | "h264"
    )
}

fn is_convertable_format(extension: &str) -> ConversionType {
    match extension {
        "png" | "dds" => ConversionType::Image,
        _ => ConversionType::Invalid,
    }
}

fn convert_to_bntx(path: &PathBuf) {
    let mut output_path = path.clone();
    output_path.set_extension("bntx");
    // let output_name = path.file_stem().unwrap().to_str().unwrap();
    let image = image::open(&path).unwrap();
    bntx::BntxFile::from_image(image, "file")
        .save(output_path)
        .unwrap();
}

fn convert_to_nutexb(path: &PathBuf) {
    let mut output_path = path.clone();
    output_path.set_extension("nutexb");
    let output_name = path.file_stem().unwrap().to_str().unwrap();
    let image = nutexb::image::open(&path).unwrap();
    let nutexb = NutexbFile::create(&image, output_name).unwrap();
    nutexb.write_to_file(&output_path).unwrap();
}

fn refresh_file_on_server(arc_path: &PathBuf, data: &Arc<Mutex<Data>>) {
    let address = format!("{}:7878", data.lock().unwrap().switch_ip);
    println!("[refresh_file_on_server] Attempting to connect to {}", address);
    match TcpStream::connect("10.0.0.143:7878") {
        Ok(mut stream) => {
            println!("[refresh_file_on_server] Successfully connected to auto-refresh-server!");
            stream
                .write(
                    format!("{}\n", arc_path.display())
                        .replace("\\", "/")
                        .as_bytes(),
                )
                .unwrap();
        }
        Err(e) => {
            println!("[refresh_file_on_server] Failed to connect: {}", e);
        }
    }
}

fn upload_file_to_ftp(arc_path: &PathBuf, data: &Arc<Mutex<Data>>) {
    // Create a connection to an FTP server and authenticate to it.
    let ip = data.lock().unwrap().switch_ip.clone();
    let ftp_port = &data.lock().unwrap().ftp_port.clone();
    let address = format!("{}:{}", ip, ftp_port);
    println!("[upload_file_to_ftp] Attempting to connect to {}", address);
    let mut ftp_stream = FtpStream::connect(address).unwrap();
    println!("[upload_file_to_ftp] Connection successful!");
    let _ = ftp_stream.login("", "").unwrap();

    // // Get the current directory that the client will be reading from and writing to.
    println!("[upload_file_to_ftp] Current directory: {}", ftp_stream.pwd().unwrap());

    let output_name = arc_path.file_name().unwrap().to_str().unwrap();
    let ftp_path = &format!("{}\\{}", data.lock().unwrap().target_path.replace("ftp:", "."), arc_path.display());
    let ftp_path = ftp_path.replace("/", "\\");
    let mut folders = ftp_path.split('\\').collect::<Vec<&str>>();
    folders.pop();

    println!("{}", folders.join("/"));
    for folder in folders {
        match ftp_stream.mkdir(folder) {
            Ok(_ok) => {}
            Err(_err) => {}
        }
        match ftp_stream.cwd(folder) {
            Ok(_ok) => {}
            Err(_err) => {}
        }
    }

    // Store (PUT) a file from the client to the current working directory of the server.
    let physical_path = Path::new(&data.lock().unwrap().watch_path).join(arc_path);
    let mut reader = Cursor::new(std::fs::read(physical_path).unwrap());
    let _ = ftp_stream.put(output_name, &mut reader);
    println!("[upload_file_to_ftp] Successfully wrote {}", output_name);

    // Terminate the connection to the server.
    let _ = ftp_stream.quit();
}

fn handle_path(path: &PathBuf, data: &Arc<Mutex<Data>>) {
    if !path.is_file() {
        return;
    }

    let extension = path.extension().unwrap().to_str().unwrap();
    if !is_smash_extension(extension) {
        // Convert it if possible
        match is_convertable_format(extension) {
            ConversionType::Image => {
                if format!("{}", path.display())[data.lock().unwrap().watch_path.len() + 1..].starts_with("ui\\") {
                    println!("[handle_path] Converting {} to BNTX...", path.display());
                    convert_to_bntx(path);
                } else {
                    println!("[handle_path] Converting {} to nutexb...", path.display());
                    convert_to_nutexb(path);
                }
            }
            _ => println!("[handle_path] Not a convertable format!"),
        }

        return;
    }

    let arc_path = PathBuf::from(&format!("{}", path.display())[data.lock().unwrap().watch_path.len() + 1..]);
    
    if data.lock().unwrap().target_path.starts_with("ftp:") {
        // Upload path to FTP
        println!("Uploading to ftp!");
        upload_file_to_ftp(&arc_path, data);
    } else {
        // Copy file to target path
        let target_path = Path::new(&data.lock().unwrap().target_path).join(&arc_path);
        let parent_path = target_path.parent().unwrap();
        match std::fs::create_dir_all(parent_path) {
            Ok(ok) => println!("[handle_path] Target path created successfully! {:?}", ok),
            Err(err) => println!("[handle_path] Failed creating target path! {:?}", err)
        }
        match std::fs::copy(path, target_path) {
            Ok(ok) => println!("[handle_path] File successfully copied! {:?}", ok),
            Err(err) => println!("[handle_path] Failed copying file! {:?}", err)
        }
    }

    // Refresh the file on the server
    refresh_file_on_server(&arc_path, data);
}

fn setup_watcher(data: Arc<Mutex<Data>>) {
    loop {
        if !data.lock().unwrap().is_watching {
            continue;
        }
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        watcher.watch(&data.lock().unwrap().watch_path, RecursiveMode::Recursive).unwrap();
        loop {
            match rx.recv() {
                Ok(event) => match event {
                    Create(mut path) => {
                        handle_path(&mut path, &data);
                    }
                    Write(mut path) => {
                        handle_path(&mut path, &data);
                    }
                    _ => {}
                },
                Err(e) => println!("watch error: {:?}", e),
            }

            if !data.lock().unwrap().is_watching {
                break;
            }
        }
    }
}

fn main() {
    let app = MainApp::default();
    let ref_data = app.data.clone();
    let win_option = NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: true,
        fullscreen: false,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_pos: None,
        initial_window_size: Some(WINDOW_SIZE),
        min_window_size: Some(WINDOW_SIZE),
        max_window_size: Some(WINDOW_SIZE),
        resizable: false,
        transparent: false,
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: HardwareAcceleration::Preferred,
        renderer: Renderer::default(),
        follow_system_theme: cfg!(target_os = "macos") || cfg!(target_os = "windows"),
        default_theme: eframe::Theme::Dark,
        run_and_return: true,
    };

    thread::spawn(move || {
        setup_watcher(ref_data);
    });
    run_native(
        "Auto-Refresh Client",
        win_option,
        Box::new(|_cc| Box::new(app)),
    )
    // setup_watcher();
}
