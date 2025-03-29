use super::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUTTON_PROCS: Arc<Mutex<HashMap<isize, unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref DRAGGING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

pub fn create_controls(hwnd: HWND) -> io::Result<()> {
    let hinstance = get_hinstance();

    // 主编辑框 (左侧)
    let edit_handle = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w("EDIT\0").as_ptr(),
            null_mut(),
            (WS_CHILD | WS_VISIBLE | ES_MULTILINE | WS_HSCROLL | WS_VSCROLL | ES_AUTOVSCROLL | ES_WANTRETURN) as _,
            10, 10, 650, 500,  // 宽度减小为600
            hwnd,
            IDC_EDIT1 as isize as HMENU,
            hinstance,
            null_mut(),
        )
    };

    // 消息文本框 (左下)
    let msg_handle = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w("EDIT\0").as_ptr(),
            null_mut(),
            (WS_CHILD | WS_VISIBLE | ES_MULTILINE | WS_VSCROLL | ES_READONLY) as _,
            10, 520, 650, 60,  // 放在底部
            hwnd,
            1007 as isize as HMENU,  // 新ID
            hinstance,
            null_mut(),
        )
    };

    // 设置字体
    unsafe {
        let hfont = CreateFontW(
            16, 0, 0, 0, FW_NORMAL, 0, 0, 0,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE,
            w("Consolas\0").as_ptr(),
        );
        SendMessageW(edit_handle, WM_SETFONT, hfont as WPARAM, 1 as LPARAM);
        SendMessageW(msg_handle, WM_SETFONT, hfont as WPARAM, 1 as LPARAM);
    }

    // 右侧按钮 (竖排)
    let button_handles = [
        ("打开\0", IDC_BUTTON2),  // 打开
        ("保存\0", IDC_BUTTON3),  // 保存
        ("发送\0", IDC_BUTTON1),  // 发送
        ("多发\x00", IDC_BUTTON_MULTI),  // 多发
        ("绑定\0", IDC_BUTTON4),  // 绑定 (放在最下面)
    ]
    .iter()
    .enumerate()
    .map(|(i, (text, id))| {
        let y_pos = if *id == IDC_BUTTON4 { 520 } else { 10 + (i as i32) * 60 };  // 绑定按钮单独放下面
        let handle = unsafe {
            CreateWindowExW(
                0,
                w("BUTTON\0").as_ptr(),
                w(text).as_ptr(),
                (WS_CHILD | WS_VISIBLE | WS_TABSTOP) as DWORD,
                670,
                y_pos,
                100,
                40,
                hwnd,
                *id as isize as HMENU,
                hinstance,
                null_mut(),
            )
        };
        if handle.is_null() {
            show_error_message(hwnd, "CreateWindowExW failed");
            Err(io::Error::last_os_error())
        } else {
            if *id == IDC_BUTTON4 {
                subclass_button(handle)?;
            }
            Ok(handle)
        }
    })
    .collect::<Result<Vec<_>, _>>()?;

    // 更新WindowData
    let data_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowData };
    if !data_ptr.is_null() {
        let data = unsafe { &mut *data_ptr };
        data.edit_handle = edit_handle;
        data.send_button_handle = button_handles[2];  // 发送按钮
        data.load_button_handle = button_handles[0];  // 打开按钮
        data.save_button_handle = button_handles[1];  // 保存按钮
        data.multi_send_button_handle = button_handles[3];  // 多发按钮
    }

    Ok(())
}

