use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::{Arc, atomic::{AtomicBool}};
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;

#[derive(Debug)]
pub struct WindowData {
    pub edit_handle: HWND,
    pub send_button_handle: HWND,
    pub load_button_handle: HWND,
    pub save_button_handle: HWND,
    pub file_path: PathBuf,
    pub com_initialized: bool,
    pub target_hwnd: HWND,
    pub attached_thread_id: DWORD,
    pub is_attached: bool,
    pub background_brush: Option<HBRUSH>,
    pub cursor_pos: i32,
    pub cursor_visible: bool,
    pub multi_send_button_handle: HWND,
    pub sender_thread: Option<std::thread::JoinHandle<()>>,
    pub abort_flag: Arc<AtomicBool>,
    pub thread_running: Arc<AtomicBool>,
}

impl WindowData {
    pub fn new() -> Self {
        Self {
            edit_handle: null_mut(),
            send_button_handle: null_mut(),
            load_button_handle: null_mut(),
            save_button_handle: null_mut(),
            file_path: PathBuf::new(),
            com_initialized: false,
            target_hwnd: null_mut(),
            attached_thread_id: 0,
            is_attached: false,
            background_brush: None,
            cursor_pos: 0,
            cursor_visible: false,
            multi_send_button_handle: null_mut(),
            sender_thread: None,
            abort_flag: Arc::new(AtomicBool::new(false)),
            thread_running: Arc::new(AtomicBool::new(false)),
        }
    }
}
