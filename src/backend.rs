use std::process::Command;

use crate::models::{ApplyConfigurationRequest, SaveTimeSettingsRequest, TimeState, UserConfig};

#[derive(Clone, Debug, Default)]
pub struct NativeHostService;

impl NativeHostService {
    pub fn current_time(&self) -> TimeState {
        TimeState {
            date: cmd("date", &["+%Y-%m-%d"]).unwrap_or_else(|| "----/--/--".to_string()),
            time: cmd("date", &["+%H:%M:%S"]).unwrap_or_else(|| "--:--:--".to_string()),
            timezone: current_timezone(),
        }
    }

    pub fn apply_configuration(&self, request: ApplyConfigurationRequest) -> String {
        let payload = request
            .users
            .into_iter()
            .map(user_to_payload)
            .collect::<Vec<_>>()
            .join("\n");

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

    pub fn backup_recovery(&self) -> String {
        run_host(
            "sh",
            &[
                "-c",
                "echo '[backup] requested' >> /tmp/firstboot-actions.log && uname -a",
            ],
        )
    }

    pub fn factory_reset(&self) -> String {
        run_host(
            "sh",
            &[
                "-c",
                "echo '[factory-reset] requested' >> /tmp/firstboot-actions.log && date",
            ],
        )
    }

    pub fn save_time_settings(&self, request: SaveTimeSettingsRequest) -> String {
        let datetime = format!("{} {}", request.date, request.time);
        let tz = run_host("timedatectl", &["set-timezone", &request.timezone]);
        let dt = run_host("timedatectl", &["set-time", &datetime]);
        format!("timezone:\n{tz}\n\ntime:\n{dt}")
    }
}

fn user_to_payload(user: UserConfig) -> String {
    user.to_line()
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
        .or_else(|| cmd("cat", &["/etc/timezone"]))
        .unwrap_or_else(|| "UTC".to_string())
}
