mod api;
mod backend;
mod localization;
mod models;
mod web;

use std::thread;
use std::time::Duration;

use api::ApiClient;
use backend::NativeHostService;
use localization::{LanguagePack, LocalizationCatalog};
use models::{ApplyConfigurationRequest, SaveTimeSettingsRequest, UserConfig};
use slint::{ModelRc, SharedString, Timer, TimerMode, VecModel};

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

    let localization_catalog =
        LocalizationCatalog::load_embedded().expect("failed to load language catalog");
    let default_language_idx = localization_catalog.default_index();
    let default_language = localization_catalog.language(default_language_idx);

    let api_client = ApiClient::new(format!("http://{api_addr}"));
    let app = AppWindow::new()?;
    app.set_language_options(localization_catalog.language_names_model());
    app.set_permission_options(permission_options_model(default_language));
    app.set_timezone_options(timezone_options_model());
    app.set_current_language_idx(default_language_idx as i32);
    apply_language(&app, default_language);
    app.set_status_message(default_language.text("status_ready"));

    // Sincronizza subito data/ora/timezone visibili nella toolbar superiore.
    refresh_clock(&app, &api_client, default_language);

    // Collega tutti i callback della UI alle relative chiamate HTTP.
    wire_callbacks(&app, api_client.clone(), localization_catalog.clone());

    // Aggiorna l'orologio una volta al secondo per mantenere la UI allineata allo
    // stato del backend/host.
    let weak = app.as_weak();
    let clock_timer = Timer::default();
    let localization_catalog_for_timer = localization_catalog.clone();
    clock_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
        if let Some(app) = weak.upgrade() {
            let language =
                localization_catalog_for_timer.language(app.get_current_language_idx() as usize);
            refresh_clock(&app, &api_client, language);
        }
    });

    app.run()
}

/// Associa i callback dichiarati in `ui/app.slint` alle operazioni applicative.
///
/// Ogni azione dell'utente viene tradotta in una richiesta HTTP verso il backend
/// locale. L'uso di `Weak<AppWindow>` evita di trattenere riferimenti forti alla UI
/// dentro le closure dei callback e del timer.
fn wire_callbacks(
    app: &AppWindow,
    api_client: ApiClient,
    localization_catalog: LocalizationCatalog,
) {
    // Feedback client-side della robustezza password: è puramente informativo e non
    // sostituisce eventuali validazioni lato backend/host.
    {
        let weak = app.as_weak();
        let localization_catalog = localization_catalog.clone();
        app.on_password_feedback(move |role, password| {
            weak.upgrade()
                .map(|app| {
                    let language =
                        localization_catalog.language(app.get_current_language_idx() as usize);
                    password_feedback(language, &role, &password).into()
                })
                .unwrap_or_else(|| SharedString::from(""))
        });
    }

    {
        let weak = app.as_weak();
        let localization_catalog = localization_catalog.clone();
        app.on_language_changed(move |index| {
            if let Some(app) = weak.upgrade() {
                let selected_idx =
                    (index.max(0) as usize).min(localization_catalog.len().saturating_sub(1));
                let language = localization_catalog.language(selected_idx);
                app.set_current_language_idx(selected_idx as i32);
                app.set_permission_options(permission_options_model(language));
                apply_language(&app, language);
            }
        });
    }

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
        let localization_catalog = localization_catalog.clone();
        app.on_open_time_settings(move || {
            if let Some(app) = weak.upgrade() {
                // Prima di mostrare il dialog, ricarica lo stato corrente per evitare
                // che l'utente modifichi dati già obsoleti.
                let language =
                    localization_catalog.language(app.get_current_language_idx() as usize);
                refresh_clock(&app, &api_client, language);
                app.set_show_time_settings(true);
            }
        });
    }

    {
        let weak = app.as_weak();
        let api_client = api_client.clone();
        let localization_catalog = localization_catalog.clone();
        app.on_close_time_settings(move || {
            if let Some(app) = weak.upgrade() {
                app.set_show_time_settings(false);

                // Ripristina i valori letti dal backend, nel caso l'utente abbia
                // digitato modifiche non salvate nel popup.
                let language =
                    localization_catalog.language(app.get_current_language_idx() as usize);
                refresh_clock(&app, &api_client, language);
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
            }
        });
    }
}

