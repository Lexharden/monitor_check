use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryRecord {
    #[serde(with = "my_date_format")]
    pub time: DateTime<Local>,
    pub name: String,
    pub action: String, // "opened" or "closed"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    records: Vec<HistoryRecord>,
    previous_apps: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        History {
            records: Vec::new(),
            previous_apps: Vec::new(),
        }
    }

    pub fn update(&mut self, current_apps: &[String]) {
        let now = Local::now();

        // Detectar aplicaciones cerradas
        for app in &self.previous_apps {
            if !current_apps.contains(app) {
                self.records.push(HistoryRecord {
                    time: now,
                    name: app.clone(),
                    action: "Cerrado".to_string(),
                });
            }
        }

        // Detectar aplicaciones abiertas
        for app in current_apps {
            if !self.previous_apps.contains(app) {
                self.records.push(HistoryRecord {
                    time: now,
                    name: app.clone(),
                    action: "Abierto".to_string(),
                });
            }
        }

        self.previous_apps = current_apps.to_vec();
    }

    pub fn get_records(&self) -> &[HistoryRecord] {
        &self.records
    }
}

// Módulo para formatear las fechas
mod my_date_format {
    use chrono::{DateTime, Local, SecondsFormat};
    use serde::{self, Serializer, Deserializer, Deserialize};

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = date.to_rfc3339_opts(SecondsFormat::Secs, true);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Local))
    }
}
