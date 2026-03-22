#[derive(Clone, Debug)]
pub struct TimeState {
    pub date: String,
    pub time: String,
    pub timezone: String,
}

impl TimeState {
    pub fn to_body(&self) -> String {
        format!(
            "date={}\ntime={}\ntimezone={}\n",
            self.date, self.time, self.timezone
        )
    }

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

#[derive(Clone, Debug)]
pub struct SaveTimeSettingsRequest {
    pub date: String,
    pub time: String,
    pub timezone: String,
}

impl SaveTimeSettingsRequest {
    pub fn to_body(&self) -> String {
        format!(
            "date={}\ntime={}\ntimezone={}\n",
            self.date, self.time, self.timezone
        )
    }

    pub fn from_body(body: &str) -> Result<Self, String> {
        let state = TimeState::from_body(body)?;
        Ok(Self {
            date: state.date,
            time: state.time,
            timezone: state.timezone,
        })
    }
}

#[derive(Clone, Debug)]
pub struct UserConfig {
    pub role: String,
    pub username: String,
    pub full_name: String,
    pub password: String,
    pub permission_idx: i32,
}

impl UserConfig {
    pub fn to_line(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.role, self.username, self.full_name, self.password, self.permission_idx
        )
    }

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

#[derive(Clone, Debug)]
pub struct ApplyConfigurationRequest {
    pub users: Vec<UserConfig>,
}

impl ApplyConfigurationRequest {
    pub fn to_body(&self) -> String {
        self.users
            .iter()
            .map(UserConfig::to_line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn from_body(body: &str) -> Result<Self, String> {
        let users = body
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(UserConfig::from_line)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { users })
    }
}
