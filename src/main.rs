#![windows_subsystem = "windows"]

use winapi::{
    shared::{minwindef::*, windef::*, wtypesbase::CLSCTX_INPROC_SERVER},
    um::{
        combaseapi::*, libloaderapi::GetModuleHandleW, objbase::COINIT_APARTMENTTHREADED,
        processthreadsapi::GetCurrentThreadId, shobjidl::*, shobjidl_core::*, wingdi::*,
        winuser::*,
    },
};
use std::{
    ffi::OsString,  // Removed c_void
    fs,
    io::{self},
    os::windows::prelude::OsStringExt,
    ptr::null_mut,
    sync::{Arc, atomic::Ordering},
};

mod consts;
mod window_data;
mod input;
mod file_io;
mod controls;
mod utils;

use utils::*;
use controls::*;
use file_io::*;
use input::*;
use consts::*;
use window_data::WindowData;

fn get_hinstance() -> HINSTANCE {
    unsafe { GetModuleHandleW(std::ptr::null_mut()) }
}
fn main() -> io::Result<()> {
    let class_name_wide: Vec<u16> = CLASS_NAME.encode_utf16().chain(Some(0)).collect();

    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: get_hinstance(),
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: unsafe { GetSysColorBrush(COLOR_WINDOW + 1) },
        lpszMenuName: std::ptr::null_mut(),
        lpszClassName: class_name_wide.as_ptr(),
    };

    if unsafe { RegisterClassW(&wc) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let title: Vec<u16> = "Tty Sender\0".encode_utf16().collect();
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name_wide.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800,
            600,
            null_mut(),
            null_mut(),
            get_hinstance(),
            null_mut(),
        )
    };

    if hwnd.is_null() {
        return Err(io::Error::last_os_error());
    }

    let data = Box::new(WindowData::new());
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as LPARAM);
    }

    if let Err(e) = create_controls(hwnd) {
        unsafe {
            DestroyWindow(hwnd);
        }
        return Err(e);
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let mut msg = MSG {
        hwnd: null_mut(),
        message: 0,
        wParam: 0,
        lParam: 0,
        pt: POINT { x: 0, y: 0 },
        time: 0,
    };

    while unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) } != 0 {
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}
