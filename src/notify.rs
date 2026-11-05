use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_foundation::{NSBundle, NSError, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotificationRequest,
    UNUserNotificationCenter,
};

/// Ask macOS for permission to notify. macOS prompts once and remembers the
/// answer.
pub fn request_permission() {
    authorise(|| {});
}

/// Show a notification, if the user allowed them. A newer notification with
/// the same `id` replaces an older one.
///
/// Permission is checked every time and a refusal is never remembered here:
/// the first request can come back refused while the macOS prompt is still
/// open, and a later approval must still take effect.
pub fn send(id: &str, title: &str, body: &str) {
    let id = id.to_string();
    let title = title.to_string();
    let body = body.to_string();
    authorise(move || {
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(&title));
        content.setBody(&NSString::from_str(&body));
        let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
            &NSString::from_str(&id),
            &content,
            None,
        );
        let on_added = RcBlock::new(|error: *mut NSError| {
            if let Some(error) = unsafe { error.as_ref() } {
                tracing::error!(
                    "Failed to show notification: {}",
                    error.localizedDescription()
                );
            }
        });
        UNUserNotificationCenter::currentNotificationCenter()
            .addNotificationRequest_withCompletionHandler(&request, Some(&on_added));
    });
}

/// Run `then` once macOS allows notifications.
fn authorise(then: impl Fn() + 'static) {
    // UNUserNotificationCenter needs a bundle identifier, and the bare binary
    // from `just run` has none.
    if NSBundle::mainBundle().bundleIdentifier().is_none() {
        tracing::info!("Notifications need the app bundle, so none are shown");
        return;
    }

    let on_answer = RcBlock::new(move |granted: Bool, error: *mut NSError| {
        if granted.as_bool() {
            then();
            return;
        }
        let reason = unsafe { error.as_ref() }
            .map(|e| e.localizedDescription().to_string())
            .unwrap_or_else(|| "turned off in System Settings".to_string());
        tracing::warn!("Notifications are not allowed: {reason}");
    });
    UNUserNotificationCenter::currentNotificationCenter()
        .requestAuthorizationWithOptions_completionHandler(
            UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
            &on_answer,
        );
}
