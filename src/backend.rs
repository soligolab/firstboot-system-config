use std::process::Command;

use crate::models::{ApplyConfigurationRequest, SaveTimeSettingsRequest, TimeState, UserConfig};

/// Implementazione minimale del servizio host.
///
/// In questo repository il backend esegue comandi locali e scrive log di esempio in
/// `/tmp`, così la GUI può essere provata senza integrare subito il vero stack di
/// provisioning del dispositivo.
#[derive(Clone, Debug, Default)]
pub struct NativeHostService;

impl NativeHostService {
    /// Legge data, ora e timezone dal sistema operativo sottostante.
    pub fn current_time(&self) -> TimeState {
        TimeState {
            date: cmd("date", &["+%Y-%m-%d"]).unwrap_or_else(|| "----/--/--".to_string()),
            time: cmd("date", &["+%H:%M:%S"]).unwrap_or_else(|| "--:--:--".to_string()),
            timezone: current_timezone(),
        }
    }

    /// Applica la configurazione utenti.
    ///
    /// L'implementazione corrente serializza i profili richiesti e li appende a un
    /// file di log temporaneo. È un comportamento volutamente sicuro/dimostrativo,
    /// semplice da sostituire con chiamate reali a user-management o provisioning.
    pub fn apply_configuration(&self, request: ApplyConfigurationRequest) -> String {
        let payload = request
            .users
            .into_iter()
            .map(user_to_payload)
            .collect::<Vec<_>>()
            .join("\n");

        // Escape minimo per incorporare il payload dentro una shell command demo.
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

    /// Simula l'avvio di una procedura di backup recovery.
    pub fn backup_recovery(&self) -> String {
        run_host(
            "sh",
            &[
                "-c",
                "echo '[backup] requested' >> /tmp/firstboot-actions.log && uname -a",
            ],
        )
    }

    /// Simula l'avvio di una procedura di factory reset.
    pub fn factory_reset(&self) -> String {
        run_host(
            "sh",
            &[
                "-c",
                "echo '[factory-reset] requested' >> /tmp/firstboot-actions.log && date",
            ],
        )
    }

    /// Aggiorna timezone e data/ora di sistema usando `timedatectl`.
    ///
    /// Il risultato concatena gli output delle due invocazioni per mostrare nella UI
    /// sia l'esito del cambio timezone sia quello del cambio data/ora.
    pub fn save_time_settings(&self, request: SaveTimeSettingsRequest) -> String {
        let datetime = format!("{} {}", request.date, request.time);
        let tz = run_host("timedatectl", &["set-timezone", &request.timezone]);
        let dt = run_host("timedatectl", &["set-time", &datetime]);
        format!("timezone:\n{tz}\n\ntime:\n{dt}")
    }
}

/// Converte la configurazione utente in una riga compatta adatta al trasporto HTTP
/// e ai log temporanei del backend demo.
fn user_to_payload(user: UserConfig) -> String {
    user.to_line()
}

/// Esegue un comando locale e restituisce un messaggio pronto per essere mostrato in
/// GUI o restituito dall'API.
///
/// L'output combina `stdout` e `stderr` per non perdere informazioni utili durante il
/// debug delle operazioni host.
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

/// Helper per eseguire comandi da cui interessa solo `stdout` se il processo ha
/// successo.
fn cmd(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Determina la timezone di sistema tentando prima `timedatectl` e poi il fallback
/// classico `/etc/timezone`.
fn current_timezone() -> String {
    cmd("timedatectl", &["show", "--property=Timezone", "--value"])
        .or_else(|| cmd("cat", &["/etc/timezone"]))
        .unwrap_or_else(|| "UTC".to_string())
}
