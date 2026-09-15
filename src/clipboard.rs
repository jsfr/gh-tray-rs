/// Put text on the system clipboard.
///
/// macOS owns the pasteboard, so the text stays available after gh-tray exits.
pub fn copy(text: &str) {
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(e) => {
            tracing::error!("Failed to open the clipboard: {e}");
            return;
        }
    };

    match clipboard.set_text(text) {
        Ok(()) => tracing::info!("Copied to clipboard: {text}"),
        Err(e) => tracing::error!("Failed to write to the clipboard: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "writes to the real clipboard"]
    fn copy_puts_the_text_on_the_clipboard() {
        const URL: &str = "https://example.com/gh-tray-clipboard-test";
        let mut clipboard = arboard::Clipboard::new().expect("clipboard unavailable");
        let before = clipboard.get_text().ok();

        copy(URL);
        let after = clipboard.get_text().expect("clipboard read failed");

        if let Some(before) = before {
            let _ = clipboard.set_text(before);
        }
        assert_eq!(after, URL);
    }
}
