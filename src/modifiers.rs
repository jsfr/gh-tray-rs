use objc2_app_kit::{NSEvent, NSEventModifierFlags};

/// The modifier keys held down right now.
///
/// Reads the current modifier state instead of an event's, so it is only
/// correct when called as the click is handled.
pub fn held() -> NSEventModifierFlags {
    NSEvent::modifierFlags_class()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_modifier_is_held_when_no_key_is_down() {
        assert!(!held().intersects(NSEventModifierFlags::Command | NSEventModifierFlags::Control));
    }
}
