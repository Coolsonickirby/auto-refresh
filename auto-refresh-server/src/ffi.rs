use smash_arc::*;

#[no_mangle]
unsafe extern "C" fn auto_refresh_bntx(hash: u64, replace: *mut u8, size: usize) -> bool {
    let slice = std::slice::from_raw_parts_mut(replace, size);
    crate::bntx::handle_file_replace(Hash40::from(hash), slice)
}

#[no_mangle]
unsafe extern "C" fn auto_refresh_file(hash: u64, replace: *mut u8, size: usize) -> bool {
    let is_loaded = arcropolis_api::is_file_loaded(hash);
    if is_loaded {
        let fs = crate::resource::filesystem_info();
        let loaded_arc = &fs.path_info.arc;

        let file_info = loaded_arc
            .get_file_info_from_hash(Hash40::from(hash))
            .unwrap();
        let loaded_data = &fs.get_loaded_datas()[file_info.file_info_indice_index.0 as usize];

        if !loaded_data.data.is_null() {
            let decompressed_size = loaded_arc
                .get_file_data(file_info, Region::UsEnglish)
                .decomp_size;
            let slice = std::slice::from_raw_parts_mut(
                loaded_data.data as *mut u8,
                decompressed_size as usize,
            );
            let replace_slice = std::slice::from_raw_parts_mut(replace, size);
            slice.copy_from_slice(replace_slice);

            return true;
        }
        return false;
    }
    return false;
}
