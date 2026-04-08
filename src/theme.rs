/// Returns true if the OS is in dark mode.
pub fn is_dark_theme() -> bool {
    #[cfg(target_os = "macos")]
    {
        is_dark_macos()
    }

    #[cfg(target_os = "windows")]
    {
        is_dark_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

#[cfg(target_os = "macos")]
fn is_dark_macos() -> bool {
    std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .is_some_and(|s| s.trim() == "Dark")
}

#[cfg(target_os = "windows")]
fn is_dark_windows() -> bool {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .ok();

    key.and_then(|k| k.get_value::<u32, _>("SystemUsesLightTheme").ok())
        .is_some_and(|v| v == 0)
}
