use std::{fs::File, io::BufReader};

use chrono::{NaiveDateTime}; // NaiveDateTimeはタイムゾーンが含まれない時間情報。DateTimeは含まれる。
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Schedule {
    id: u64,
    subject: String,
    start: NaiveDateTime,
    end: NaiveDateTime
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Calendar {
    schedules: Vec<Schedule>,
}

const SCHEDULE_FILE: &str = "schedule.json";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// 予定の一覧表示
    List,
}

fn main() {
    let options = Cli::parse();
    match options.command {
        Commands::List => show_list(),
    }
}

fn show_list() {
    let calendar: Calendar = {
        let file = File::open(SCHEDULE_FILE).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };
    println!("ID\tSTART\tEND\tSUBJECT");
    for schedule in calendar.schedules {
        println!(
            "{}\t{}\t{}\t{}",
            schedule.id, schedule.start, schedule.end, schedule.subject
        );
    }
}