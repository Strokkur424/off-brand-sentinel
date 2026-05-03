use poise::serenity_prelude::{GuildId, UserId};
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use rusqlite::ToSql;

pub struct UserIdWrapper(pub u64);

#[derive(Clone, Eq, PartialEq, Hash)]
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

impl Into<UserIdWrapper> for UserId {
  fn into(self) -> UserIdWrapper {
    UserIdWrapper(self.get())
  }
}

impl From<UserIdWrapper> for UserId {
  fn from(value: UserIdWrapper) -> Self {
    UserId::new(value.0)
  }
}

impl Into<GuildIdWrapper> for GuildId {
  fn into(self) -> GuildIdWrapper {
    GuildIdWrapper(self.get())
  }
}

impl From<GuildIdWrapper> for GuildId {
  fn from(value: GuildIdWrapper) -> Self {
    GuildId::new(value.0)
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
