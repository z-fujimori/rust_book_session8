use std::{fs::File, io::{BufReader, BufWriter}};

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
impl Schedule {
    fn intersects(&self, other: &Schedule) -> bool {
      self.start < other.end && other.start < self.end
    }
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
  /// 予定の追加
  Add {
    /// タイトル
    subject: String,
    /// 開始時刻
    start: NaiveDateTime,
    /// 終了時間
    end: NaiveDateTime,
  },
}

fn main() {
  let options = Cli::parse();
  match options.command {
    Commands::List => show_list(),
    Commands::Add { subject, start, end }
      => add_schedule(subject, start, end),
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

fn add_schedule(
  subject: String,
  start: NaiveDateTime,
  end: NaiveDateTime,
){
  let mut calendar: Calendar = {
    let file = File::open(SCHEDULE_FILE).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
  };

  // 予定の作成
  let id = calendar.schedules.len() as u64;
  let new_schedule = Schedule {
    id,
    subject,
    start,
    end,
  };
  // 重複判定
  for schedule in &calendar.schedules {
    if schedule.intersects(&new_schedule){
      println!("エラー: 予定が重複しています");
      return;
    }
  }
  //予定の追加
  calendar.schedules.push(new_schedule);

  // 予定の保存
  {
    let file = File::create(SCHEDULE_FILE).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &calendar).unwrap();
  }
  println!("予定を追加しました。");
}

#[cfg(test)]
mod tests {
  use super::*;

  fn naive_date_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
  ) -> NaiveDateTime {
    chrono::NaiveDate::from_ymd_opt(year, month, day)
      .unwrap()
      .and_hms_opt(hour, minute, second)
      .unwrap()
  }

  #[test]
  fn test_schedule_intersects_1(){
    let schedule = Schedule {
      id: 1,
      subject: "既存予定１".to_string(),
      start: naive_date_time(2024, 1, 1, 18, 15, 0),
      end: naive_date_time(2024, 1, 1, 19, 15, 0),
    };
    let new_schedule = Schedule {
      id: 999,
      subject: "新規予定".to_string(),
      start: naive_date_time(2024, 1, 1, 19, 0, 0),
      end: naive_date_time(2024, 1, 1, 20, 0, 0),
    };
    assert!(schedule.intersects(&new_schedule));
  }
  #[test]
  fn test_schedule_intersects_2(){
    let schedule = Schedule {
      id: 1,
      subject: "既存予定１".to_string(),
      start: naive_date_time(2024, 1, 1, 19, 45, 0),
      end: naive_date_time(2024, 1, 1, 20, 45, 0),
    };
    let new_schedule = Schedule {
      id: 999,
      subject: "新規予定".to_string(),
      start: naive_date_time(2024, 1, 1, 19, 0, 0),
      end: naive_date_time(2024, 1, 1, 20, 0, 0),
    };
    assert!(schedule.intersects(&new_schedule));
  }
  #[test]
  fn test_schedule_intersects_3(){
    let schedule = Schedule {
      id: 1,
      subject: "既存予定１".to_string(),
      start: naive_date_time(2024, 1, 1, 18, 30, 0),
      end: naive_date_time(2024, 1, 1, 20, 15, 0),
    };
    let new_schedule = Schedule {
      id: 999,
      subject: "新規予定".to_string(),
      start: naive_date_time(2024, 1, 1, 19, 0, 0),
      end: naive_date_time(2024, 1, 1, 20, 0, 0),
    };
    assert!(schedule.intersects(&new_schedule));
  }
  #[test]
  fn test_schedule_intersects_4(){
    let schedule = Schedule {
      id: 1,
      subject: "既存予定１".to_string(),
      start: naive_date_time(2024, 1, 1, 20, 15, 0),
      end: naive_date_time(2024, 1, 1, 20, 45, 0),
    };
    let new_schedule = Schedule {
      id: 999,
      subject: "新規予定".to_string(),
      start: naive_date_time(2024, 1, 1, 19, 0, 0),
      end: naive_date_time(2024, 1, 1, 20, 0, 0),
    };
    assert!(!schedule.intersects(&new_schedule));
  }
  #[test]
  fn test_schedule_intersects_5(){
    let schedule = Schedule {
      id: 1,
      subject: "既存予定１".to_string(),
      start: naive_date_time(2024,12, 8, 9, 0, 0),
      end: naive_date_time(2024, 12, 8, 10, 30, 0),
    };
    let new_schedule = Schedule {
      id: 999,
      subject: "新規予定".to_string(),
      start: naive_date_time(2024, 12, 15, 10, 0, 0),
      end: naive_date_time(2024, 12, 15, 11, 0, 0),
    };
    assert!(!schedule.intersects(&new_schedule));
  }
  #[test]
  fn test_schedule_intersects_6(){
    let schedule= Schedule {
      id: 6,
      subject:"既存予定6".to_string(),
      start: naive_date_time(2024, 1, 1, 19, 10, 0),
      end: naive_date_time(2024, 1, 1, 19, 45, 0),
    };
    let new_schedule = Schedule  {
      id: 999,
      subject: "新規予定6".to_string(),
      start: naive_date_time(2024, 1, 1, 19, 0, 0),
      end: naive_date_time(2024, 1, 1, 20, 0, 0),
    };
    assert!(schedule.intersects(&new_schedule));
  }
}
