use crate::database;
use crate::database::get_all_factoids;
use poise::serenity_prelude::{AutocompleteChoice, CreateAutocompleteResponse};
use sentinel_common::wrapper::GuildIdWrapper;
use sentinel_common::{Error, FactoidData};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

#[derive(Eq, PartialEq, Hash)]
struct CacheKey {
  pub guild: GuildIdWrapper,
  pub name: String,
}

static CACHE: RwLock<LazyLock<HashMap<CacheKey, FactoidData>>> = RwLock::new(LazyLock::new(|| HashMap::new()));

pub fn load_from_database() -> Result<(), Error> {
  let mut write = CACHE.write().map_err(|_| "Failed to acquire WRITE lock.")?;

  write.clear();
  for (guild, data) in get_all_factoids()? {
    write.insert(
      CacheKey {
        guild,
        name: data.factoid_name.clone(),
      },
      data,
    );
  }

  Ok(())
}

pub fn is_name_taken(guild: GuildIdWrapper, name: &String) -> bool {
  let read = CACHE.read().unwrap();
  read.iter().any(|(k, _)| k.guild == guild && k.name == *name)
}

pub fn suggest_factoids_for<'a>(guild: GuildIdWrapper) -> CreateAutocompleteResponse<'a> {
  let read = CACHE.read().unwrap();
  let choices: Vec<AutocompleteChoice> = read
    .iter()
    .filter(|(k, _)| k.guild == guild)
    .map(|(_, v)| AutocompleteChoice::new(v.factoid_name.clone(), v.factoid_name.clone()))
    .collect();

  CreateAutocompleteResponse::new().set_choices(choices)
}

pub fn create_new_factoid(guild: GuildIdWrapper, factoid: FactoidData) -> Result<(), Error> {
  let mut write = CACHE.write().map_err(|_| "Failed to acquire WRITE lock.")?;
  database::insert_factoid(&guild, factoid.clone())?;
  write.insert(
    CacheKey {
      guild,
      name: factoid.factoid_name.clone(),
    },
    factoid,
  );
  Ok(())
}

pub fn update_factoid(guild: GuildIdWrapper, factoid: String, updated: FactoidData) -> Result<(), Error> {
  let mut write = CACHE.write().map_err(|_| "Failed to acquire WRITE lock.")?;
  database::update_factoid(&guild, &factoid, updated.clone())?;

  write.remove(&CacheKey {
    guild: guild.clone(),
    name: factoid,
  });
  write.insert(
    CacheKey {
      guild,
      name: updated.factoid_name.clone(),
    },
    updated,
  );
  Ok(())
}

pub fn get_factoid(guild: GuildIdWrapper, name: String) -> Option<FactoidData> {
  let read = CACHE.read().unwrap();
  let key = CacheKey { guild, name };

  match read.get(&key) {
    Some(v) => Some(v.clone()),
    None => None,
  }
}

pub fn get_factoid_by_display_name(guild: GuildIdWrapper, name: String) -> Option<FactoidData> {
  let factoids = get_factoids_for_guild(guild);
  for factoid in factoids {
    if (factoid.display_name == name) {
      return Some(factoid);
    }
  }
  None
}

pub fn get_factoids_for_guild(guild: GuildIdWrapper) -> Vec<FactoidData> {
  let read = CACHE.read().unwrap();
  read
    .iter()
    .filter(|(k, _)| k.guild == guild)
    .map(|(_, v)| v.clone())
    .collect()
}

pub fn delete_factoid(guild: GuildIdWrapper, name: String) -> Result<(), Error> {
  database::delete_factoid(&guild, &name)?;

  let mut write = CACHE.write().map_err(|_| "Failed to acquire WRITE lock.")?;
  let key = CacheKey { guild, name };
  write.remove(&key);
  Ok(())
}
