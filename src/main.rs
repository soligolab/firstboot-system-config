use std::process::Command;
use std::time::Duration;

use slint::{Timer, TimerMode};

slint::include_modules!();

const TIMEZONES: [&str; 5] = [
    "UTC",
    "Europe/Rome",
    "Europe/London",
    "America/New_York",
    "Asia/Tokyo",
];

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    app.set_status_message("Pronto.".into());

    refresh_clock(&app);
    wire_callbacks(&app);

    let weak = app.as_weak();
    let clock_timer = Timer::default();
    clock_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
        if let Some(app) = weak.upgrade() {
            refresh_clock(&app);
        }
    });

    app.run()
}

fn wire_callbacks(app: &AppWindow) {
    app.on_password_feedback(|role, password| password_feedback(&role, &password).into());

    {
        let weak = app.as_weak();
        app.on_apply_configuration(move || {
            if let Some(app) = weak.upgrade() {
                let payload = build_user_payload(&app);
                let result = apply_on_host(payload);
                app.set_status_message(result.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_backup_recovery(move || {
            if let Some(app) = weak.upgrade() {
                let result = run_host(
                    "sh",
                    &[
                        "-c",
                        "echo '[backup] requested' >> /tmp/firstboot-actions.log && uname -a",
                    ],
                );
                app.set_status_message(result.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_factory_reset(move || {
            if let Some(app) = weak.upgrade() {
                let result = run_host(
                    "sh",
                    &[
                        "-c",
                        "echo '[factory-reset] requested' >> /tmp/firstboot-actions.log && date",
                    ],
                );
                app.set_status_message(result.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_open_time_settings(move || {
            if let Some(app) = weak.upgrade() {
                refresh_clock(&app);
                app.set_show_time_settings(true);
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_close_time_settings(move || {
            if let Some(app) = weak.upgrade() {
                app.set_show_time_settings(false);
                refresh_clock(&app);
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_save_time_settings(move || {
            if let Some(app) = weak.upgrade() {
                let timezone = timezone_from_index(app.get_current_timezone_idx()).to_string();
                let result =
                    save_time_settings(&app.get_current_date(), &app.get_current_time(), &timezone);
                app.set_current_timezone_label(timezone.into());
                app.set_status_message(result.into());
                app.set_show_time_settings(false);
                refresh_clock(&app);
            }
        });
    }
}

fn refresh_clock(app: &AppWindow) {
    let date = cmd("date", &["+%Y-%m-%d"]).unwrap_or_else(|| "----/--/--".to_string());
    let time = cmd("date", &["+%H:%M:%S"]).unwrap_or_else(|| "--:--:--".to_string());
    let timezone = current_timezone();
    let timezone_idx = timezone_index(&timezone);

    app.set_current_date(date.into());
    app.set_current_time(time.into());
    app.set_current_timezone_idx(timezone_idx);
    app.set_current_timezone_label(timezone.into());
}

fn build_user_payload(app: &AppWindow) -> String {
    let users = [
        (
            "admin",
            app.get_admin_username(),
            app.get_admin_full_name(),
            app.get_admin_password(),
            app.get_admin_permission_idx(),
        ),
        (
            "installer",
            app.get_installer_username(),
            app.get_installer_full_name(),
            app.get_installer_password(),
            app.get_installer_permission_idx(),
        ),
        (
            "viewer",
            app.get_viewer_username(),
            app.get_viewer_full_name(),
            app.get_viewer_password(),
            app.get_viewer_permission_idx(),
        ),
    ];

    users
        .into_iter()
        .map(|(role, username, full_name, password, permission_idx)| {
            format!("{role}|{username}|{full_name}|{password}|{permission_idx}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn password_feedback(role: &str, password: &str) -> String {
    if password.is_empty() {
        return format!("{role}: password non impostata.");
    }

    let mut score = 0;
    if password.len() >= 12 {
        score += 1;
    }
    if password.chars().any(|c| c.is_ascii_lowercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        score += 1;
    }

    let message = match score {
        0..=2 => "password debole",
        3..=4 => "password discreta",
        _ => "password forte",
    };

    format!("{role}: {message} (valutazione informativa).")
}

fn save_time_settings(date: &str, time: &str, timezone: &str) -> String {
    let datetime = format!("{date} {time}");
    let tz = run_host("timedatectl", &["set-timezone", timezone]);
    let dt = run_host("timedatectl", &["set-time", &datetime]);
    format!("timezone:\n{tz}\n\ntime:\n{dt}")
}

fn apply_on_host(payload: String) -> String {
    let escaped = payload.replace('"', "\\\"");
    run_host(
        "sh",
        &[
            "-c",
            &format!(
                "echo \"{escaped}\" >> /tmp/firstboot-user-config.log && echo 'configuration request executed on host'"
            ),
        ],
    )
}

fn run_host(program: &str, args: &[&str]) -> String {
    match Command::new(program).args(args).output() {
        Ok(out) => {
            let output = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            if out.status.success() {
                format!("OK: {} {}\n{}", program, args.join(" "), output.trim())
            } else {
                format!("ERROR: {} {}\n{}", program, args.join(" "), output.trim())
            }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}

fn cmd(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn current_timezone() -> String {
    cmd("timedatectl", &["show", "--property=Timezone", "--value"])
        .filter(|value| !value.is_empty())
        .or_else(|| cmd("date", &["+%Z"]))
        .unwrap_or_else(|| TIMEZONES[0].to_string())
}

fn timezone_index(label: &str) -> i32 {
    TIMEZONES
        .iter()
        .position(|candidate| *candidate == label)
        .map(|idx| idx as i32)
        .unwrap_or(0)
}

fn timezone_from_index(index: i32) -> &'static str {
    TIMEZONES
        .get(index.max(0) as usize)
        .copied()
        .unwrap_or(TIMEZONES[0])
}
