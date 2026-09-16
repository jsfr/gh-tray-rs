mod clipboard;
mod config;
mod demo;
mod github;
mod logging;
mod modifiers;
mod theme;
mod tray;
mod types;

use clap::Parser;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};
use muda::MenuEvent;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSEventMask};
use objc2_foundation::{NSDate, NSDefaultRunLoopMode};
use std::sync::mpsc;
use std::time::Duration;
use tray_icon::TrayIconBuilder;

/// How long the pump waits for an AppKit event before draining queues again.
const PUMP_TIMEOUT_SECS: f64 = 0.25;

/// Menu clicks, each paired with the modifier state read as the click happened.
static MENU_CLICKS: std::sync::Mutex<Vec<(muda::MenuId, bool)>> = std::sync::Mutex::new(Vec::new());

#[derive(Parser)]
#[command(name = "gh-tray", about = "GitHub PR monitor in your system tray")]
struct Cli {
    /// Run with fake PR data for visual testing
    #[arg(long)]
    demo: bool,
}

/// Messages sent from the polling thread to the main thread.
enum PollMessage {
    Update(types::PullRequestGroup),
    Stale,
}

struct App {
    tray_icon: Option<tray_icon::TrayIcon>,
    menu_actions: std::collections::HashMap<muda::MenuId, tray::MenuAction>,
    last_group: types::PullRequestGroup,
    rx: mpsc::Receiver<PollMessage>,
    auto_launch: Option<auto_launch::AutoLaunch>,
    auto_start_enabled: bool,
    last_updated: Option<String>,
    is_stale: bool,
    should_exit: bool,
}

impl App {
    /// Create the status item. NSApp has to exist first, so this runs after
    /// `finishLaunching` rather than in the constructor.
    fn create_tray_icon(&mut self) {
        let is_dark = theme::is_dark_theme();
        let icon = tray::render_icon("...", is_dark);
        let (loading_menu, loading_actions) =
            tray::build_menu(&types::PullRequestGroup::default(), false, None, false);

        let tray_icon = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(loading_menu))
            .with_tooltip("gh-tray: loading...")
            .build()
            .expect("Failed to create tray icon");

        self.tray_icon = Some(tray_icon);
        self.menu_actions = loading_actions;
    }

    /// Handle everything queued since the last pass: menu clicks, hotkeys and
    /// poll results.
    fn drain(&mut self) {
        // Process menu events
        let clicks: Vec<(muda::MenuId, bool)> = MENU_CLICKS
            .lock()
            .map(|mut clicks| clicks.drain(..).collect())
            .unwrap_or_default();
        for (id, shift_held) in clicks {
            if let Some(action) = self.menu_actions.get(&id).cloned() {
                match tray::apply_modifier(action, shift_held) {
                    tray::MenuAction::OpenUrl(url) => {
                        let _ = open::that(&url);
                    }
                    tray::MenuAction::CopyUrl(url) => {
                        clipboard::copy(&url);
                    }
                    tray::MenuAction::ToggleAutoStart => {
                        if let Some(al) = &self.auto_launch {
                            let new_state = !self.auto_start_enabled;
                            let result = if new_state { al.enable() } else { al.disable() };
                            if result.is_ok() {
                                self.auto_start_enabled = new_state;
                            }
                        }
                        self.rebuild_menu();
                    }
                    tray::MenuAction::Quit => {
                        self.should_exit = true;
                    }
                }
            }
        }

        // Process hotkey events
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            tracing::debug!("Hotkey pressed: {:?}", event);
        }

        // Process poll messages
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                PollMessage::Update(group) => {
                    let now = local_time_now();
                    self.last_updated = Some(now);
                    self.is_stale = false;
                    self.last_group = group;

                    let count = self.last_group.total_count();
                    let is_dark = theme::is_dark_theme();
                    let icon = tray::render_icon(&count.to_string(), is_dark);

                    if let Some(tray) = &self.tray_icon {
                        let _ = tray.set_icon(Some(icon));
                        let _ = tray.set_tooltip(Some(&format!("gh-tray: {count} PRs")));
                        tracing::debug!("Tray updated: {count} PRs");
                    }

                    self.rebuild_menu();
                }
                PollMessage::Stale => {
                    self.is_stale = true;
                    self.rebuild_menu();
                }
            }
        }
    }

    fn rebuild_menu(&mut self) {
        let (menu, actions) = tray::build_menu(
            &self.last_group,
            self.is_stale,
            self.last_updated.as_deref(),
            self.auto_start_enabled,
        );
        self.menu_actions = actions;
        if let Some(tray) = &self.tray_icon {
            tray.set_menu(Some(Box::new(menu)));
        }
    }
}

