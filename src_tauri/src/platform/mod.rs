#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(windows)]
pub use windows_lib::*;

#[cfg(windows)]
pub mod windows_lib;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use hbb_common::{message_proto::CursorData, ResultType};
#[cfg(not(target_os = "macos"))]
const SERVICE_INTERVAL: u64 = 300;

pub fn get_license_key() -> String {
    #[cfg(windows)]
    if let Some(lic) = windows_lib::get_license() {
        return lic.key;
    }
    Default::default()
}

pub fn is_xfce() -> bool {
    #[cfg(target_os = "linux")]
    {
        return std::env::var_os("XDG_CURRENT_DESKTOP") == Some(std::ffi::OsString::from("XFCE"));
    }
    #[cfg(not(target_os = "linux"))]
    {
        return false;
    }
}

// Android
#[cfg(target_os = "android")]
pub fn get_active_username() -> String {
    // TODO
    "android".into()
}

#[cfg(target_os = "android")]
pub const PA_SAMPLE_RATE: u32 = 48000;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cursor_data() {
        for _ in 0..30 {
            if let Some(hc) = get_cursor().unwrap() {
                let cd = get_cursor_data(hc).unwrap();
                repng::encode(
                    std::fs::File::create("cursor.png").unwrap(),
                    cd.width as _,
                    cd.height as _,
                    &cd.colors[..],
                )
                .unwrap();
            }
            #[cfg(target_os = "macos")]
            macos::is_process_trusted(false);
        }
    }
    #[test]
    fn test_get_cursor_pos() {
        for _ in 0..30 {
            assert!(!get_cursor_pos().is_none());
        }
    }
}
