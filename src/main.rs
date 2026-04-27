#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod config;
mod demo;
mod github;
mod logging;
mod theme;
mod tray;
mod types;

use clap::Parser;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};
use muda::MenuEvent;
use std::sync::mpsc;
use std::time::Duration;
use tray_icon::TrayIconBuilder;
use winit::application::ApplicationHandler;
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopBuilder};
use winit::window::WindowId;

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
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            // Create tray icon here — on macOS it must be created after the event loop starts
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
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Process menu events
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if let Some(action) = self.menu_actions.get(&event.id).cloned() {
                match action {
                    tray::MenuAction::OpenUrl(url) => {
                        let _ = open::that(&url);
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
                        event_loop.exit();
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
}

impl App {
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
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        unsafe {
            let time = libc::time(std::ptr::null_mut());
            let mut tm = MaybeUninit::uninit();
            libc::localtime_r(&time, tm.as_mut_ptr());
            let tm = tm.assume_init();
            format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
        }
    }

    #[cfg(windows)]
    {
        use std::mem::MaybeUninit;

        #[repr(C)]
        struct SystemTime {
            w_year: u16,
            w_month: u16,
            w_day_of_week: u16,
            w_day: u16,
            w_hour: u16,
            w_minute: u16,
            w_second: u16,
            w_milliseconds: u16,
        }

        unsafe extern "system" {
            fn GetLocalTime(lp_system_time: *mut SystemTime);
        }

        unsafe {
            let mut st = MaybeUninit::<SystemTime>::uninit();
            GetLocalTime(st.as_mut_ptr());
            let st = st.assume_init();
            format!("{:02}:{:02}:{:02}", st.w_hour, st.w_minute, st.w_second)
        }
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
#[cfg(target_os = "macos")]
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
    #[cfg(target_os = "macos")]
    reexec_via_platform_binary();

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

    // Build event loop — on macOS, set Accessory policy to hide from Dock
    let mut builder = EventLoopBuilder::default();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder.with_activation_policy(ActivationPolicy::Accessory);
    }
    let event_loop = builder.build().expect("Failed to create event loop");

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
        tray_icon: None, // Created in new_events after event loop starts
        menu_actions: std::collections::HashMap::new(),
        last_group: types::PullRequestGroup::default(),
        rx,
        auto_launch,
        auto_start_enabled,
        last_updated: None,
        is_stale: false,
    };

    event_loop.run_app(&mut app).expect("Event loop failed");
}
