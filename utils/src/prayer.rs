use chrono::{format, DateTime, Days, Local, NaiveDate, NaiveDateTime, NaiveTime};
use reqwest;
use std::{collections::VecDeque, time::Duration};
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
    pub time: NaiveDateTime,
    pub name: String,
}

#[derive(Debug)]
pub struct Prayers {
    city: String,
    country: String,
    // TODO: we should remove this pubs 
    pub next: Option<Prayer>,
    pub prayer_que: VecDeque<Prayer>,
    pub prayers: Vec<Prayer>,
}

impl Prayers {
    pub async fn new_async(
        city: String,
        country: String,
        date: NaiveDate,
    ) -> Result<Prayers, PrayerRetrievalError> {
        let formatted_date = date.format("%d-%m-%Y").to_string();
        let url = format!(
            "https://api.aladhan.com/v1/timingsByCity/{}?city={}&country={}",
            formatted_date, city, country
        );
        let map = reqwest::get(url).await?.json::<serde_json::Value>().await?;
        println!("Looking up prayers for: {}", formatted_date);

        let mut prayers = Vec::new();
        // let data_date = map["data"]["date"]["gregorian"]["date"].as_str().unwrap();
        // let date = DateTime::parse_from_str(, s)?;
        for (name, time) in map["data"]["timings"].as_object().unwrap() {
            let time_parts: Vec<&str> = time.as_str().unwrap().split(':').collect();
            let hour: u32 = time_parts[0].parse().unwrap();
            let minute: u32 = time_parts[1].parse().unwrap();

            // Combine the parsed time with the current date
            let naive_time = NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
            let datetime = date.and_time(naive_time);

            prayers.push(Prayer {
                name: name.to_owned(),
                time: datetime,
            })
        }
        let mut plist = prayers
            .clone()
            .into_iter()
            .filter(|prayer| {
                // Filter out invalid or zero durations
                let duration = prayer
                    .time
                    .signed_duration_since(Local::now().naive_local());
                // println!("Prayer {} has a time delta of {:?}", prayer.name, duration);
                duration.num_seconds() > 0 // Filter out zero or negative durations
            })
            .collect::<Vec<Prayer>>();
        plist.sort_by(|a, b| a.time.cmp(&b.time));
        // Sort the prayers by the remaining time
        Ok(Prayers {
            city,
            country,
            next: None,
            prayer_que: plist.into(),
            prayers,
        })
    }
    // This future resolves when it is a prayer time. Will refresh the struct if expired
    pub async fn get_next_prayer_duration(self: &mut Self) -> Result<Duration, PrayerRetrievalError> {
        let now = Local::now().naive_local();

        println!("Getting next prayer...");

        self.next = self.prayer_que.pop_front();

        match &self.next {
            Some(current_prayer) => {
                let sleep_dur = current_prayer.time.signed_duration_since(now).num_seconds() as u64;
                println!(
                    "Sleeping for {:?} to wait for {}",
                    sleep_dur, current_prayer.name
                );
                Ok(Duration::new(sleep_dur, 0))
            }
            None => {
                println!("We have expired");
                let new = Self::new_async(
                    self.city.clone(),
                    self.country.clone(),
                    Local::now().date_naive().succ_opt().unwrap(),
                )
                .await?;
                // new.get_next_prayer_async().await;
                *self = new; //apparently this is safe?
                             // TODO: this shouldnt be an error.
                Err(PrayerRetrievalError::Unknown)
            }
        }
    }
}

// General utils

pub fn format_time_difference(future_time: NaiveDateTime) -> String {
    let now = Local::now().naive_local();
    let duration = future_time - now;

    if duration.num_seconds() <= 0 {
        return "now".to_string();
    }

    let hours = duration.num_hours();
    let minutes = (duration.num_minutes() % 60).abs();

    match (hours, minutes) {
        (0, m) => format!("in {} mins", m),
        (h, 0) => format!("in {} hours", h),
        (h, m) => format!("in {} hours and {} mins", h, m),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toronto_canada_prayers() {}
}
