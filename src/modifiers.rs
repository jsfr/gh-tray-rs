use objc2_app_kit::{NSEvent, NSEventModifierFlags};

/// True while the Shift key is held down.
///
/// Reads the current modifier state instead of an event's, so it is only
/// correct when called as the click is handled.
///
/// Shift rather than Option, which is the usual macOS modifier for an
/// alternate menu action: tiling window managers claim Option-click for
/// themselves, and the click then never reaches the menu.
pub fn shift_held() -> bool {
    NSEvent::modifierFlags_class().contains(NSEventModifierFlags::Shift)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_held_is_false_when_no_key_is_down() {
        assert!(!shift_held());
    }
}
