use crate::factoids::FactoidData;
use crate::{factoids, modals, util};
use poise::serenity_prelude::CreateAutocompleteResponse;
use poise::ApplicationContext;
use regex::Regex;
use sentinel_common::wrapper::GuildIdWrapper;
use sentinel_common::{Context, Data, Error};
use serde_json::Value;
use std::str::FromStr;
use std::sync::LazyLock;

static REGEX_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9_-]+$").unwrap());
static REGEX_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9 ]+$").unwrap());

macro_rules! validate_and_minimize_json {
  ( $source:expr, $ctx:expr ) => {
    match Value::from_str($source) {
      Ok(v) => format!("{}", v),
      Err(err) => {
        $ctx
          .say(format!(
            "<:no:1500220802841182211> Failed to parse components JSON: {err}"
          ))
          .await?;
        return Ok(());
      }
    }
  };
}

macro_rules! format_json {
  ( $source:expr ) => {
    format!("{:#}", Value::from_str($source)?)
  };
}

pub fn get_sentinel_commands() -> Vec<poise::Command<Data, Error>> {
  vec![factoid_root()]
}

pub(crate) fn get_factoid_commands() -> Vec<poise::Command<Data, Error>> {
  vec![]
}

#[poise::command(
  slash_command,
  guild_only,
  subcommand_required,
  subcommands("factoid_add", "factoid_remove", "factoid_update", "factoid_source"),
  required_permissions = "MODERATE_MEMBERS",
  rename = "factoid"
)]
async fn factoid_root(_: Context<'_>) -> Result<(), Error> {
  Ok(()) // never called
}

/// Adds a new factoid
#[poise::command(slash_command, rename = "add", ephemeral)]
async fn factoid_add(
  ctx: ApplicationContext<'_, Data, Error>,
  #[description = "The identifier of the factoid."] id: String,
  #[description = "The display name of the factoid."] name: String,
) -> Result<(), Error> {
  if !REGEX_ID.is_match(id.as_str()) {
    ctx
      .say("<:no:1500220802841182211> Invalid id. Please only use lower alphanumerical and `-` or `_` characters.")
      .await?;
    return Ok(());
  }
  if !REGEX_NAME.is_match(name.as_str()) {
    ctx
      .say("<:no:1500220802841182211> Invalid name. Please only use alphanumerical characters and ` `.")
      .await?;
    return Ok(());
  }

  if factoids::is_name_taken(ctx.guild_id().unwrap().into(), &id) {
    ctx
      .say("<:no:1500220802841182211> A factoid with this name already exists.")
      .await?;
    return Ok(());
  }

  let factoid_create_data =
    modals::send_factoid_create_modal(&ctx, format!("Create a new factoid: {}", name), id.clone()).await?;

  if factoid_create_data.is_none() {
    ctx.say("<:no:1500220802841182211> Something went wrong.").await?;
    return Ok(());
  }

  let factoid_create_data = factoid_create_data.unwrap();
  let components_json = validate_and_minimize_json!(factoid_create_data.components.as_str(), ctx);

  util::respond_manually_components(ctx, components_json.clone()).await?;
  factoids::create_new_factoid(
    ctx.guild_id().unwrap().into(),
    FactoidData {
      display_name: name,
      factoid_name: id,
      description: factoid_create_data.description,
      components: components_json,
    },
  )?;
  Ok(())
}

/// Removes a factoid
#[poise::command(slash_command, rename = "remove", ephemeral)]
async fn factoid_remove(
  ctx: Context<'_>,
  #[description = "The identifier of the factoid."]
  #[autocomplete = "autocomplete_factoid_id"]
  id: String,
) -> Result<(), Error> {
  if !factoids::is_name_taken(ctx.guild_id().unwrap().into(), &id) {
    ctx.say("<:no:1500220802841182211> No factoid found.").await?;
    return Ok(());
  }

  factoids::delete_factoid(ctx.guild_id().unwrap().into(), id.clone())?;
  ctx
    .say(format!("<:yes:1500220801108934786> Successfully removed `{}`.", id))
    .await?;
  Ok(())
}

/// Retrieves a factoid's source
#[poise::command(slash_command, rename = "source", ephemeral)]
async fn factoid_source(
  ctx: Context<'_>,
  #[description = "The identifier of the factoid."]
  #[autocomplete = "autocomplete_factoid_id"]
  id: String,
) -> Result<(), Error> {
  if !factoids::is_name_taken(ctx.guild_id().unwrap().into(), &id) {
    ctx.say("<:no:1500220802841182211> No factoid found.").await?;
    return Ok(());
  }

  if let Some(val) = factoids::get_factoid(ctx.guild_id().unwrap().into(), id.clone()) {
    ctx
      .say(
        format!(
          "<:yes:1500220801108934786> Components (JSON):
```json
{}
```",
          format_json!(val.components.as_str())
        )
        .as_str(),
      )
      .await?;
  }
  Ok(())
}

/// Updates an existing factoid
#[poise::command(slash_command, rename = "update")]
async fn factoid_update(
  ctx: ApplicationContext<'_, Data, Error>,
  #[description = "The identifier of the factoid."]
  #[autocomplete = "autocomplete_factoid_id"]
  id: String,
) -> Result<(), Error> {
  if !REGEX_ID.is_match(id.as_str()) {
    ctx
      .say("<:no:1500220802841182211> Invalid id. Please only use lower alphanumerical and `-` or `_` characters.")
      .await?;
    return Ok(());
  }

  let guild = GuildIdWrapper(ctx.guild_id().unwrap().get());

  let factoid_data = factoids::get_factoid(guild.clone(), id.clone());
  if factoid_data.is_none() {
    ctx.say("<:no:1500220802841182211> No factoid found.").await?;
    return Ok(());
  }
  let factoid_data = factoid_data.unwrap();

  let new_data = modals::send_factoid_edit_modal(&ctx, factoid_data).await?;
  if let Some(data) = new_data {
    let components_json = validate_and_minimize_json!(data.components.as_str(), ctx);
    factoids::update_factoid(guild, id, data.with_components(components_json.clone()))?;
    util::respond_manually_components(ctx, components_json).await?;
    return Ok(());
  }

  ctx.say("<:no:1500220802841182211> No response sent.").await?;
  Ok(())
}

async fn autocomplete_factoid_id<'a>(ctx: Context<'_>, _partial: &str) -> CreateAutocompleteResponse<'a> {
  factoids::suggest_factoids_for(ctx.guild_id().unwrap().into())
}
