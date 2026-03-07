mod generated;
pub mod schema;

pub use generated::*;

pub const fn baseline_style_id() -> &'static str {
    BASELINE_STYLE_ID
}

const fn parse_px(raw: &str) -> i32 {
    let bytes = raw.as_bytes();
    let mut value = 0i32;
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_digit() {
            value = (value * 10) + (byte - b'0') as i32;
            index += 1;
            continue;
        }
        break;
    }
    value
}

pub const SHELL_TASKBAR_HEIGHT_PX: i32 = parse_px(SHELL_TASKBAR_HEIGHT);
pub const SHELL_TASKBAR_BUTTON_HEIGHT_PX: i32 = parse_px(SHELL_TASKBAR_BUTTON_HEIGHT);
pub const SHELL_TITLEBAR_HEIGHT_PX: i32 = parse_px(SHELL_TITLEBAR_HEIGHT);
