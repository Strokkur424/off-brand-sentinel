use crate::wrapper::UserIdWrapper;
use rusqlite::{Connection, Row};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

//<editor-fold desc="Data Types">
pub struct Punishment {
  pub punishment_id: Uuid,
  pub issued_to: UserIdWrapper,
  pub issued_by: UserIdWrapper,
  pub punishment_type: PunishmentType,
  pub duration: Option<Duration>,
  pub stale: bool,
  pub permanent: bool,
  pub time_sec: u64,
  pub reason: Option<String>,
}

#[derive(PartialEq)]
pub enum PunishmentType {
  KICK,
  WARN,
  BAN,
  TIMEOUT,
  NOTE
}

impl PunishmentType {
  pub fn index(&self) -> u8 {
    match self {
      PunishmentType::KICK => 0,
      PunishmentType::WARN => 1,
      PunishmentType::BAN => 2,
      PunishmentType::TIMEOUT => 3,
      PunishmentType::NOTE => 4,
    }
  }

  pub fn from_index(index: u8) -> PunishmentType {
    match index {
      0 => PunishmentType::KICK,
      1 => PunishmentType::WARN,
      2 => PunishmentType::BAN,
      3 => PunishmentType::TIMEOUT,
      4 => PunishmentType::NOTE,
      _ => panic!("Unknown punishment type index: {index}"),
    }
  }
}

#[derive(Debug)]
pub struct Duration {
  pub display: String,
  pub std_duration: std::time::Duration,
}

impl Duration {
  pub fn new(display: &str, std_duration: std::time::Duration) -> Self {
    Duration {
      display: String::from(display),
      std_duration,
    }
  }

  pub fn to_unix_time_from_now(&self) -> u64 {
    (SystemTime::now() + self.std_duration)
      .duration_since(UNIX_EPOCH)
      .unwrap().as_secs()
  }
}

impl Clone for Duration {
  fn clone(&self) -> Self {
    Duration {
      display: self.display.clone(),
      std_duration: self.std_duration.clone(),
    }
  }
}
//</editor-fold>

//<editor-fold desc="Internal Utility">
fn get_connection() -> Result<Connection, String> {
  let dir: String = std::env::var("SENTINEL_WORKING_DIR").unwrap_or_else(|_| String::from("./"));
  fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directories for {dir}: {e}"))?;

  let path = format!("{dir}database.sqlite");
  Connection::open(path.clone()).map_err(|e| format!("Failed to open {path} file: {e}"))
}
//</editor-fold>

pub fn setup_database() -> Result<(), String> {
  // language=sqlite
  const TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS punishment (
    id              BLOB NOT NULL PRIMARY KEY CHECK(length(id) == 16),
    issued_to       INTEGER NOT NULL,
    issued_by       INTEGER NOT NULL,
    type            INTEGER NOT NULL,
    duration_name   TEXT DEFAULT NULL,
    duration_sec    INTEGER DEFAULT NULL,
    stale           INTEGER NOT NULL DEFAULT 0,
    permanent       INTEGER NOT NULL DEFAULT 0,
    time_sec        INTEGER NOT NULL,
    reason          TEXT DEFAULT NULL
) STRICT;";
  get_connection()?.execute(TABLE_SQL, ()).map_err(|_| String::from("Failed to create table `punishment`."))?;
  Ok(())
}

pub fn insert_punishment(
  issued_to: UserIdWrapper,
  issued_by: UserIdWrapper,
  punishment_type: PunishmentType,
  duration: Option<Duration>,
  reason: Option<String>,
) -> Result<Punishment, String> {
  // language=sqlite
  const INSERT_SQL: &str = "INSERT INTO punishment(id, issued_to, issued_by, type, duration_name, duration_sec, time_sec, reason)
VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) RETURNING *;";

  let conn = get_connection()?;
  let id = Uuid::new_v4();

  let mut prepared_stmt = conn.prepare(INSERT_SQL).map_err(|e| format!("Failed to prepare statement for insert: {e}"))?;
  let mut rows = prepared_stmt
    .query((
      id,
      issued_to,
      issued_by,
      punishment_type.index(),
      duration.clone().map(|d| d.display),
      duration.map_or_else(|| 0, |d| d.std_duration.as_secs() as i64),
      SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
      reason,
    ))
    .map_err(|e| format!("Failed to executed prepared insert statement: {e}"))?;

  let next: Option<&Row> = rows.next().map_err(|_| "err")?;
  let next_row: &Row = next.ok_or_else(|| "No row.")?;

  Ok(convert_to_struct(next_row))
}

fn convert_to_struct(row: &Row) -> Punishment {
  let duration_sec: i64 = row.get_unwrap("duration_sec");
  let time_sec_i64: i64 = row.get_unwrap("time_sec");

  let duration = if duration_sec == 0 {
    None
  } else {
    Some(Duration {
      display: row.get_unwrap("duration_name"),
      std_duration: std::time::Duration::from_secs(duration_sec as u64),
    })
  };

  let reason_str: String = row.get_unwrap("reason");
  let reason = if reason_str.is_empty() { None } else { Some(reason_str) };

  Punishment {
    punishment_id: row.get_unwrap("id"),
    issued_to: row.get_unwrap("issued_to"),
    issued_by: row.get_unwrap("issued_by"),
    punishment_type: PunishmentType::from_index(row.get_unwrap("type")),
    duration,
    stale: row.get_unwrap("stale"),
    permanent: row.get_unwrap("permanent"),
    time_sec: time_sec_i64 as u64,
    reason,
  }
}
