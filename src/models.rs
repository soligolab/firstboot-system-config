/// Stato orario esposto dal backend e consumato dalla GUI.
#[derive(Clone, Debug)]
pub struct TimeState {
    pub date: String,
    pub time: String,
    pub timezone: String,
}

impl TimeState {
    /// Serializza il modello nel formato testuale semplice usato dagli endpoint HTTP.
    pub fn to_body(&self) -> String {
        format!(
            "date={}\ntime={}\ntimezone={}\n",
            self.date, self.time, self.timezone
        )
    }

    /// Deserializza il body testuale ritornato dagli endpoint `/api/time`.
    pub fn from_body(body: &str) -> Result<Self, String> {
        let mut date = None;
        let mut time = None;
        let mut timezone = None;

        for line in body.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "date" => date = Some(value.to_string()),
                    "time" => time = Some(value.to_string()),
                    "timezone" => timezone = Some(value.to_string()),
                    // Chiavi non note vengono ignorate per mantenere il parser tollerante
                    // a future estensioni backward-compatible.
                    _ => {}
                }
            }
        }

        Ok(Self {
            date: date.ok_or_else(|| "missing date".to_string())?,
            time: time.ok_or_else(|| "missing time".to_string())?,
            timezone: timezone.ok_or_else(|| "missing timezone".to_string())?,
        })
    }
}

/// Payload inviato dalla GUI quando l'operatore salva data/ora/timezone.
#[derive(Clone, Debug)]
pub struct SaveTimeSettingsRequest {
    pub date: String,
    pub time: String,
    pub timezone: String,
}

impl SaveTimeSettingsRequest {
    /// Riusa lo stesso formato `chiave=valore` adottato da `TimeState` per ridurre la
    /// complessità del protocollo HTTP locale.
    pub fn to_body(&self) -> String {
        format!(
            "date={}\ntime={}\ntimezone={}\n",
            self.date, self.time, self.timezone
        )
    }

    /// Il payload di salvataggio ha la stessa forma dello stato orario letto dal
    /// backend, quindi può essere ricostruito partendo da `TimeState`.
    pub fn from_body(body: &str) -> Result<Self, String> {
        let state = TimeState::from_body(body)?;
        Ok(Self {
            date: state.date,
            time: state.time,
            timezone: state.timezone,
        })
    }
}

/// Configurazione di un singolo profilo utente mostrato nella GUI.
#[derive(Clone, Debug)]
pub struct UserConfig {
    pub role: String,
    pub username: String,
    pub full_name: String,
    pub password: String,
    pub permission_idx: i32,
}

impl UserConfig {
    /// Serializza un profilo utente come record pipe-separated.
    pub fn to_line(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.role, self.username, self.full_name, self.password, self.permission_idx
        )
    }

    /// Parser dell'equivalente formato pipe-separated usato dall'API locale.
    pub fn from_line(line: &str) -> Result<Self, String> {
        let parts: Vec<_> = line.split('|').collect();
        if parts.len() != 5 {
            return Err(format!("invalid user line: {line}"));
        }

        Ok(Self {
            role: parts[0].to_string(),
            username: parts[1].to_string(),
            full_name: parts[2].to_string(),
            password: parts[3].to_string(),
            permission_idx: parts[4]
                .parse()
                .map_err(|_| format!("invalid permission index in: {line}"))?,
        })
    }
}

/// Richiesta completa di configurazione utenti inviata dal frontend al backend.
#[derive(Clone, Debug)]
pub struct ApplyConfigurationRequest {
    pub users: Vec<UserConfig>,
}

impl ApplyConfigurationRequest {
    /// Il body contiene una riga per utente, nell'ordine con cui la GUI presenta i
    /// profili all'operatore.
    pub fn to_body(&self) -> String {
        self.users
            .iter()
            .map(UserConfig::to_line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Converte il body testuale in una lista di utenti ignorando righe vuote, così
    /// la serializzazione resta leggibile anche con newline finali.
    pub fn from_body(body: &str) -> Result<Self, String> {
        let users = body
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(UserConfig::from_line)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { users })
    }
}
