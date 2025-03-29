use super::*;

pub fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

pub fn show_message(hwnd: HWND, msg: &str) {
    show_message_with_color(hwnd, msg, RGB(0, 255, 0)); // 默认绿色
}


pub fn show_error_message(hwnd: HWND, msg: &str) {
    show_message_with_color(hwnd, msg, RGB(255, 0, 0)); // 错误红色
}

fn show_message_with_color(hwnd: HWND, msg: &str, color: COLORREF) {
    unsafe {
        let msg_handle = GetDlgItem(hwnd, 1007);
        if !msg_handle.is_null() {
            // 临时设置文本颜色
            let hdc = GetDC(msg_handle);
            SetTextColor(hdc, color);
            ReleaseDC(msg_handle, hdc);

            let wide_msg: Vec<u16> = msg.encode_utf16().chain(Some(0)).collect();
            SendMessageW(msg_handle, EM_REPLACESEL as u32, 0, wide_msg.as_ptr() as LPARAM);
            SendMessageW(msg_handle, EM_REPLACESEL as u32, 0, w("\r\n").as_ptr() as LPARAM);
        }
    }
}
