use chrono::{DateTime, SecondsFormat, Utc};

pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

pub fn now_utc_iso() -> String {
    now_utc().to_rfc3339_opts(SecondsFormat::Secs, true)
}
