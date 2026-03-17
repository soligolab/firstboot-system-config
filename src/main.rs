use slint::{ComponentHandle, SharedString};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

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

    ui.run()
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
