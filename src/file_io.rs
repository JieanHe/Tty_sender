use super::*;
use winapi::Interface;
use wio::com::ComPtr;

pub fn init_com() -> Result<(), i32> {
    unsafe {
        let hr = CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED);
        if hr < 0 {
            Err(hr)
        } else {
            Ok(())
        }
    }
}

pub fn load_file(data: &mut WindowData) {
    if !data.com_initialized {
        if let Err(_) = init_com() {
            return;
        }
        data.com_initialized = true;
    }

    unsafe {
        let mut pfd: *mut IFileOpenDialog = null_mut();
        let hr = CoCreateInstance(
            &CLSID_FileOpenDialog,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IFileOpenDialog::uuidof(),
            &mut pfd as *mut _ as *mut _,
        );

        if hr < 0 || pfd.is_null() {
            CoUninitialize();
            show_error_message(
                data.edit_handle,
                "Failed to create file open dialog",
            );
            return;
        }

        let dialog: ComPtr<IFileOpenDialog> = ComPtr::from_raw(pfd);
        let _ = dialog.SetTitle(w("Select File to Open\0").as_ptr());
        let _ = dialog.SetOptions(FOS_FORCEFILESYSTEM | FOS_ALLNONSTORAGEITEMS);

        let hr = dialog.Show(null_mut());
        if hr < 0 {
            CoUninitialize();
            show_error_message(
                data.edit_handle,
                "Failed to show file open dialog",
            );
            return;
        }

        let mut psi: *mut IShellItem = null_mut();
        if dialog.GetResult(&mut psi) >= 0 && !psi.is_null() {
            let item = ComPtr::from_raw(psi);
            let mut path_ptr: *mut u16 = null_mut();
            let hr = item.GetDisplayName(0x80028000, &mut path_ptr);
            if hr >= 0 && !path_ptr.is_null() {
                let len = (0..).take_while(|&i| *path_ptr.offset(i) != 0).count();
                let slice = std::slice::from_raw_parts(path_ptr, len);
                let os_str = OsString::from_wide(slice);
                data.file_path = os_str.into();

                if let Ok(contents) = fs::read_to_string(&data.file_path) {
                    let wide: Vec<u16> = contents.encode_utf16().chain(Some(0)).collect();
                    SetWindowTextW(data.edit_handle, wide.as_ptr());
                }
                CoTaskMemFree(path_ptr as _);
            }
        }
        CoUninitialize();
    }
}

pub fn save_file(data: &mut WindowData) {
    if !data.com_initialized {
        if let Err(_) = init_com() {
            return;
        }
        data.com_initialized = true;
    }

    unsafe {
        let mut pfd: *mut IFileSaveDialog = null_mut();
        let hr = CoCreateInstance(
            &CLSID_FileSaveDialog,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IFileSaveDialog::uuidof(),
            &mut pfd as *mut _ as *mut _,
        );

        if hr < 0 || pfd.is_null() {
            CoUninitialize();
            show_error_message(data.edit_handle, "Failed to create file save dialog");
            return;
        }

        let dialog: ComPtr<IFileSaveDialog> = ComPtr::from_raw(pfd);
        let _ = dialog.SetTitle(w("Save File As\0").as_ptr());
        let _ = dialog.SetOptions(FOS_OVERWRITEPROMPT | FOS_FORCEFILESYSTEM);

        let hr = dialog.Show(null_mut());
        if hr < 0 {
            CoUninitialize();
            show_error_message(
                data.edit_handle,
                "Failed to show file save dialog",
            );
            return;
        }

        let mut psi: *mut IShellItem = null_mut();
        if dialog.GetResult(&mut psi) >= 0 && !psi.is_null() {
            let item = ComPtr::from_raw(psi);
            let mut path_ptr: *mut u16 = null_mut();
            let hr = item.GetDisplayName(0x80028000, &mut path_ptr);
            if hr >= 0 && !path_ptr.is_null() {
                let len = (0..).take_while(|&i| *path_ptr.offset(i) != 0).count();
                let slice = std::slice::from_raw_parts(path_ptr, len);
                let os_str = OsString::from_wide(slice);
                data.file_path = os_str.into();

                let length = GetWindowTextLengthW(data.edit_handle) as usize;
                if length > 0 {
                    let mut buffer = vec![0u16; length + 1];
                    GetWindowTextW(data.edit_handle, buffer.as_mut_ptr(), (length + 1) as i32);
                    if let Ok(text) = String::from_utf16(&buffer[..length]) {
                        fs::write(&data.file_path, text).unwrap();
                    }
                }
                CoTaskMemFree(path_ptr as _);
            }
        }
        CoUninitialize();
    }
}