pub fn subclass_button(hwnd: HWND) -> io::Result<()> {
    unsafe {
        let original_proc = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
        BUTTON_PROCS
            .lock()
            .unwrap()
            .insert(hwnd as isize, std::mem::transmute(original_proc));

        if SetWindowLongPtrW(hwnd, GWLP_WNDPROC, drag_button_proc as usize as isize) == 0 {
            show_error_message(null_mut(), "SetWindowLongPtrW failed");
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}


unsafe extern "system" fn drag_button_proc(
    hwnd: HWND,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let hwnd_key = hwnd as isize;
    let original_proc = BUTTON_PROCS.lock().unwrap().get(&hwnd_key).cloned();

    match u_msg {
        WM_LBUTTONDOWN => {
            *DRAGGING.lock().unwrap() = true;
            SetCapture(hwnd);
            if let Some(proc) = original_proc {
                CallWindowProcW(Some(proc), hwnd, u_msg, w_param, l_param)
            } else {
                DefWindowProcW(hwnd, u_msg, w_param, l_param)
            }
        }
        WM_MOUSEMOVE => 0,
        WM_LBUTTONUP => {
            *DRAGGING.lock().unwrap() = false;
            ReleaseCapture();
            show_error_message(hwnd, "神恶魔东西");
            let mut pt = POINT {
                x: LOWORD(l_param as DWORD) as i32,
                y: HIWORD(l_param as DWORD) as i32,
            };
            ClientToScreen(hwnd, &mut pt);

            let target_hwnd = WindowFromPoint(pt);
            let root_hwnd = GetAncestor(target_hwnd, GA_ROOT);

            if !root_hwnd.is_null() && IsWindow(root_hwnd) != 0 {
                let parent = GetParent(hwnd);
                let data_ptr = GetWindowLongPtrW(parent, GWLP_USERDATA) as *mut WindowData;
                if !data_ptr.is_null() {
                    let data = &mut *data_ptr;

                    if data.is_attached {
                        let current_thread_id = GetCurrentThreadId();
                        AttachThreadInput(current_thread_id, data.attached_thread_id, 0);
                        data.is_attached = false;
                    }

                    data.target_hwnd = root_hwnd;
                    show_message(parent, &format!("新窗口绑定成功: {:?}", root_hwnd));
                } else {
                    show_error_message(parent, "未找到窗口数据");
                }
            } else {
                show_error_message(hwnd, "无效的窗口");
            }

            if let Some(proc) = original_proc {
                CallWindowProcW(Some(proc), hwnd, u_msg, w_param, l_param)
            } else {
                DefWindowProcW(hwnd, u_msg, w_param, l_param)
            }
        }
        _ => {
            if let Some(proc) = original_proc {
                CallWindowProcW(Some(proc), hwnd, u_msg, w_param, l_param)
            } else {
                DefWindowProcW(hwnd, u_msg, w_param, l_param)
            }
        }
    }
}

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowData;

    match u_msg {
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {  // 同时处理编辑框和静态文本框
            let edit_hwnd = l_param as HWND;
            let ctrl_id = GetWindowLongPtrW(edit_hwnd, GWLP_ID) as i32;

            if ctrl_id == IDC_EDIT1 || ctrl_id == 1007 {  // 主编辑框或消息框
                if !data_ptr.is_null() {
                    let data = &mut *data_ptr;
                    if data.background_brush.is_none() {
                        data.background_brush = Some(CreateSolidBrush(RGB(64, 64, 64)));
                    }
                    SetTextColor(w_param as HDC,
                        if ctrl_id == 1007 { RGB(0, 255, 0) } else { RGB(255, 255, 255) });  // 消息框绿色，主编辑框白色
                    SetBkColor(w_param as HDC, RGB(64, 64, 64));
                    return data.background_brush.unwrap() as LRESULT;
                }
            }
            DefWindowProcW(hwnd, u_msg, w_param, l_param)
        }
        WM_DESTROY => {
            if !data_ptr.is_null() {
                let mut data = Box::from_raw(data_ptr);
                data.abort_flag.store(true, Ordering::SeqCst);
                if let Some(thread) = data.sender_thread.take() {
                    thread.join().unwrap();
                }
                if data.is_attached {
                    let current_thread_id = GetCurrentThreadId();
                    AttachThreadInput(current_thread_id, data.attached_thread_id, 0);
                }
                if let Some(brush) = data.background_brush {
                    DeleteObject(brush as _);
                }
            }
            PostQuitMessage(0);
            0
        }
        WM_COMMAND => {
            if data_ptr.is_null() {
                return DefWindowProcW(hwnd, u_msg, w_param, l_param);
            }
            let data = &mut *data_ptr;
            let cmd_id = LOWORD(w_param as DWORD) as i32;
            match cmd_id {
                IDC_BUTTON1 => send_line_to_window(data),
                IDC_BUTTON2 => load_file(data),
                IDC_BUTTON3 => save_file(data),
                IDC_BUTTON_MULTI => multi_send_lines(data),
                _ => (),
            }
            0
        }
        _ => DefWindowProcW(hwnd, u_msg, w_param, l_param),
    }
}