fn apply_language(app: &AppWindow, language: &LanguagePack) {
    app.set_window_title(language.text("window_title"));
    app.set_language_label(language.text("language_label"));
    app.set_current_language_flag(language.flag_image());
    app.set_current_language_flag_emoji(language.flag_emoji.clone().into());

    app.set_clock_date_label(language.text("clock_date"));
    app.set_clock_time_label(language.text("clock_time"));
    app.set_clock_timezone_label(language.text("clock_timezone"));
    app.set_configure_time_label(language.text("configure_time"));

    app.set_main_heading(language.text("main_heading"));
    app.set_suggestion_text(language.text("suggestion_text"));

    app.set_admin_title(language.text("admin_title"));
    app.set_installer_title(language.text("installer_title"));
    app.set_viewer_title(language.text("viewer_title"));
    app.set_admin_description(language.text("admin_description"));
    app.set_installer_description(language.text("installer_description"));
    app.set_viewer_description(language.text("viewer_description"));

    app.set_username_label(language.text("username_label"));
    app.set_full_name_label(language.text("full_name_label"));
    app.set_password_label(language.text("password_label"));
    app.set_permissions_label(language.text("permissions_label"));

    app.set_apply_configuration_label(language.text("apply_configuration"));
    app.set_backup_recovery_label(language.text("backup_recovery"));
    app.set_factory_reset_label(language.text("factory_reset"));
    app.set_note_password(language.text("note_password"));

    app.set_time_settings_heading(language.text("time_modal_title"));
    app.set_current_date_input_label(language.text("current_date_label"));
    app.set_current_time_input_label(language.text("current_time_label"));
    app.set_time_format_hint(language.text("format_hint"));
    app.set_cancel_label(language.text("cancel"));
    app.set_save_label(language.text("save"));

    app.set_admin_password_feedback(
        password_feedback(
            language,
            &app.get_admin_title().to_string(),
            &app.get_admin_password().to_string(),
        )
        .into(),
    );
    app.set_installer_password_feedback(
        password_feedback(
            language,
            &app.get_installer_title().to_string(),
            &app.get_installer_password().to_string(),
        )
        .into(),
    );
    app.set_viewer_password_feedback(
        password_feedback(
            language,
            &app.get_viewer_title().to_string(),
            &app.get_viewer_password().to_string(),
        )
        .into(),
    );
}

fn permission_options_model(language: &LanguagePack) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(vec![
        language.text("permission_full"),
        language.text("permission_network_time"),
        language.text("permission_readonly"),
    ]))
}

fn timezone_options_model() -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(vec![
        SharedString::from("UTC"),
        SharedString::from("Europe/Rome"),
        SharedString::from("Europe/London"),
        SharedString::from("America/New_York"),
        SharedString::from("Asia/Tokyo"),
    ]))
}

/// Legge dal backend locale la situazione corrente di data, ora e timezone e la
/// riflette nella UI.
fn refresh_clock(app: &AppWindow, api_client: &ApiClient, language: &LanguagePack) {
    match api_client.get_time() {
        Ok(state) => {
            app.set_current_date(state.date.into());
            app.set_current_time(state.time.into());
            app.set_current_timezone_idx(timezone_index(&state.timezone));
            app.set_current_timezone_label(state.timezone.into());
        }
        Err(err) => {
            app.set_status_message(
                format!(
                    "{}: {err}",
                    language.text_string("status_backend_unreachable")
                )
                .into(),
            );
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
fn password_feedback(language: &LanguagePack, role: &str, password: &str) -> String {
    if password.is_empty() {
        return format!("{role}: {}.", language.text_string("password_not_set"));
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
        0..=2 => language.text_string("password_weak"),
        3..=4 => language.text_string("password_fair"),
        _ => language.text_string("password_strong"),
    };

    format!(
        "{role}: {message} ({}).",
        language.text_string("password_feedback_suffix")
    )
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
