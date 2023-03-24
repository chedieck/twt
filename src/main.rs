use std::process::Command;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::error::Error;
use duration_string::DurationString;
use std::path;
use xcb::x::Window;
use xcb::Connection;
use stat::LogColumn;

mod stat;


const LOG_CHECK_DELAY_MS: u64 = 100;
const AFK_INTERVAL_MS: u32 = 5 * 60 * 1000; // 5 minutes


#[derive(Debug, Clone)]
pub struct Log {
    window_class: String,
    window_name: String,
    start: Option<i64>,
    end: Option<i64>,
}

impl Log {
    fn same_window_as (&self, other_log: &Log) -> bool {
        self.window_class == other_log.window_class
        && self.window_name == other_log.window_name
    }

    fn to_csv_line(&self) -> Result<String, Box<dyn Error>> {
        match self.start {
            Some(start) => {
                return Ok(format!("{}\t{}\t{}\t{}",
                    self.window_class,
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
            let header_string = "window_class\twindow_name\tstart\tend\n";
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
            window_class: window_data[0].to_string(),
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
fn is_user_afk (conn: &Connection, window: Window) -> bool {
    let query_info = xcb::screensaver::QueryInfo {
        drawable: xcb::x::Drawable::Window(window)
    };
    let query_info_cookie = conn.send_request(&query_info);
    let query_info_reply = conn.wait_for_reply(query_info_cookie).unwrap();
    if query_info_reply.ms_since_user_input() > AFK_INTERVAL_MS {
        return true
    }
    false
}

fn run() -> Result<(), Box<dyn Error>> {
    if is_running_already()? {
        return Err("There is already a running instance.".into())
    }

    // Connect to the X server.
    let (conn, screen_num) = xcb::Connection::connect_with_extensions(
        None, &[xcb::Extension::ScreenSaver], &[])?;
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let root_window = screen.root();


    // Start logs
    let mut start_new_log = true;
    let mut last_log = start()?;

    // Update logs on a loop
    loop {
        std::thread::sleep(std::time::Duration::from_millis(LOG_CHECK_DELAY_MS));
        if is_user_afk(&conn, root_window) {
            start_new_log = true;
            continue
        }
        let current_window_log_result = get_current_window_log();
        let Ok(current_window_log) = current_window_log_result else {
            start_new_log = true;
            continue;
        };
        if start_new_log {
            set_new_log(&current_window_log)?;
            last_log = current_window_log.clone();
            start_new_log = false
        }
        set_end_on_last_entry()?;
        if !current_window_log.same_window_as(&last_log) {
            start_new_log = true
        }
    }
}

fn help() {
    println!("Usage: twt [command] [..args]");
    println!("[command]  [..args]                     description");
    println!("---------------------------------------------------");
    println!("help                                    Show this message.");
    println!("run                                     Start twt");
    println!();
    println!("stat       [last|span] [n|c] [..args]   Get information about used windows, by [c]lass or [n]ame:");
    println!("            span [n|c] [begin] [end]    Shows most used programs between the two dates");
    println!("            last [n|c] [duration]       Shows most used programs on the last [duration]");
    println!("-------------------------------------------------------------------------------------------------------");
    println!("[begin] and [end] should be ISO formatted strings on UTC timezone:");
    println!("   %Y-%m-%d %H:%M:%S, e.g: 2023-03-13 17:29:00.");
    println!("[duration] is something like 1h, 2d, 1s, 800ms etc.");
}

fn str_to_duration(duration_str: &str) -> chrono::Duration {
    let dur: std::time::Duration = DurationString::from_string(String::from(duration_str)).unwrap().into();
    chrono::Duration::from_std(dur).unwrap()
}

fn parse_log_durations (log_duration_list: stat::LogDurationList, log_column: LogColumn, regex_pattern: Option<String>) -> Result<(), Box<dyn Error>> {
    match log_column {
        LogColumn::Name => {
            log_duration_list.log_durations_condensed_by_class_and_name().show_usage_list(&LogColumn::Name);
            Ok(())
        }
        LogColumn::Class => {
            log_duration_list.log_durations_condensed_by_class().show_usage_list(&LogColumn::Class);
            Ok(())
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    match args[1].as_str() {
        "help" => {
            help();
            Ok(())
        }
        "run" => {
            run()?;
            Ok(())
        }
        "stat" => {
            match args[2].as_str() {
                "last" => {
                    if args.len() < 5 {
                        help();
                        return Err("Missing arguments.".into())
                    }
                    let duration_str = &args[4];
                    let duration = str_to_duration(duration_str);
                    let log_duration_list = stat::LogDurationList::create_for_last_duration(duration)?;
                    let log_column = LogColumn::from_arg(args[3].as_str())?;
                    let regex_pattern: Option<String> = args.get(5).cloned();
                    parse_log_durations(log_duration_list, log_column, regex_pattern)
                }
                "span" => {
                    if args.len() < 6 {
                        help();
                        return Err("Missing arguments.".into())
                    }
                    let begin = stat::iso_to_timestamp_millis(&args[4])?;
                    let end = stat::iso_to_timestamp_millis(&args[5])?;

                    let log_duration_list = stat::LogDurationList::create_for_scope(begin, end)?;
                    let log_column = LogColumn::from_arg(args[3].as_str())?;
                    let regex_pattern: Option<String> = args.get(5).cloned();
                    parse_log_durations(log_duration_list, log_column, regex_pattern)
                }
                _ => {
                    help();
                    Err("Wrong argument for `twt stat`, should be one of [span|last].".into())
                }
            }
        }
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
            parse_args(args)?;
            Ok(())
        }
    }
}