fn local_time_now() -> String {
    use std::mem::MaybeUninit;
    unsafe {
        let time = libc::time(std::ptr::null_mut());
        let mut tm = MaybeUninit::uninit();
        libc::localtime_r(&time, tm.as_mut_ptr());
        let tm = tm.assume_init();
        format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
    }
}

/// GUI-launched macOS apps inherit launchd's PATH (`/usr/bin:/bin:/usr/sbin:/sbin`),
/// which omits Homebrew. `gh` (installed via brew) and the `git` it invokes
/// internally then can't be found, and the app exits before showing a tray icon.
/// Prepend the standard Homebrew prefixes so child processes can resolve them.
fn ensure_homebrew_in_path() {
    const HOMEBREW_BIN_DIRS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin"];

    let extras: Vec<std::path::PathBuf> = HOMEBREW_BIN_DIRS
        .iter()
        .map(std::path::PathBuf::from)
        .filter(|p| p.exists())
        .collect();

    if extras.is_empty() {
        return;
    }

    let current = std::env::var_os("PATH").unwrap_or_default();
    let existing: Vec<std::path::PathBuf> = std::env::split_paths(&current).collect();
    let combined = extras.into_iter().chain(existing);

    if let Ok(joined) = std::env::join_paths(combined) {
        // SAFETY: called from main before any threads are spawned.
        unsafe { std::env::set_var("PATH", joined) };
    }
}

