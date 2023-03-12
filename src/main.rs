use std::process::Command;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::error::Error;


const MAIN_LOG_CSV: &str = "/home/ttv1/codes/ttw/src/main.csv";

#[derive(Debug)]
struct Log {
    window_class_name: String,
    window_name: String,
    start: Option<i64>,
    end: Option<i64>,
}

impl Log {
    fn same_window_as (&self, other_log: &Log) -> bool {
        self.window_class_name == other_log.window_class_name
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
}

fn get_log() -> Result<Log, Box<dyn Error>> {
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

    let timestamp = chrono::Utc::now().timestamp();
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
    let timestamp = chrono::Utc::now().timestamp();
    Command::new("sed")
        .arg("-i")
        .arg(format!(
                "$s/\t/\t{timestamp}/3"
        ))
        .arg("src/main.csv")
        .output()?;
    Ok(())
}

fn set_new_log(log: &Log) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().append(true).open(MAIN_LOG_CSV)?;
    writeln!(file, "{}", log.to_csv_line()?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut last_log = get_log()?;
    set_new_log(&last_log)?;
    loop { // WIP ensure last timestamp saved at each iteration also
        let new_log = get_log()?;
        if new_log.same_window_as(&last_log) {
            continue
        }
        set_end_on_last_entry()?;
        set_new_log(&new_log)?;
        last_log = new_log;
    }
}
