use slint::{ComponentHandle, SharedString, Timer, TimerMode};
use std::{process::Command, time::Duration};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let (date, time) = system_date_time();
    ui.set_current_date(date.into());
    ui.set_current_time(time.into());
    ui.set_current_timezone_label(timezone_name(ui.get_current_timezone_idx()).into());

    let ui_weak_timer = ui.as_weak();
    let mut clock_timer = Timer::default();
    clock_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
        if let Some(ui) = ui_weak_timer.upgrade() {
            let (date, time) = system_date_time();
            ui.set_current_date(date.into());
            ui.set_current_time(time.into());
        }
    });

    ui.on_password_feedback(|role, password| {
        let score = password_complexity_score(&password);
        let level = match score {
            0..=2 => "Debole",
            3..=4 => "Media",
            _ => "Forte",
        };

        let advisory = if score < 3 {
            "Suggerimento: aggiungi maiuscole, minuscole, numeri e simboli"
        } else {
            "Complessità buona (non vincolante)"
        };

        SharedString::from(format!("{} • {}: {}", role, level, advisory))
    });

    let ui_weak = ui.as_weak();
    ui.on_apply_configuration(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let admin = (
                ui.get_admin_username(),
                ui.get_admin_full_name(),
                ui.get_admin_password(),
                permission_name(ui.get_admin_permission_idx()),
            );
            let installer = (
                ui.get_installer_username(),
                ui.get_installer_full_name(),
                ui.get_installer_password(),
                permission_name(ui.get_installer_permission_idx()),
            );
            let viewer = (
                ui.get_viewer_username(),
                ui.get_viewer_full_name(),
                ui.get_viewer_password(),
                permission_name(ui.get_viewer_permission_idx()),
            );

            // Aggancio placeholder per la logica reale di provisioning utenti.
            println!(
                "[HOOK] apply_configuration -> admin={:?}, installer={:?}, viewer={:?}",
                admin, installer, viewer
            );
        }
    });

    ui.on_backup_recovery(|| {
        // Aggancio placeholder: integrare il meccanismo di ripristino da backup.
        println!("[HOOK] backup_recovery");
    });

    ui.on_factory_reset(|| {
        // Aggancio placeholder: integrare la cancellazione impostazioni e reset fabbrica.
        println!("[HOOK] factory_reset");
    });

    let ui_weak = ui.as_weak();
    ui.on_open_time_settings(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_time_settings(true);
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_close_time_settings(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_time_settings(false);
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_save_time_settings(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let timezone = timezone_name(ui.get_current_timezone_idx());
            ui.set_current_timezone_label(timezone.into());
            ui.set_show_time_settings(false);

            // Aggancio placeholder: integrare la configurazione data/ora/timezone a livello di OS.
            println!(
                "[HOOK] save_time_settings -> date={}, time={}, timezone={}",
                ui.get_current_date(),
                ui.get_current_time(),
                timezone
            );
        }
    });

    ui.run()
}

fn system_date_time() -> (String, String) {
    let output = Command::new("date")
        .arg("+%d/%m/%Y|%H:%M:%S")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string());

    if let Some(line) = output {
        let mut parts = line.split('|');
        if let (Some(date), Some(time)) = (parts.next(), parts.next()) {
            return (date.to_string(), time.to_string());
        }
    }

    ("--/--/----".to_string(), "--:--:--".to_string())
}

fn timezone_name(index: i32) -> &'static str {
    match index {
        0 => "UTC",
        1 => "Europe/Rome",
        2 => "Europe/London",
        3 => "America/New_York",
        4 => "Asia/Tokyo",
        _ => "UTC",
    }
}

fn permission_name(index: i32) -> &'static str {
    match index {
        0 => "Amministratore completo",
        1 => "Rete e ora di sistema",
        2 => "Sola visualizzazione",
        _ => "Profilo personalizzato",
    }
}

fn password_complexity_score(password: &str) -> usize {
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

    score
}
