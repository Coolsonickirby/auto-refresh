use once_cell::sync::Lazy;
use skyline::hooks::InlineCtx;
use skyline::{hook, install_hooks};
use smash_arc::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::CStr;
use std::sync::Mutex;

use crate::resource;

struct HelperThreadedFileInfo {
    is_loaded: bool,
    data_ptr: u64,
    decompressed_size: usize,
}

#[repr(packed)]
struct ThreadedFileLoad {
    file_path_index: u32,
    padding: u32,
    data_ptr: *mut u8,
    decompressed_size: usize,
    // and more...
}

static THREADED_FILES: Lazy<Mutex<HashMap<u64, HelperThreadedFileInfo>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    Mutex::new(m)
});

fn check_size(loaded_bntx: &[u8], replace: &[u8]) -> Option<usize> {
    if loaded_bntx.len() > 0x1000 && replace.len() > 0x1000 {
        let loaded_image_size = usize::from_le_bytes(loaded_bntx[0xFF8..0x1000].try_into().ok()?);
        let replace_image_size = usize::from_le_bytes(replace[0xFF8..0x1000].try_into().ok()?);
        if loaded_image_size == replace_image_size {
            return Some(loaded_image_size);
        }
        return None;
    }
    return None;
}

pub fn handle_file_replace(hash: Hash40, replace: &[u8]) -> bool {
    let map = THREADED_FILES.lock().unwrap();
    if map.contains_key(&hash.as_u64()) {
        let loaded_image = map.get(&hash.as_u64()).unwrap();
        if loaded_image.is_loaded {
            let loaded_image_slice = unsafe {
                std::slice::from_raw_parts_mut(
                    loaded_image.data_ptr as *mut u8,
                    loaded_image.decompressed_size,
                )
            };
            if let Some(loaded_image_size) = check_size(loaded_image_slice, replace) {
                loaded_image_slice[0x1000..loaded_image_size]
                    .copy_from_slice(&replace[0x1000..loaded_image_size]);
                return true;
            }
            println!("[auto-refresh] Bntx file: {:#X} does not match loaded size, so the refresh request was rejected.", hash.as_u64());
            return false;
        }
    }

    println!(
        "[auto-refresh] Bntx file: {:#X} is not currently loaded.",
        hash.as_u64()
    );

    return false;
}

#[hook(offset = 0x37a17ac, inline)]
unsafe fn load_files_threaded_hook(ctx: &mut InlineCtx) {
    let threaded_load = *ctx.registers[19].x.as_ref() as *mut ThreadedFileLoad;

    let file_path_index = (*threaded_load).file_path_index as usize;

    let arc = resource::arc();

    if file_path_index < arc.get_file_paths().len() {
        let file_path = &arc.get_file_paths()[file_path_index];
        let path_hash = file_path.path.hash40();

        let decompressed_size = (*threaded_load).decompressed_size;

        THREADED_FILES.lock().unwrap().insert(
            path_hash.as_u64(),
            HelperThreadedFileInfo {
                is_loaded: true,
                data_ptr: (*threaded_load).data_ptr as u64,
                decompressed_size: decompressed_size,
            },
        );
    }
}

#[hook(offset = 0x37a1470, inline)]
unsafe fn free_files_threaded_hook(ctx: &mut InlineCtx) {
    let threaded_load = *ctx.registers[0].x.as_ref() as *mut ThreadedFileLoad;

    let file_path_index = (*threaded_load).file_path_index as usize;

    let arc = resource::arc();

    let mut map = THREADED_FILES.lock().unwrap();

    if file_path_index < arc.get_file_paths().len() {
        let file_path = &arc.get_file_paths()[file_path_index];
        let path_hash = file_path.path.hash40();

        if map.contains_key(&path_hash.as_u64()) {
            map.entry(path_hash.as_u64())
                .and_modify(|helper| (*helper).is_loaded = false);
        }
    } else if file_path_index == 0xFFFFFF {
        for (hash, load) in map.iter_mut() {
            if load.data_ptr == (*threaded_load).data_ptr as u64 {
                load.is_loaded = false;
            }
        }
    }
}

pub fn install() {
    install_hooks!(load_files_threaded_hook, free_files_threaded_hook);
}
