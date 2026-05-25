//! Notify Windows that proxy settings changed (WinINet + broadcast).

#[cfg(windows)]
pub fn notify_proxy_change() {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::Networking::WinInet::{
        InternetSetOptionW, INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    unsafe {
        let _ = InternetSetOptionW(
            None,
            INTERNET_OPTION_SETTINGS_CHANGED,
            None,
            0,
        );
        let _ = InternetSetOptionW(None, INTERNET_OPTION_REFRESH, None, 0);

        let msg: Vec<u16> = "Internet Settings\0".encode_utf16().collect();
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(msg.as_ptr() as _),
            SMTO_ABORTIFHUNG,
            1000,
            None,
        );
    }
}

#[cfg(not(windows))]
pub fn notify_proxy_change() {}
