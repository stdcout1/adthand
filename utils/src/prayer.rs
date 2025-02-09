use std::{thread, time::{Duration, SystemTime}};

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
    NotTimeYet(i64),
}

#[derive(Debug, Clone)]
pub struct Prayer {
    pub time: Duration,
    pub name: String,
}

#[derive(Debug)]
pub struct Prayers {
    now: Duration,
    pub prayers: Vec<Prayer>,
}
//since UNIX_EPOCH
fn now() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}

impl Prayers {
    pub fn new(city: &str, country: &str) -> Result<Prayers, PrayerRetrievalError> {
        let url = format!(
            "https://api.aladhan.com/v1/timingsByCity?city={}&country={}",
            city, country
        );
        let map = reqwest::blocking::get(url).unwrap().json::<serde_json::Value>()?;

        let mut prayers = Vec::new();
        let now = now();
        for (name, time) in map["data"]["timings"].as_object().unwrap() {
            let prayer_hour: u64 = time.to_string()[1..3].parse::<u64>().unwrap();
            let prayer_min: u64 = time.to_string()[4..6].parse::<u64>().unwrap();
            let prayer_time = now + Duration::from_secs(prayer_hour * 60 * 60 + prayer_min * 60);

            prayers.push(Prayer {
                name: name.to_owned(),
                time: prayer_time,
            })
        }
        Ok(Prayers { now, prayers })
    }
    pub fn get_next_prayer(self: &Self) -> Result<String, PrayerRetrievalError> {
        let now = now() + (self.prayers[0].time - self.now) - Duration::from_secs(5);
        let mut prayers = self
            .prayers
            .clone() //colone D:
            .into_iter()
            .map(|prayer| Prayer {
                name: prayer.name,
                time: prayer.time.checked_sub(now).unwrap_or(Duration::MAX),
            })
            .filter(|prayers| prayers.time != Duration::MAX)
            .collect::<Vec<Prayer>>();
        prayers.sort_by(|b, a| b.time.cmp(&a.time));
        thread::sleep(prayers[0].time);
        let name = prayers.remove(0);
        Ok(name.name)
    }
    // pub async fn new_async(city: &str, country: &str) -> Result<Prayers, PrayerRetrievalError> {
    //     let url = format!(
    //         "https://api.aladhan.com/v1/timingsByCity?city={}&country={}",
    //         city, country
    //     );
    //     let map = reqwest::get(url).await?.json::<serde_json::Value>().await?;
    //
    //     let mut prayers = Vec::new();
    //     let now = now();
    //     for (name, time) in map["data"]["timings"].as_object().unwrap() {
    //         let prayer_hour: u64 = time.to_string()[1..3].parse::<u64>().unwrap();
    //         let prayer_min: u64 = time.to_string()[4..6].parse::<u64>().unwrap();
    //         let prayer_time = now + Duration::from_secs(prayer_hour * 60 * 60 + prayer_min * 60);
    //
    //         prayers.push(Prayer {
    //             name: name.to_owned(),
    //             time: prayer_time,
    //         })
    //     }
    //     Ok(Prayers { now, prayers })
    // }
    // // This future resolves when it is a prayer time. 
    // pub async fn get_next_prayer_async(self: &Self) -> Result<String, PrayerRetrievalError> {
    //     let now = now() + (self.prayers[0].time - self.now) - Duration::from_secs(5);
    //     let mut prayers = self
    //         .prayers
    //         .clone() //colone D:
    //         .into_iter()
    //         .map(|prayer| Prayer {
    //             name: prayer.name,
    //             time: prayer.time.checked_sub(now).unwrap_or(Duration::MAX),
    //         })
    //         .filter(|prayers| prayers.time != Duration::MAX)
    //         .collect::<Vec<Prayer>>();
    //     prayers.sort_by(|b, a| b.time.cmp(&a.time));
    //     tokio::time::sleep(prayers[0].time).await;
    //     let name = prayers.remove(0);
    //     Ok(name.name)
    // }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toronto_canada_prayers() {}
}

