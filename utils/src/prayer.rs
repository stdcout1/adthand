use chrono::{DateTime, Local, Timelike};
use reqwest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrayerRetrievalError {
    #[error("Failed to retrive time for `{0}`")]
    Redaction(String),
    #[error("Empty prayer queue")]
    Empty,
    #[error("Reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("unknown data store error")]
    Unknown,
}

pub enum PrayerResults {
    Prayer(Prayer),
    CaughtUp,
    NotTimeYet(i64),
}

#[derive(Debug)]
pub struct Prayer {
    pub time: DateTime<Local>,
    pub name: String,
}

#[derive(Debug)]
pub struct Prayers {
    pub prayers: Vec<Prayer>,
}

impl Prayers {
    pub fn new(city: String, country: String) -> Result<Prayers, PrayerRetrievalError> {
        let url = format!(
            "https://api.aladhan.com/v1/timingsByCity?city={}&country={}",
            city, country
        );
        let map = reqwest::blocking::get(url)?.json::<serde_json::Value>()?;
        
        let mut prayers = Vec::new();
        let now = Local::now();
        for (name, time) in map["data"]["timings"].as_object().unwrap() {
            let prayer_hour: u32 = time.to_string()[1..3].parse::<u32>().unwrap();
            let prayer_min: u32 = time.to_string()[4..6].parse::<u32>().unwrap();
            let prayer_time = now
                .with_hour(prayer_hour)
                .ok_or(PrayerRetrievalError::Redaction(name.to_string()))?
                .with_minute(prayer_min)
                .ok_or(PrayerRetrievalError::Redaction(name.to_string()))?;

            prayers.push(Prayer {
                name: name.to_owned(),
                time: prayer_time,
            })
        }
        prayers.sort_by(|b, a| a.time.cmp(&b.time));
        Ok(Prayers { prayers })
    }
    pub fn get_next_prayer_unix(
        &mut self,
        leeway: i64,
    ) -> Result<PrayerResults, PrayerRetrievalError> {
        let now = Local::now();
        let first_prayer = self.prayers.last().ok_or(PrayerRetrievalError::Empty)?;
        // let prayer_hour: u32 = now.hour();
        // let prayer_min: u32 = now.minute();
        println!(
            "pt: {}, now: {}, diffrence: {}",
            first_prayer.time,
            now,
            (now - first_prayer.time).num_seconds()
        );
        let duration = (first_prayer.time - now).num_seconds();
        if duration < -60 * leeway {
            self.prayers.pop().unwrap();
            Ok(PrayerResults::CaughtUp)
        } else if duration >= 60 * leeway {
            Ok(PrayerResults::NotTimeYet(duration))
        } else {
            Ok(PrayerResults::Prayer(self.prayers.pop().unwrap())) // should never panic
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toronto_canada_prayers() {
        let mut m = Prayers::new("Toronto".to_owned(), "Canada".to_owned()).unwrap();
        match m.get_next_prayer_unix(1).unwrap() {
            PrayerResults::Prayer(p) => assert_eq!("", p.name),
            PrayerResults::CaughtUp => assert_eq!("", "CaughtUp"),
            PrayerResults::NotTimeYet(t) => assert_eq!(0, t),
        }
    }
}

