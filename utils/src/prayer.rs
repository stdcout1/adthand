use std::{
    thread,
    time::{Duration, SystemTime},
};

use chrono::{format, DateTime, Days, Local, NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::Pacific::Niue;
use reqwest;
use thiserror::Error;

use crate::prayer;

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
    pub time: NaiveDateTime,
    pub name: String,
}

#[derive(Debug)]
pub struct Prayers {
    expiry: NaiveDateTime,
    city: String,
    country: String,
    pub prayers: Vec<Prayer>,
}

impl Prayers {
    pub async fn new_async(city: String, country: String) -> Result<Prayers, PrayerRetrievalError> {
        let url = format!(
            "https://api.aladhan.com/v1/timingsByCity?city={}&country={}",
            city, country
        );
        let map = reqwest::get(url).await?.json::<serde_json::Value>().await?;

        let mut prayers = Vec::new();
        let creation_time = Local::now();
        let expiry = creation_time
            .date_naive()
            .succ_opt()
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        // let data_date = map["data"]["date"]["gregorian"]["date"].as_str().unwrap();
        // let date = DateTime::parse_from_str(, s)?;
        for (name, time) in map["data"]["timings"].as_object().unwrap() {
            let time_parts: Vec<&str> = time.as_str().unwrap().split(':').collect();
            let hour: u32 = time_parts[0].parse().unwrap();
            let minute: u32 = time_parts[1].parse().unwrap();

            // Combine the parsed time with the current date
            let naive_date = creation_time.date_naive();
            let naive_time = NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
            let datetime = naive_date.and_time(naive_time);

            prayers.push(Prayer {
                name: name.to_owned(),
                time: datetime,
            })
        }
        Ok(Prayers {
            city,
            country,
            expiry,
            prayers,
        })
    }
    // This future resolves when it is a prayer time. Will refresh the struct if expired
    pub async fn get_next_prayer_async(self: &mut Self) -> Result<String, PrayerRetrievalError> {
        let now = Local::now().naive_local();
        if now > self.expiry {
            println!("We have expired");
            let new = Self::new_async(self.city.clone(), self.country.clone()).await?;
            std::mem::replace(self, new);
        }

        println!("Getting next prayer...");
        // println!("{:?}", self.prayers);
        self.prayers = self.prayers
            .clone() // Clone the prayers list
            .into_iter()
            .filter(|prayer| {
                // Filter out invalid or zero durations
                let duration = prayer.time.signed_duration_since(now);
                // println!("Prayer {} has a time delta of {:?}", prayer.name, duration);
                duration.num_seconds() > 0 // Filter out zero or negative durations
            })
            .collect();
        // Sort the prayers by the remaining time
        self.prayers.sort_by(|a, b| a.time.cmp(&b.time));

        // println!("{:?}", prayers);
        let sleep_dur = self.prayers[0].time.signed_duration_since(now).num_seconds() as u64;
        println!("Sleeping for {:?} to wait for {}", sleep_dur, self.prayers[0].name);
        tokio::time::sleep(Duration::new(sleep_dur, 0)).await;
        let name = self.prayers.remove(0);
        Ok(name.name)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toronto_canada_prayers() {}
}
