use winapi::um::winuser::*;
use super::*;

pub fn multi_send_lines(data: &mut WindowData) {
    if data.target_hwnd.is_null() {
        show_error_message(data.edit_handle, "未绑定目标窗口!");
        return;
    }

    if data.thread_running.load(Ordering::SeqCst) {
        show_error_message(data.edit_handle, "已有发送线程运行中");
        return;
    }

    data.abort_flag.store(false, Ordering::SeqCst);
    data.thread_running.store(true, Ordering::SeqCst);

    let target_hwnd = data.target_hwnd as isize;
    let edit_handle = data.edit_handle as isize;
    let abort_flag = Arc::clone(&data.abort_flag);
    let thread_running = Arc::clone(&data.thread_running);

    let thread_handler = std::thread::spawn(move || {
        unsafe {
            let target_hwnd = target_hwnd as HWND;
            let edit_handle = edit_handle as HWND;

            if IsWindow(target_hwnd) == 0 {
                show_error_message(edit_handle, "目标窗口已失效");
                thread_running.store(false, Ordering::SeqCst);
                return;
            }

            let mut thread_data = WindowData {
                edit_handle,
                target_hwnd,
                abort_flag: Arc::clone(&abort_flag),
                ..WindowData::new()
            };

            focus_target_window(&mut thread_data);

            loop {
                if abort_flag.load(Ordering::SeqCst) {
                    break;
                }

                if !send_one_line(&mut thread_data) {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            thread_running.store(false, Ordering::SeqCst);
        }
    });

    data.sender_thread = Some(thread_handler);
}

pub fn send_line_to_window(data: &mut WindowData) {
    if data.target_hwnd.is_null() {
        show_error_message(data.edit_handle, "No target window bound!");
        return;
    }

    // 检查线程是否在运行
    if data.thread_running.load(Ordering::SeqCst) {
        show_error_message(data.edit_handle, "已有发送线程运行中");
        return;
    }

    data.abort_flag.store(false, Ordering::SeqCst);
    data.thread_running.store(true, Ordering::SeqCst);

    let target_hwnd = data.target_hwnd as isize;
    let edit_handle = data.edit_handle as isize;
    let abort_flag = Arc::clone(&data.abort_flag);
    let thread_running = Arc::clone(&data.thread_running);

    let thread_handler = std::thread::spawn(move || {
        unsafe {
            let target_hwnd = target_hwnd as HWND;
            let edit_handle = edit_handle as HWND;

            if IsWindow(target_hwnd) == 0 {
                show_error_message(edit_handle, "目标窗口已失效");
                thread_running.store(false, Ordering::SeqCst);
                return;
            }

            let mut thread_data = WindowData {
                edit_handle,
                target_hwnd,
                abort_flag: Arc::clone(&abort_flag),
                ..WindowData::new()
            };

            focus_target_window(&mut thread_data);
            send_one_line(&mut thread_data);
            thread_running.store(false, Ordering::SeqCst);
        }
    });

    data.sender_thread = Some(thread_handler);
}

fn focus_target_window(data: &mut WindowData) {
    unsafe {
        let mut target_process_id = 0;
        let target_thread_id = GetWindowThreadProcessId(data.target_hwnd, &mut target_process_id);
        let current_thread_id = GetCurrentThreadId();

        if data.is_attached && data.attached_thread_id != target_thread_id {
            AttachThreadInput(current_thread_id, data.attached_thread_id, 0);
            data.is_attached = false;
        }

        if target_thread_id != current_thread_id && !data.is_attached {
            if AttachThreadInput(current_thread_id, target_thread_id, 1) != 0 {
                data.attached_thread_id = target_thread_id;
                data.is_attached = true;
            }
        }

        let mut retry_count = 3;
        while retry_count > 0 {
            ShowWindow(data.target_hwnd, SW_RESTORE);
            BringWindowToTop(data.target_hwnd);
            SetForegroundWindow(data.target_hwnd);
            SetFocus(data.target_hwnd);

            if GetForegroundWindow() == data.target_hwnd {
                break;
            }

            retry_count -= 1;
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

unsafe fn send_one_line(data: &mut WindowData) -> bool {
    let mut start: DWORD = 0;
    let mut end: DWORD = 0;
    SendMessageW(
        data.edit_handle,
        EM_GETSEL as u32,
        &mut start as *mut _ as WPARAM,
        &mut end as *mut _ as LPARAM,
    );

    let line_index =
        SendMessageW(data.edit_handle, EM_LINEFROMCHAR as u32, start as WPARAM, 0) as i32;
    let line_start = SendMessageW(
        data.edit_handle,
        EM_LINEINDEX as u32,
        line_index as WPARAM,
        0,
    ) as i32;
    let next_line_start = SendMessageW(
        data.edit_handle,
        EM_LINEINDEX as u32,
        (line_index + 1) as WPARAM,
        0,
    ) as i32;

    let length = if next_line_start == -1 {
        GetWindowTextLengthW(data.edit_handle) as i32 - line_start
    } else {
        next_line_start - line_start
    };

    if length <= 0 {
        return false;
    }
    if length > 1024 {
        show_error_message(data.edit_handle, &format!("Line too long, max length is 1024 but got {}!", length));
        return false;
    }

    let mut line_info = LINEW {
        cb: length as u16,
        psz_text: [0; 1024],
    };
    *line_info.psz_text.as_mut_ptr() = 1024;
    let copied = SendMessageW(
        data.edit_handle,
        EM_GETLINE as u32,
        line_index as WPARAM,
        &mut line_info as *mut _ as LPARAM,
    ) as i32;

    if copied == 0 {
        return false;
    }

    let line = String::from_utf16_lossy(&line_info.psz_text[..copied as usize]);
    for c in line.chars() {
        send_key(c);
    }
    send_enter();

    let next_line_pos = if next_line_start == -1 {
        let text_len = GetWindowTextLengthW(data.edit_handle) as i32;
        if line_start + length >= text_len {
            return false;
        }
        text_len
    } else {
        next_line_start
    };

    SendMessageW(
        data.edit_handle,
        EM_SETSEL as u32,
        next_line_pos as WPARAM,
        next_line_pos as LPARAM,
    );

    data.cursor_pos = next_line_pos;
    data.cursor_visible = true;
    InvalidateRect(data.edit_handle, null_mut(), TRUE);
    UpdateWindow(data.edit_handle);
    return true;
}

unsafe fn send_space() {
    let mut inputs = [
        INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        },
        INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        },
    ];

    inputs[0].u.ki_mut().wVk = VK_SPACE as u16;
    inputs[0].u.ki_mut().dwFlags = 0;

    inputs[1].u.ki_mut().wVk = VK_SPACE as u16;
    inputs[1].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;

    SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as _);
}

unsafe fn send_normal_key(c: char) {
    let mut inputs = [
        INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        },
        INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        },
    ];

    // 处理大写字母
    if c.is_uppercase() {
        // 先发送Shift键按下
        let mut shift_input = INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        };
        shift_input.u.ki_mut().wVk = VK_SHIFT as u16;
        shift_input.u.ki_mut().dwFlags = 0;
        SendInput(1, &mut shift_input as *mut _, std::mem::size_of::<INPUT>() as _);
    }

    // 使用UNICODE方式发送字符
    inputs[0].u.ki_mut().wScan = c as u16;
    inputs[0].u.ki_mut().dwFlags = KEYEVENTF_UNICODE;

    inputs[1].u.ki_mut().wScan = c as u16;
    inputs[1].u.ki_mut().dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

    SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as _);

    // 如果是大写字母，发送Shift键释放
    if c.is_uppercase() {
        let mut shift_input = INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        };
        shift_input.u.ki_mut().wVk = VK_SHIFT as u16;
        shift_input.u.ki_mut().dwFlags = KEYEVENTF_KEYUP;
        SendInput(1, &mut shift_input as *mut _, std::mem::size_of::<INPUT>() as _);
    }
}

fn send_key(c: char) {
    unsafe {
        if c == ' ' {
            // 空格仍然使用原来的方式
            send_space();
        } else {
            send_normal_key(c);
        }
    }
    // 适当增加延迟
    std::thread::sleep(std::time::Duration::from_millis(50));
}

unsafe fn send_enter() {
    let mut inputs = [
        INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        },
        INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        },
    ];

    // 仍然使用虚拟键码方式发送回车
    inputs[0].u.ki_mut().wVk = VK_RETURN as u16;
    inputs[0].u.ki_mut().dwFlags = 0;

    inputs[1].u.ki_mut().wVk = VK_RETURN as u16;
    inputs[1].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;

    SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as _);
    std::thread::sleep(std::time::Duration::from_millis(50));
}

#[repr(C)]struct LINEW {
    psz_text: [u16; 1024],
    cb: u16,
}
