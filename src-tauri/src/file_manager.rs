//! Reveal a file in the platform's file manager (Explorer, Finder, or a
//! FreeDesktop file manager) with the file selected where the platform
//! supports it.

use std::path::Path;
use std::process::Command;

/// Percent-encodes a path for use in a `file://` URI. Unreserved characters
/// and `/` are kept as-is; every other byte is escaped.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn path_to_file_uri(path: &Path) -> String {
    use std::fmt::Write;

    let mut uri = String::from("file://");
    for byte in path.to_string_lossy().bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                uri.push(byte as char)
            }
            _ => {
                let _ = write!(uri, "%{:02X}", byte);
            }
        }
    }
    uri
}

#[cfg(target_os = "windows")]
pub fn reveal_in_file_manager(path: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    // Explorer expects backslash separators and a single quoted argument, and
    // it exits with a non-zero status even on success, so only spawning is checked.
    let native_path = path.to_string_lossy().replace('/', "\\");
    Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", native_path))
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("Failed to open Explorer: {}", err))
}

#[cfg(target_os = "macos")]
pub fn reveal_in_file_manager(path: &Path) -> Result<(), String> {
    let status = Command::new("open")
        .arg("-R")
        .arg(path)
        .status()
        .map_err(|err| format!("Failed to open Finder: {}", err))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Finder exited with {}", status))
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn reveal_in_file_manager(path: &Path) -> Result<(), String> {
    // Preferred: the FreeDesktop FileManager1 D-Bus interface, which selects
    // the file in whichever file manager the desktop uses.
    let dbus_selected = Command::new("dbus-send")
        .args([
            "--session",
            "--print-reply",
            "--dest=org.freedesktop.FileManager1",
            "--type=method_call",
            "/org/freedesktop/FileManager1",
            "org.freedesktop.FileManager1.ShowItems",
        ])
        .arg(format!("array:string:{}", path_to_file_uri(path)))
        .arg("string:")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if dbus_selected {
        return Ok(());
    }

    // Fallback: open the containing folder with the default handler.
    let folder = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| "The file has no parent folder".to_string())?;
    let status = Command::new("xdg-open")
        .arg(folder)
        .status()
        .map_err(|err| format!("Failed to open the file manager: {}", err))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("xdg-open exited with {}", status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uri_keeps_unreserved_characters_and_separators() {
        assert_eq!(
            path_to_file_uri(Path::new("/home/user/Music/a-b_c.d~e/song.mp3")),
            "file:///home/user/Music/a-b_c.d~e/song.mp3"
        );
    }

    #[test]
    fn file_uri_escapes_spaces_and_reserved_characters() {
        assert_eq!(
            path_to_file_uri(Path::new("/music/AC DC/100% #1 (live).flac")),
            "file:///music/AC%20DC/100%25%20%231%20%28live%29.flac"
        );
    }

    #[test]
    fn file_uri_escapes_non_ascii_as_utf8_bytes() {
        assert_eq!(
            path_to_file_uri(Path::new("/music/Björk/é.mp3")),
            "file:///music/Bj%C3%B6rk/%C3%A9.mp3"
        );
    }
}
