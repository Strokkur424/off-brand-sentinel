use crate::factoids::FactoidData;
use rusqlite::Connection;
use sentinel_common::wrapper::GuildIdWrapper;
use sentinel_common::Error;
use std::fs;

fn get_connection() -> Result<Connection, Error> {
  let dir: &String = crate::get_working_dir()?;
  fs::create_dir_all(&dir)?;

  let path = format!("{dir}database.sqlite");
  Ok(Connection::open(path.clone())?)
}

pub(crate) fn create_table() -> Result<(), Error> {
  // language=sqlite
  const CREATE_TABLE: &str = "
CREATE TABLE IF NOT EXISTS factoids (
    guild_id        INTEGER NOT NULL,
    factoid_name    TEXT NOT NULL,
    display_name    TEXT NOT NULL,
    description     TEXT,
    components      TEXT,
    PRIMARY KEY(guild_id, factoid_name)
) STRICT;";
  let conn = get_connection()?;
  conn.execute(CREATE_TABLE, ())?;
  Ok(())
}

pub(crate) fn insert_factoid(guild_id: &GuildIdWrapper, factoid: FactoidData) -> Result<(), Error> {
  // language=sqlite
  const INSERT: &str =
    "INSERT INTO factoids (guild_id, factoid_name, display_name, description, components) VALUES (?, ?, ?, ?, ?);";

  let conn = get_connection()?;
  let mut prepared = conn.prepare(INSERT)?;
  prepared.insert((
    guild_id,
    factoid.factoid_name,
    factoid.display_name,
    factoid.description,
    factoid.components,
  ))?;
  Ok(())
}

pub(crate) fn get_all_factoids() -> Result<Vec<(GuildIdWrapper, FactoidData)>, Error> {
  // language=sqlite
  const SELECT: &str = "SELECT display_name, factoid_name, description, components, guild_id FROM factoids;";

  let conn = get_connection()?;
  let mut prepared = conn.prepare(SELECT)?;
  let mut rows = prepared.query(())?;

  let mut out: Vec<(GuildIdWrapper, FactoidData)> = Vec::new();
  while let Some(row) = rows.next()? {
    out.push((
      row.get_unwrap("guild_id"),
      FactoidData {
        display_name: row.get_unwrap("display_name"),
        factoid_name: row.get_unwrap("factoid_name"),
        description: row.get_unwrap("description"),
        components: row.get_unwrap("components"),
      },
    ));
  }

  Ok(out)
}

pub(crate) fn update_factoid(
  guild_id: &GuildIdWrapper,
  factoid_name: &String,
  new_data: FactoidData,
) -> Result<(), Error> {
  // language=sqlite
  const UPDATE: &str = "UPDATE factoids SET factoid_name = ?, description = ?, display_name = ?, components = ?
WHERE factoid_name = ? AND guild_id = ?;";

  let conn = get_connection()?;
  let mut prepared = conn.prepare(UPDATE)?;
  prepared.execute((
    new_data.factoid_name,
    new_data.description,
    new_data.display_name,
    new_data.components,
    factoid_name,
    guild_id,
  ))?;

  Ok(())
}

pub(crate) fn delete_factoid(guild_id: &GuildIdWrapper, name: &String) -> Result<(), Error> {
  // language=sqlite
  const DELETE: &str = "DELETE FROM factoids WHERE factoid_name = ? AND guild_id = ?;";
  let conn = get_connection()?;
  let mut prepared = conn.prepare(DELETE)?;
  prepared.execute((name, guild_id))?;
  Ok(())
}