/// macOS Tahoe (26.x) crashes (`SIGBUS` / `EXC_ARM_DA_ALIGN`) inside the ImageIO
/// PNG plugin when a tray-icon NSImage is decoded in a process whose parent is
/// an adhoc-signed binary (e.g. Homebrew `fish`). Re-execing through the
/// Apple platform-signed `/usr/bin/env` resets the inherited security context
/// and avoids the crash.
///
/// Skip when launched from inside an `.app` bundle: launchd is already a
/// platform binary, and re-execing detaches the process from its bundle
/// context (NSApplication then fails to register the status item).
/// Honors `GH_TRAY_NO_REEXEC` for opting out.
fn reexec_via_platform_binary() {
    use std::os::unix::process::CommandExt;

    const SENTINEL: &str = "GH_TRAY_REEXECED";
    if std::env::var_os(SENTINEL).is_some() || std::env::var_os("GH_TRAY_NO_REEXEC").is_some() {
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    if exe.to_string_lossy().contains(".app/Contents/MacOS/") {
        return;
    }
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let _ = std::process::Command::new("/usr/bin/env")
        .arg(exe)
        .args(args)
        .env(SENTINEL, "1")
        .exec();
}

fn main() {
    reexec_via_platform_binary();
    ensure_homebrew_in_path();

    // muda runs this handler while the click is still being dispatched, so the
    // modifier state it reads is the one the user held. It must be registered
    // before any menu interaction: muda's handler slot is a OnceCell that the
    // first menu event initialises to None, and a later registration is
    // silently dropped.
    MenuEvent::set_event_handler(Some(|event: MenuEvent| {
        let shift_held = modifiers::shift_held();
        if let Ok(mut clicks) = MENU_CLICKS.lock() {
            clicks.push((event.id, shift_held));
        }
    }));

    let cli = Cli::parse();
    let mut config = config::load();
    config::apply_env_overrides(&mut config);

    logging::init(config.log_level, config.log_file.as_deref());

    // Resolve auth token
    let token = if cli.demo {
        None
    } else {
        match &config.account {
            Some(account) => match github::resolve_token(account) {
                Ok(t) => Some(t),
                Err(e) => {
                    eprintln!("Failed to get token for account '{account}': {e}");
                    None
                }
            },
            None => None,
        }
    };

    // Validate auth
    if !cli.demo
        && let Err(e) = github::validate_auth(token.as_deref())
    {
        eprintln!("gh CLI authentication failed: {e}");
        eprintln!("Please run 'gh auth login' first.");
        std::process::exit(1);
    }

    // Accessory activation policy keeps the app out of the Dock. The bundled
    // .app also sets LSUIElement, so this matters when running the bare binary.
    let mtm = MainThreadMarker::new().expect("main runs on the main thread");
    let ns_app = NSApplication::sharedApplication(mtm);
    ns_app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Set up auto-launch
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let auto_launch = auto_launch::AutoLaunchBuilder::new()
        .set_app_name("gh-tray")
        .set_app_path(&exe_path)
        .build()
        .ok();

    let auto_start_enabled = auto_launch
        .as_ref()
        .and_then(|al| al.is_enabled().ok())
        .unwrap_or(false);

    // Spawn polling thread
    let (tx, rx) = mpsc::channel();
    let poll_interval = config.poll_interval;
    let token_clone = token.clone();
    let demo = cli.demo;

    std::thread::spawn(move || {
        if demo {
            let group = demo::demo_pull_requests();
            tracing::info!("Demo mode: {} PRs", group.total_count());
            let _ = tx.send(PollMessage::Update(group));
            loop {
                std::thread::sleep(Duration::from_secs(3600));
            }
        }

        let username = match github::get_username(token_clone.as_deref()) {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("Failed to get username: {e}");
                return;
            }
        };

        tracing::info!("Polling PRs for user: {username}");

        loop {
            match github::fetch_pull_requests(token_clone.as_deref(), &username) {
                Ok(group) => {
                    tracing::info!("Fetched {} PRs", group.total_count());
                    let _ = tx.send(PollMessage::Update(group));
                }
                Err(e) => {
                    tracing::error!("Failed to fetch PRs: {e}");
                    let _ = tx.send(PollMessage::Stale);
                }
            }

            std::thread::sleep(poll_interval);
        }
    });

    // Register global hotkey
    let _hotkey_manager = GlobalHotKeyManager::new().ok();
    let _registered_hotkey = _hotkey_manager.as_ref().and_then(|manager| {
        config
            .hotkey
            .parse::<HotKey>()
            .ok()
            .and_then(|hk| match manager.register(hk) {
                Ok(()) => {
                    tracing::info!("Global hotkey registered: {}", config.hotkey);
                    Some(hk)
                }
                Err(e) => {
                    tracing::warn!("Failed to register hotkey '{}': {e}", config.hotkey);
                    None
                }
            })
    });

    let mut app = App {
        tray_icon: None, // Created below, once NSApp is ready
        menu_actions: std::collections::HashMap::new(),
        last_group: types::PullRequestGroup::default(),
        rx,
        auto_launch,
        auto_start_enabled,
        last_updated: None,
        is_stale: false,
        should_exit: false,
    };

    ns_app.finishLaunching();
    app.create_tray_icon();

    // Drive AppKit directly. `nextEventMatchingMask` blocks until an event
    // arrives or the timeout expires, so an idle app sleeps, and work queued by
    // the polling thread is picked up within one timeout.
    while !app.should_exit {
        app.drain();

        let timeout = NSDate::dateWithTimeIntervalSinceNow(PUMP_TIMEOUT_SECS);
        let event = unsafe {
            ns_app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                Some(&timeout),
                NSDefaultRunLoopMode,
                true,
            )
        };
        if let Some(event) = event {
            ns_app.sendEvent(&event);
        }
    }
}
