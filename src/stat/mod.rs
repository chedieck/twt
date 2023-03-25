use chrono::{Utc, TimeZone, Duration};
use std::error::Error;
use std::collections::HashMap;
use csv::ReaderBuilder;
use super::Log;
use std::process::Command;


const EXPECTED_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

struct LogDuration {
    log: Log,
    title: Option<String>,
    duration: Duration
}

struct LogDurationView {
    title: String,
    duration: Duration
}

pub struct LogDurationListView {
    log_duration_views: Vec<LogDurationView>
}

pub struct LogDurationList {
    log_durations: Vec<LogDuration>
}

impl LogDurationListView {
    fn from_duration_hash_map(map: HashMap<&str, Duration>) -> Self {
        let mut log_duration_views = map.iter()
            .filter(|t| t.1.num_milliseconds() != 0)
            .map(|t| LogDurationView {
                title: String::from(*t.0),
                duration: *t.1
            })
        .collect::<Vec<LogDurationView>>();
        log_duration_views.sort_by(|a, b| b.duration.partial_cmp(&a.duration).unwrap());
        Self {
            log_duration_views
        }
    }

    fn get_max_title_length(&self) -> usize {
        self.log_duration_views
            .iter()
            .map(|l| l.title.chars().count())
            .max()
            .unwrap()
    }

    pub fn show_usage_list(&self) {
        let padding = self.get_max_title_length() + 1;
        for log_duration_view in &self.log_duration_views {
            println!("{:pad$}: {}",
                log_duration_view.title,
                log_duration_view.pretty_duration(),
                pad=padding);
        }
    }
}


impl LogDurationView {
    fn pretty_duration(&self) -> String {
        match self.duration.num_seconds() {
            n if n < 1 => format!("{}ms", self.duration.num_milliseconds()),
            n if (1..60).contains(&n) => format!("{n}s"),
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

impl LogDuration {
    fn from_record(record: csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        let log = Log::from_record(record)?;

        let duration = Duration::milliseconds(log.end.unwrap() - log.start.unwrap());
        Ok(
            Self {
                log,
                title: None,
                duration
            }
        )
    }

    fn from_record_on_duration (record: csv::StringRecord, duration: Duration) -> Result<Self, Box<dyn Error>> {
        let log = Log::from_record(record)?;
        Ok(
            Self {
                log,
                title: None,
                duration
            }
        )
    }
}

pub enum LogColumn {
    Name,
    Class
}

impl LogColumn {
    pub fn from_arg(arg: &str) -> Result<LogColumn, Box<dyn Error>> {
        match arg {
            "n" => {
                Ok(LogColumn::Name)
            }
            "c" => {
                Ok(LogColumn::Class)
            }
            _ => {
                Err("Wrong argument for `twt stat [last|span]`, should be one of [n|c].".into())
            }
        }

    }
}

mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_duration_from_record() {
        let record = csv::StringRecord::from(vec!["kitty", "bash", "1678898607940", "1678898608940"]);

        let log_duration = LogDuration::from_record(record).unwrap();
        assert_eq!(log_duration.log.window_name, String::from("bash"));
        assert_eq!(log_duration.log.window_class, String::from("kitty"));
        assert_eq!(log_duration.duration, Duration::seconds(1));
    }
}

impl LogDurationList {
    fn get_reader() -> Result<csv::Reader<std::fs::File>, Box<dyn Error>>{
        Ok(
            ReaderBuilder::new().
            delimiter(b'\t')
            .from_path(Log::get_log_path()?)?
        )
    }

    fn from_vec(log_durations: Vec<LogDuration>) -> Self {
        return Self {
            log_durations
        }
    }

    pub fn create_for_scope(begin: i64, end: i64) -> Result<Self, Box<dyn Error>> {
        let mut rdr = Self::get_reader()?;
        let mut log_durations: Vec<LogDuration> = vec![];
        for result in rdr.records() {
            let record = result?;
            let duration = get_record_duration_for_scope(begin, end, &record)?;
            let log_duration = LogDuration::from_record_on_duration(record, duration)?;
            log_durations.push(log_duration)
        }

        Ok(Self::from_vec(log_durations))
    }

    pub fn create_for_last_duration(duration: Duration) -> Result<Self, Box<dyn Error>> {
        let end = chrono::Utc::now().timestamp_millis();
        let begin = end - duration.num_milliseconds();


        let mut rdr = Self::get_reader()?;
        let mut log_durations: Vec<LogDuration> = vec![];
        for result in rdr.records() {
            let record = result?;
            let duration = get_record_duration_for_scope(begin, end, &record)?;
            let log_duration = LogDuration::from_record_on_duration(record, duration)?;
            log_durations.push(log_duration)
        }

        Ok(Self::from_vec(log_durations))
    }

    pub fn create_for_last_n(n: &usize) -> Result<Self, Box<dyn Error>> {
        let tail_bytes: &[u8] = &Command::new("tail")
            .arg("-n")
            .arg(format!("{n}"))
            .arg(Log::get_log_path()?)
            .output().expect("Fail to tail log file.")
            .stdout;
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(tail_bytes);
        let mut log_durations: Vec<LogDuration> = vec![];

        for result in rdr.records() {
            let record = result?;
            let log_duration = LogDuration::from_record(record)?;
            log_durations.push(log_duration)
        }
        Ok(Self::from_vec(log_durations))
    }

    pub fn get_view_for_log_column(&self, log_column: &LogColumn, regex_pattern: Option<&regex::Regex>) -> LogDurationListView {
        let mut map: HashMap<&str, Duration> = HashMap::new();

        for log_duration in &self.log_durations {
            let index = match log_column {
                LogColumn::Class => log_duration.log.window_class.as_str(),
                LogColumn::Name => log_duration.log.window_name.as_str(),
            };
            if let Some(re) = regex_pattern {
                if !re.is_match(index) {
                    continue
                }
            };
            match map.get(index) {
                Some(&duration) => map.insert(index, duration + log_duration.duration),
                _ =>  map.insert(index, log_duration.duration)
            };
        }

        LogDurationListView::from_duration_hash_map(map)
    }
}

pub fn iso_to_timestamp_millis(date_str: &str) -> Result<i64, Box<dyn Error>> {
    let naive_datetime = Utc.datetime_from_str(date_str, EXPECTED_DATE_FORMAT)?;
    Ok(naive_datetime.timestamp_millis())
}

fn get_record_duration_for_scope(lower_limit: i64, upper_limit: i64, record: &csv::StringRecord) -> Result<Duration, Box<dyn Error>> {
    let record_start = record[2].parse::<i64>()?;
    let record_end = record[3].parse::<i64>().unwrap();

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
