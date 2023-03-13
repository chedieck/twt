use chrono::{Utc, TimeZone, Duration};
use std::error::Error;
use std::collections::HashMap;
use csv::ReaderBuilder;
use super::Log;


const EXPECTED_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

struct LogDuration {
    window_class_name: String,
    window_name: Option<String>,
    duration: Duration
}

pub struct LogDurationList {
    log_durations: Vec<LogDuration>
}

impl LogDuration { 
    fn from_record (record: csv::StringRecord, duration: Duration) -> Result<Self, Box<dyn Error>> {
        Ok(
            Self {
                window_class_name:  record[0].to_string(),
                window_name:  Some(record[1].to_string()),
                duration
            }
        )
    }

    fn pretty_duration(&self) -> String {
        match self.duration.num_seconds() {
            n if n < 60 => format!("{n}s"),
            n if (60..3600).contains(&n) => format!(
                "{}m{}s",
                self.duration.num_minutes(),
                self.duration.num_seconds() % 60
            ),
            n if (3600..86400).contains(&n)=> format!(
                "{}h{}m{}s",
                self.duration.num_hours(),
                self.duration.num_minutes() % 60,
                self.duration.num_seconds() % 60
            ),
            n if (86400..604800).contains(&n)=> format!(
                "{}d, {}h{}m{}s",
                self.duration.num_days(),
                self.duration.num_hours() % 24,
                self.duration.num_minutes() % 60,
                self.duration.num_seconds() % 60
            ),
            _ => format!("{}w, {}d, {}h{}m{}s",
                self.duration.num_weeks(),
                self.duration.num_days() % 7,
                self.duration.num_hours() % 24,
                self.duration.num_minutes() % 60,
                self.duration.num_seconds() % 60
            ),
        }
    }
}

impl LogDurationList {
    pub fn create_for_scope(begin_date_str: &str, end_date_str: &str) -> Result<Self, Box<dyn Error>> {
        let begin = iso_to_timestamp_millis(begin_date_str)?;
        let end = iso_to_timestamp_millis(end_date_str)?;


        let mut rdr = ReaderBuilder::new().
        delimiter(b'\t')
        .from_path(Log::get_log_path()?)?;
        let mut log_durations: Vec<LogDuration> = vec![];
        for result in rdr.records() {
            let record = result?;
            let duration = get_record_duration_for_scope(begin, end, &record)?;
            let log_duration = LogDuration::from_record(record, duration)?;
            log_durations.push(log_duration)
        }

        Ok(
            Self {
                log_durations
            }
        )
    }

    fn get_max_log_name_length(&self) -> usize {
        self.log_durations.iter()
        .map(|l| l.window_class_name.len())
        .max()
        .unwrap()

    }

    fn from_duration_hash_map(map: HashMap<&str, Duration>) -> Self {
        let mut log_durations = map.iter()
                .map(|t| LogDuration {
                    window_class_name: String::from(*t.0),
                    window_name: None,
                    duration: *t.1
                })
            .collect::<Vec<LogDuration>>();
        log_durations.sort_by(|a, b| b.duration.partial_cmp(&a.duration).unwrap());
         Self {
            log_durations
        }
    }

    fn from_name_and_duration_hash_map(map: HashMap<(&str, &str), Duration>) -> Self {
        let mut log_durations = map.iter()
                .map(|t| LogDuration {
                    window_class_name: String::from(t.0.1),
                    window_name: Some(String::from(t.0.1)),
                    duration: *t.1
                })
            .collect::<Vec<LogDuration>>();
        log_durations.sort_by(|a, b| b.duration.partial_cmp(&a.duration).unwrap());
         Self {
            log_durations
        }
    }

    pub fn log_durations_condensed_by_class_and_name(&self) -> Self {
        let mut map: HashMap<(&str, &str), Duration> = HashMap::new();

        for log_duration in &self.log_durations {
            let index = (log_duration.window_class_name.as_str(),
            log_duration.window_name.as_ref().unwrap().as_str());
            match map.get(&index) {
                Some(&duration) => map.insert(index, duration + log_duration.duration),
                _ =>  map.insert(index, log_duration.duration)
            };
        }

        Self::from_name_and_duration_hash_map(map)
    }

    pub fn log_durations_condensed_by_class(&self) -> Self {
        let mut map: HashMap<&str, Duration> = HashMap::new();

        for log_duration in &self.log_durations {
            let index = log_duration.window_class_name.as_str();
            match map.get(index) {
                Some(&duration) => map.insert(index, duration + log_duration.duration),
                _ =>  map.insert(index, log_duration.duration)
            };
        }

        Self::from_duration_hash_map(map)
    }

    pub fn show_simple_use_list(&self) {
        let padding = self.get_max_log_name_length() + 1;
        for log_duration in &self.log_durations {
            println!("{:pad$}: {}",
                log_duration.window_class_name,
                log_duration.pretty_duration(),
                pad=padding);
        }
    }
}

fn iso_to_timestamp_millis(date_str: &str) -> Result<i64, Box<dyn Error>> {
    let naive_datetime = Utc.datetime_from_str(date_str, EXPECTED_DATE_FORMAT)?;
    Ok(naive_datetime.timestamp_millis())
}

fn get_record_duration_for_scope(lower_limit: i64, upper_limit: i64, record: &csv::StringRecord) -> Result<Duration, Box<dyn Error>> {
    let record_start = record[2].parse::<i64>()?;
    let record_end = record[3].parse::<i64>()?;

    if record_end < lower_limit || record_start > upper_limit {
        return Ok(
            Duration::milliseconds(0)
        )
    }
    let true_upper = std::cmp::min(record_end, upper_limit);
    let true_bottom = std::cmp::max(record_start, lower_limit);

    Ok(
        Duration::milliseconds(true_upper - true_bottom)
    )

}
