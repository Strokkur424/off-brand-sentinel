use poise::serenity_prelude::{GuildId, UserId};
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use rusqlite::ToSql;

pub struct UserIdWrapper(pub u64);

#[derive(Clone)]
pub struct GuildIdWrapper(pub u64);

impl UserIdWrapper {
  pub fn unwrap(self) -> UserId {
    UserId::new(self.0)
  }
}

impl GuildIdWrapper {
  pub fn unwrap(self) -> GuildId {
    GuildId::new(self.0)
  }
}

impl FromSql for UserIdWrapper {
  fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
    Ok(Self(i64::column_result(value)? as u64))
  }
}

impl ToSql for UserIdWrapper {
  fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
    Ok(ToSqlOutput::from(self.0.cast_signed()))
  }
}

impl FromSql for GuildIdWrapper {
  fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
    Ok(Self(i64::column_result(value)? as u64))
  }
}

impl ToSql for GuildIdWrapper {
  fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
    Ok(ToSqlOutput::from(self.0.cast_signed()))
  }
}
