use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use rusqlite::ToSql;

pub struct UserIdWrapper(pub u64);

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
