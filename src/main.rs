use std::process::Command;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::error::Error;
use duration_string::DurationString;
use std::path;

mod stat;


const LOG_CHECK_DELAY_MS: u64 = 100;
const AFK_INTERVAL_MS: u64 = 5 * 60 * 1000; // 5 minutes


#[derive(Debug, Clone)]
pub struct Log {
    window_class_name: String,
    window_name: String,
    start: Option<i64>,
    end: Option<i64>,
}

impl Log {
    fn same_window_as (&self, other_log: &Log) -> bool {
        self.window_class_name == other_log.window_class_name
        && self.window_name == other_log.window_name
    }

    fn to_csv_line(&self) -> Result<String, Box<dyn Error>> {
        match self.start {
            Some(start) => {
                return Ok(format!("{}\t{}\t{}\t{}",
                    self.window_class_name,
                    self.window_name,
                    start,
                    ""
                ))
            },
            None => panic!("Missing start date")
        }
    }

    fn get_log_path() -> Result<path::PathBuf, Box<dyn Error>> {
        let dir_path = dirs::data_dir().unwrap().join("twt/");
        let csv_file_name = "main.csv";
        if !dir_path.is_dir() {
            std::fs::create_dir(&dir_path)?;
        }
        let file_path = dir_path.join(csv_file_name);
        if !file_path.is_file() {
            let header_string = "window_class_name\twindow_name\tstart\tend\n";
            let mut file = std::fs::File::create(&file_path)?;
            file.write_all(header_string.as_bytes())?;
        }
        Ok(file_path)
    }
}

fn get_current_window_log() -> Result<Log, Box<dyn Error>> {
    let window_data_string = String::from_utf8(
        Command::new("xdotool")
        .arg("getwindowfocus")
        .arg("getwindowclassname")
        .arg("getwindowname")
        .output().expect("Failed to get window name or class name.").stdout
    )?;
    let window_data = window_data_string
    .trim()
    .split('\n')
    .map(|s| s.replace(['\n', '\t'], " ").trim().to_string())
    .collect::<Vec<String>>();
    
    if window_data.len() < 2 {
        return Err(
            format!("Failed to get window name or class name. Got: {window_data:?}").into()
        )
    }

    let timestamp = chrono::Utc::now().timestamp_millis();
    Ok(
        Log {
            window_class_name: window_data[0].to_string(),
            window_name: window_data[1].to_string(),
            start: Some(timestamp),
            end: None
        }
    )
}

fn set_end_on_last_entry() -> Result<(), Box<dyn Error>> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    Command::new("sed")
        .arg("-i")
        .arg(format!(
                "$s/\t[^\t]*/\t{timestamp}/3"
        ))
        .arg(Log::get_log_path()?)
        .output()?;
    Ok(())
}

fn set_new_log(log: &Log) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().append(true).open(Log::get_log_path()?)?;
    writeln!(file, "{}", log.to_csv_line()?)?;
    Ok(())
}

fn start () -> Result<Log, Box<dyn Error>>  {
    let first_log_result = get_current_window_log(); 
    let Ok(first_log) = first_log_result else {
        std::thread::sleep(std::time::Duration::from_millis(LOG_CHECK_DELAY_MS));
        return start()
    };
    set_new_log(&first_log)?;
    Ok(first_log)
}

fn is_running_already() -> Result<bool, Box<dyn Error>> {
    let pids_len = String::from_utf8(
        Command::new("pidof")
        .arg("twt")
        .output()?
        .stdout
    )?
        .split_whitespace()
        .count();
    if pids_len > 1 {
        return Ok(true)
    }
    Ok(false)
}

fn run() -> Result<(), Box<dyn Error>> {
    if is_running_already()? {
        return Err("There is already a running instance.".into())
    }
    let mut start_new_log = true;
    let mut last_log = start()?;
    loop {
        let current_window_log_result = get_current_window_log();
        let Ok(current_window_log) = current_window_log_result else {
            start_new_log = true;
            continue;
        };
        set_end_on_last_entry()?;
        if start_new_log {
            set_new_log(&current_window_log)?;
            last_log = current_window_log.clone();
            start_new_log = false
        }
        if !current_window_log.same_window_as(&last_log) {
            start_new_log = true
        }
        std::thread::sleep(std::time::Duration::from_millis(LOG_CHECK_DELAY_MS));
    }
}

fn help() {
    println!("Usage: twt [command] [..args]");
    println!("[command]    [..args]                   description");
    println!("-------");
    println!("run          Collect window usage information and save it to $HOME/.local/share/twt/main.csv.");
    println!("             This should be ran as a daemon, can't be run twice.");
    println!("topc         [start_date] [end_date]    Shows most used programs between the two dates, by window class");
    println!("topn         [start_date] [end_date]    Shows most used programs between the two dates, by window name");
    println!("lastcn       [n]                        Shows the time spent on the last [n] logs");
    println!("lastnn       [n]                        Shows the time spent on the last [n] logs");
    println!("-------");
    println!("[start_date] and [end_date] should be ISO formatted strings on UTC timezone,");
    println!("that is: %Y-%m-%d %H:%M:%S, such as e.g: 2023-03-13 17:29:00.");
}

fn str_to_duration(duration_str: &str) -> chrono::Duration {
    let dur: std::time::Duration = DurationString::from_string(String::from(duration_str)).unwrap().into();
    chrono::Duration::from_std(dur).unwrap()
}

fn parse_args(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    match args[1].as_str() {
        "run" => {
            run()?;
            Ok(())
        },
        "nspan" => {
            let begin = stat::iso_to_timestamp_millis(&args[2])?;
            let end = stat::iso_to_timestamp_millis(&args[3])?;

            let log_duration_list = stat::LogDurationList::create_for_scope(begin, end)?;
            log_duration_list.log_durations_condensed_by_class().show_simple_use_list();
            Ok(())
        },
        "cspan" => {
            let begin = stat::iso_to_timestamp_millis(&args[2])?;
            let end = stat::iso_to_timestamp_millis(&args[3])?;

            let log_duration_list = stat::LogDurationList::create_for_scope(begin, end)?;
            log_duration_list.log_durations_condensed_by_class_and_name().show_simple_use_list();
            Ok(())
        },
        "clast" => {
            let duration_str = &args[2];
            let duration = str_to_duration(duration_str);
            let log_duration_list = stat::LogDurationList::create_for_last_duration(duration)?;
            log_duration_list.log_durations_condensed_by_class().show_simple_use_list();
            Ok(())
        },
        "nlast" => {
            let duration_str = &args[2];
            let duration = str_to_duration(duration_str);
            let log_duration_list = stat::LogDurationList::create_for_last_duration(duration)?;
            log_duration_list.log_durations_condensed_by_class_and_name().show_simple_use_list();
            Ok(())
        },
        _ => {
            help();
            Ok(())
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            help();
            Ok(())
        },
        _ => {
            parse_args(args);
            Ok(())
        }
    }
}

