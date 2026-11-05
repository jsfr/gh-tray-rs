use crate::types::HideCondition;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSApplication, NSButton, NSControlStateValueOn,
    NSLayoutAttribute, NSStackView, NSUserInterfaceLayoutOrientation, NSView,
};
use objc2_foundation::{NSArray, NSString};

/// Ask which of `options` a hidden PR should wait for. Returns the ticked
/// conditions, or `None` when the user cancels or ticks nothing.
///
/// Blocks until the dialog closes. Must run on the main thread.
pub fn choose_conditions(subject: &str, options: &[HideCondition]) -> Option<Vec<HideCondition>> {
    let mtm = MainThreadMarker::new().expect("dialogs run on the main thread");
    let alert = NSAlert::new(mtm);
    let boxes: Vec<Retained<NSButton>> = options
        .iter()
        .map(|condition| unsafe {
            NSButton::checkboxWithTitle_target_action(
                &NSString::from_str(condition.label()),
                None,
                None,
                mtm,
            )
        })
        .collect();

    if boxes.is_empty() {
        alert.setMessageText(&NSString::from_str("Nothing to wait for"));
        alert.setInformativeText(&NSString::from_str(&format!(
            "{subject} already meets every condition gh-tray can wait for."
        )));
    } else {
        alert.setMessageText(&NSString::from_str(&format!("Hide {subject} until:")));
        alert.setInformativeText(&NSString::from_str(
            "It comes back when everything you tick has happened.",
        ));
        let views: Vec<&NSView> = boxes.iter().map(|b| -> &NSView { b }).collect();
        let stack = NSStackView::stackViewWithViews(&NSArray::from_slice(&views), mtm);
        stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
        stack.setAlignment(NSLayoutAttribute::Leading);
        stack.setFrameSize(stack.fittingSize());
        alert.setAccessoryView(Some(&stack));
        alert.addButtonWithTitle(&NSString::from_str("Hide"));
        alert.addButtonWithTitle(&NSString::from_str("Cancel"));
    }

    // An accessory app is never frontmost on its own, so the alert would open
    // behind other windows. Hiding afterwards hands focus back.
    let app = NSApplication::sharedApplication(mtm);
    #[allow(deprecated)] // `activate` needs macOS 14, and the app supports 11
    app.activateIgnoringOtherApps(true);
    let response = alert.runModal();
    app.hide(None);

    if boxes.is_empty() || response != NSAlertFirstButtonReturn {
        return None;
    }
    let chosen: Vec<HideCondition> = options
        .iter()
        .zip(&boxes)
        .filter(|(_, b)| b.state() == NSControlStateValueOn)
        .map(|(condition, _)| *condition)
        .collect();
    (!chosen.is_empty()).then_some(chosen)
}
