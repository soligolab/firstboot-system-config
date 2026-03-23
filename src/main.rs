mod api;
mod backend;
mod models;
mod web;

use std::thread;
use std::time::Duration;

use api::ApiClient;
use backend::NativeHostService;
use models::{ApplyConfigurationRequest, SaveTimeSettingsRequest, UserConfig};
use slint::{Timer, TimerMode};

slint::include_modules!();

/// Indirizzo di default del backend locale usato sia dalla GUI sia dalla modalità
/// `server`. Può essere sovrascritto con la variabile d'ambiente
/// `FIRSTBOOT_API_ADDR`.
const DEFAULT_API_ADDR: &str = "127.0.0.1:7878";

/// Punto d'ingresso dell'applicazione.
///
/// Il binario supporta due modalità operative:
/// - `cargo run`: avvia GUI + backend HTTP locale in background;
/// - `cargo run -- server`: avvia solo il backend HTTP, utile per test o integrazione.
fn main() -> Result<(), slint::PlatformError> {
    let api_addr =
        std::env::var("FIRSTBOOT_API_ADDR").unwrap_or_else(|_| DEFAULT_API_ADDR.to_string());

    // Modalità server headless: nessuna GUI, solo API HTTP locale.
    if std::env::args().nth(1).as_deref() == Some("server") {
        api::run_server(api_addr, NativeHostService::default());
        return Ok(());
    }

    // In modalità desktop la GUI dipende sempre dall'API locale, quindi il backend
    // viene avviato in un thread separato prima di costruire la finestra.
    api::spawn_server(api_addr.clone(), NativeHostService::default());

    // Piccola attesa per ridurre il rischio che la GUI tenti la prima richiesta
    // HTTP prima che il listener TCP sia effettivamente in ascolto.
    thread::sleep(Duration::from_millis(150));

    let api_client = ApiClient::new(format!("http://{api_addr}"));
    let app = AppWindow::new()?;
    app.set_status_message("Pronto. Backend API locale collegato.".into());

    // Sincronizza subito data/ora/timezone visibili nella toolbar superiore.
    refresh_clock(&app, &api_client);

    // Collega tutti i callback della UI alle relative chiamate HTTP.
    wire_callbacks(&app, api_client.clone());

    // Aggiorna l'orologio una volta al secondo per mantenere la UI allineata allo
    // stato del backend/host.
    let weak = app.as_weak();
    let clock_timer = Timer::default();
    clock_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
        if let Some(app) = weak.upgrade() {
            refresh_clock(&app, &api_client);
        }
    });

    app.run()
}

