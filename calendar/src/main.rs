use std::{fs::File, io::{BufReader, BufWriter}};

use chrono::{NaiveDateTime}; // NaiveDateTimeはタイムゾーンが含まれない時間情報。DateTimeは含まれる。
use serde::{Deserialize, Serialize};
use clap::{error, Parser, Subcommand};
use thiserror::Error;

// use rstest::rstest;


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
  /// 予定の削除
  Delete {
    ///　予定のID
    id: u64,
  }
}

#[derive(thiserror::Error, Debug)]
enum MyError {
	#[error("io error: {0}")]
	Io(#[from] std::io::Error),
	
	#[error("json error: {0}")]
	Json(#[from] serde_json::Error),
}
// // 下記を上記のように書ける　
// enum MyError {
// 	Io(std::io::Error),
// 	Json(serde_json::Error),
// }
// impl From<std::io::Error> for MyError {
//     fn from(err: std::io::Error) -> Self {
//         MyError::Io(err)
//     }
// }
// impl From<serde_json::Error> for MyError {
//   fn from(err: serde_json::Error) -> Self {
//       MyError::Json(err)
//   }
// }

fn main() {
  let options = Cli::parse();
  match options.command {
    Commands::List => show_list(),
    Commands::Add { subject, start, end }
      => add_schedule(subject, start, end),
    Commands::Delete { id } => {
      let mut calendar = read_calendar().unwrap();
      if delete_schedule(&mut calendar, id) {
        save_calendar(&calendar);
        println!("予定を削除しました");
      } else {
          println!("エラー:IDが不正です")
      }
    }
  }
}

fn delete_schedule(calendar: &mut Calendar, id: u64) -> bool {
  // 予定の削除
  for i in 0..calendar.schedules.len() {
    if calendar.schedules[i].id == id {
      calendar.schedules.remove(i);
      return true;
    }
  }
  false
}
fn read_calendar() -> Result<Calendar, std::io::Error> {
  let file = File::open(SCHEDULE_FILE)?;
  let reader = BufReader::new(file);
  let calendar = serde_json::from_reader(reader).unwrap();
  Ok(calendar)
}
fn save_calendar(calendar: &Calendar) ->Result<(), MyError> {
  let file = File::create(SCHEDULE_FILE)?;
  let writer = BufWriter::new(file);
  serde_json::to_writer(writer, calendar)?;
  Ok(())
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
  use std::process::ExitCode;

use rstest::rstest;
use serde::de::Expected;
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
  #[rstest]
  #[case(18, 15, 19, 15, true)]
  #[case(19, 45, 20, 45, true)]
  #[case(18, 30, 20, 15, true)]
  #[case(20, 15, 20, 45, false)]
  #[case(18, 15, 18, 45, false)]
  #[case(19, 15, 19, 45, true)]
  fn test_schedule_intersects (
    #[case] h0: u32,
    #[case] m0: u32,
    #[case] h1: u32,
    #[case] m1: u32,
    #[case] shoud_intersect: bool,
  ) {
    let schedule = Schedule {
      id: 0,
      subject: "新規予定".to_string(),
      start: naive_date_time(2024, 1, 1, h0, m0, 0),
      end: naive_date_time(2024, 1, 1, h1, m1, 0),
    };
    let new_schedule = Schedule {
      id: 999,
      subject: "新規予定".to_string(),
      start: naive_date_time(2024, 1, 1, 19, 0, 0),
      end: naive_date_time(2024, 1, 1, 20, 9, 0),
    };
    assert_eq!(shoud_intersect, schedule.intersects(&new_schedule));
  }

  #[test]
  fn test_delete_schedule(){
    let mut calendar = Calendar {
      schedules: vec![
        Schedule {
          id: 0,
          subject: "テスト予定".to_string(),
          start: naive_date_time(2023, 11, 19, 11, 22, 33),
          end: naive_date_time(2023, 11, 19, 22, 33, 44),
        },
        Schedule {
          id: 1,
          subject: "テスト予定2".to_string(),
          start: naive_date_time(2023, 12, 8, 9, 0, 0),
          end: naive_date_time(2023, 12, 8, 10, 30, 0),
        },
        Schedule {
          id: 2,
          subject: "追加できる予定".to_string(),
          start: naive_date_time(2023, 12, 15, 10, 0, 0),
          end: naive_date_time(2023, 12, 15, 11, 00, 0),
        },
      ],
    };
    // id=0を消す
    assert!(delete_schedule(&mut calendar, 0));
    let expected = Calendar {
      schedules: vec![
        Schedule {
          id: 1,
          subject: "テスト予定2".to_string(),
          start: naive_date_time(2023, 12, 8, 9, 0, 0),
          end: naive_date_time(2023, 12, 8, 10, 30, 0),
        },
        Schedule {
          id: 2,
          subject: "追加できる予定".to_string(),
          start: naive_date_time(2023, 12, 15, 10, 0, 0),
          end: naive_date_time(2023, 12, 15, 11, 00, 0),
        },
      ]
    };
    assert_eq!(expected, calendar);
    // id=1を消す
    assert!(delete_schedule(&mut calendar, 1));
    let expected = Calendar {
      schedules: vec![
        Schedule {
          id: 2,
          subject: "追加できる予定".to_string(),
          start: naive_date_time(2023, 12, 15, 10, 0, 0),
          end: naive_date_time(2023, 12, 15, 11, 00, 0),
        },
      ],
    };
    assert_eq!(expected, calendar);
    assert!(delete_schedule(&mut calendar, 2));
    let expected = Calendar {
      schedules: vec![],
    };
    assert_eq!(expected, calendar);
  }
}

// 上記のように書くと省略できる
// #[cfg(test)]
// mod tests {
//   use super::*;

//   fn naive_date_time(
//     year: i32,
//     month: u32,
//     day: u32,
//     hour: u32,
//     minute: u32,
//     second: u32,
//   ) -> NaiveDateTime {
//     chrono::NaiveDate::from_ymd_opt(year, month, day)
//       .unwrap()
//       .and_hms_opt(hour, minute, second)
//       .unwrap()
//   }

//   #[test]
//   fn test_schedule_intersects_1(){
//     let schedule = Schedule {
//       id: 1,
//       subject: "既存予定１".to_string(),
//       start: naive_date_time(2024, 1, 1, 18, 15, 0),
//       end: naive_date_time(2024, 1, 1, 19, 15, 0),
//     };
//     let new_schedule = Schedule {
//       id: 999,
//       subject: "新規予定".to_string(),
//       start: naive_date_time(2024, 1, 1, 19, 0, 0),
//       end: naive_date_time(2024, 1, 1, 20, 0, 0),
//     };
//     assert!(schedule.intersects(&new_schedule));
//   }
//   #[test]
//   fn test_schedule_intersects_2(){
//     let schedule = Schedule {
//       id: 1,
//       subject: "既存予定１".to_string(),
//       start: naive_date_time(2024, 1, 1, 19, 45, 0),
//       end: naive_date_time(2024, 1, 1, 20, 45, 0),
//     };
//     let new_schedule = Schedule {
//       id: 999,
//       subject: "新規予定".to_string(),
//       start: naive_date_time(2024, 1, 1, 19, 0, 0),
//       end: naive_date_time(2024, 1, 1, 20, 0, 0),
//     };
//     assert!(schedule.intersects(&new_schedule));
//   }
//   #[test]
//   fn test_schedule_intersects_3(){
//     let schedule = Schedule {
//       id: 1,
//       subject: "既存予定１".to_string(),
//       start: naive_date_time(2024, 1, 1, 18, 30, 0),
//       end: naive_date_time(2024, 1, 1, 20, 15, 0),
//     };
//     let new_schedule = Schedule {
//       id: 999,
//       subject: "新規予定".to_string(),
//       start: naive_date_time(2024, 1, 1, 19, 0, 0),
//       end: naive_date_time(2024, 1, 1, 20, 0, 0),
//     };
//     assert!(schedule.intersects(&new_schedule));
//   }
//   #[test]
//   fn test_schedule_intersects_4(){
//     let schedule = Schedule {
//       id: 1,
//       subject: "既存予定１".to_string(),
//       start: naive_date_time(2024, 1, 1, 20, 15, 0),
//       end: naive_date_time(2024, 1, 1, 20, 45, 0),
//     };
//     let new_schedule = Schedule {
//       id: 999,
//       subject: "新規予定".to_string(),
//       start: naive_date_time(2024, 1, 1, 19, 0, 0),
//       end: naive_date_time(2024, 1, 1, 20, 0, 0),
//     };
//     assert!(!schedule.intersects(&new_schedule));
//   }
//   #[test]
//   fn test_schedule_intersects_5(){
//     let schedule = Schedule {
//       id: 1,
//       subject: "既存予定１".to_string(),
//       start: naive_date_time(2024,12, 8, 9, 0, 0),
//       end: naive_date_time(2024, 12, 8, 10, 30, 0),
//     };
//     let new_schedule = Schedule {
//       id: 999,
//       subject: "新規予定".to_string(),
//       start: naive_date_time(2024, 12, 15, 10, 0, 0),
//       end: naive_date_time(2024, 12, 15, 11, 0, 0),
//     };
//     assert!(!schedule.intersects(&new_schedule));
//   }
//   #[test]
//   fn test_schedule_intersects_6(){
//     let schedule= Schedule {
//       id: 6,
//       subject:"既存予定6".to_string(),
//       start: naive_date_time(2024, 1, 1, 19, 10, 0),
//       end: naive_date_time(2024, 1, 1, 19, 45, 0),
//     };
//     let new_schedule = Schedule  {
//       id: 999,
//       subject: "新規予定6".to_string(),
//       start: naive_date_time(2024, 1, 1, 19, 0, 0),
//       end: naive_date_time(2024, 1, 1, 20, 0, 0),
//     };
//     assert!(schedule.intersects(&new_schedule));
//   }
// }