/// Associa i callback dichiarati in `ui/app.slint` alle operazioni applicative.
///
/// Ogni azione dell'utente viene tradotta in una richiesta HTTP verso il backend
/// locale. L'uso di `Weak<AppWindow>` evita di trattenere riferimenti forti alla UI
/// dentro le closure dei callback e del timer.
fn wire_callbacks(app: &AppWindow, api_client: ApiClient) {
    // Feedback client-side della robustezza password: è puramente informativo e non
    // sostituisce eventuali validazioni lato backend/host.
    app.on_password_feedback(|role, password| password_feedback(&role, &password).into());

    {
        let weak = app.as_weak();
        let api_client = api_client.clone();
        app.on_apply_configuration(move || {
            if let Some(app) = weak.upgrade() {
                // Costruisce il payload leggendo i campi correnti della finestra.
                let request = ApplyConfigurationRequest {
                    users: build_user_configs(&app),
                };
                let result = api_client.apply_configuration(&request);
                app.set_status_message(result.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        let api_client = api_client.clone();
        app.on_backup_recovery(move || {
            if let Some(app) = weak.upgrade() {
                let result = api_client.backup_recovery();
                app.set_status_message(result.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        let api_client = api_client.clone();
        app.on_factory_reset(move || {
            if let Some(app) = weak.upgrade() {
                let result = api_client.factory_reset();
                app.set_status_message(result.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        let api_client = api_client.clone();
        app.on_open_time_settings(move || {
            if let Some(app) = weak.upgrade() {
                // Prima di mostrare il dialog, ricarica lo stato corrente per evitare
                // che l'utente modifichi dati già obsoleti.
                refresh_clock(&app, &api_client);
                app.set_show_time_settings(true);
            }
        });
    }

    {
        let weak = app.as_weak();
        let api_client = api_client.clone();
        app.on_close_time_settings(move || {
            if let Some(app) = weak.upgrade() {
                app.set_show_time_settings(false);

                // Ripristina i valori letti dal backend, nel caso l'utente abbia
                // digitato modifiche non salvate nel popup.
                refresh_clock(&app, &api_client);
            }
        });
    }

    {
        let weak = app.as_weak();
        let api_client = api_client.clone();
        app.on_save_time_settings(move || {
            if let Some(app) = weak.upgrade() {
                let timezone = timezone_from_index(app.get_current_timezone_idx()).to_string();
                let request = SaveTimeSettingsRequest {
                    date: app.get_current_date().to_string(),
                    time: app.get_current_time().to_string(),
                    timezone: timezone.clone(),
                };
                let result = api_client.save_time_settings(&request);

                // La label mostrata nella toolbar usa la stringa effettiva inviata al
                // backend, così UI e payload restano allineati.
                app.set_current_timezone_label(timezone.into());
                app.set_status_message(result.into());
                app.set_show_time_settings(false);
                refresh_clock(&app, &api_client);
            }
        });
    }
}

/// Legge dal backend locale la situazione corrente di data, ora e timezone e la
/// riflette nella UI.
fn refresh_clock(app: &AppWindow, api_client: &ApiClient) {
    match api_client.get_time() {
        Ok(state) => {
            app.set_current_date(state.date.into());
            app.set_current_time(state.time.into());
            app.set_current_timezone_idx(timezone_index(&state.timezone));
            app.set_current_timezone_label(state.timezone.into());
        }
        Err(err) => {
            app.set_status_message(format!("Backend API non raggiungibile: {err}").into());
        }
    }
}

/// Raccoglie dalla UI i tre profili utente esposti all'operatore.
///
/// L'ordine viene mantenuto stabile per avere payload prevedibili lato backend e
/// nei log di test/simulazione.
fn build_user_configs(app: &AppWindow) -> Vec<UserConfig> {
    vec![
        UserConfig {
            role: "admin".into(),
            username: app.get_admin_username().to_string(),
            full_name: app.get_admin_full_name().to_string(),
            password: app.get_admin_password().to_string(),
            permission_idx: app.get_admin_permission_idx(),
        },
        UserConfig {
            role: "installer".into(),
            username: app.get_installer_username().to_string(),
            full_name: app.get_installer_full_name().to_string(),
            password: app.get_installer_password().to_string(),
            permission_idx: app.get_installer_permission_idx(),
        },
        UserConfig {
            role: "viewer".into(),
            username: app.get_viewer_username().to_string(),
            full_name: app.get_viewer_full_name().to_string(),
            password: app.get_viewer_password().to_string(),
            permission_idx: app.get_viewer_permission_idx(),
        },
    ]
}

/// Restituisce una valutazione qualitativa della password.
///
/// Il punteggio è volutamente semplice e pensato per offrire un'indicazione rapida
/// all'operatore, non per implementare una policy di sicurezza completa.
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

/// Mappa l'indice usato dal `ComboBox` Slint al nome canonicale della timezone.
fn timezone_from_index(index: i32) -> &'static str {
    match index {
        0 => "UTC",
        1 => "Europe/Rome",
        2 => "Europe/London",
        3 => "America/New_York",
        4 => "Asia/Tokyo",
        _ => "UTC",
    }
}

/// Converte il nome della timezone ricevuto dal backend nell'indice atteso dalla UI.
///
/// I valori non riconosciuti ricadono su `UTC` per garantire che il `ComboBox`
/// resti sempre in uno stato valido.
fn timezone_index(timezone: &str) -> i32 {
    match timezone {
        "UTC" => 0,
        "Europe/Rome" => 1,
        "Europe/London" => 2,
        "America/New_York" => 3,
        "Asia/Tokyo" => 4,
        _ => 0,
    }
}
