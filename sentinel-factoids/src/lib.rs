use crate::context::FactoidContext;
use poise::async_trait;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CommandType, EventHandler, FullEvent, GatewayIntents, GuildId, Http, Interaction, Message};
use sentinel_common::{Data, Error};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

pub mod commands;
mod context;
mod database;
mod factoids;
mod modals;
mod util;

static INITIALISED: AtomicBool = AtomicBool::new(false);
static WORKING_DIRECTORY: OnceLock<String> = OnceLock::new();
static HTTP_CONTEXT: OnceLock<Arc<Http>> = OnceLock::new();

pub async fn run_factoids(working_dir: &String, token: &String) -> Result<(), Error> {
  WORKING_DIRECTORY.set((*working_dir).clone())?;

  let intents = GatewayIntents::non_privileged();

  let token = token
    .as_str()
    .parse()
    .map_err(|_| "Invalid token defined for sentinel.")?;
  let mut client = serenity::ClientBuilder::new(token, intents)
    .event_handler(Arc::new(FactoidEventHandler {
      has_registered_commands: AtomicBool::new(false),
    }))
    .await?;

  database::create_table()?;
  factoids::load_from_database()?;
  client.start().await?;

  Ok(())
}

pub fn is_initialised() -> bool {
  INITIALISED.load(Ordering::SeqCst)
}

pub fn set_initialised(val: bool) {
  INITIALISED.store(val, Ordering::SeqCst)
}

pub(crate) fn get_working_dir() -> Result<&'static String, Error> {
  WORKING_DIRECTORY
    .get()
    .ok_or_else(|| Error::from("No working directory set."))
}

struct FactoidEventHandler {
  has_registered_commands: AtomicBool,
}

pub async fn update_factoid_commands(guild: GuildId) -> Result<(), Error> {
  let commands = poise::builtins::create_application_commands(&commands::get_factoid_commands(guild));
  guild.set_commands(HTTP_CONTEXT.get().unwrap(), &commands).await?;
  Ok(())
}

#[async_trait]
impl EventHandler for FactoidEventHandler {
  async fn dispatch(&self, ctx: &serenity::Context, event: &FullEvent) {
    match event {
      FullEvent::Ready { data_about_bot: _, .. } => {
        async {
          if self
            .has_registered_commands
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
          {
            let _ = HTTP_CONTEXT.set(ctx.http.clone());
            for guild in ctx.cache.guilds() {
              match update_factoid_commands(guild).await {
                Ok(_) => println!("[Factoids] Successfully registered commands for guild with id <{guild}>."),
                Err(error) => println!("[Factoids] Failed to register commands for guild with id <{guild}>: {error:?}"),
              }
            }
          }
        }
        .await
      }
      FullEvent::InteractionCreate { interaction, .. } => {
        if let Interaction::Command(cmd) = interaction {
          let guild_id = cmd.guild_id;
          if guild_id.is_none() {
            return;
          }

          let name = cmd.data.name.clone();
          let kind = cmd.data.kind.clone();
          let messages = cmd.data.resolved.messages.clone();

          let guild_id = guild_id.unwrap();
          let context = FactoidContext { ctx, interaction: cmd };

          if kind == CommandType::ChatInput {
            let factoid = factoids::get_factoid(guild_id.into(), name.into_string());
            if let Some(factoid) = factoid {
              if let Err(e) = context.respond_manually_components(factoid.components).await {
                println!("Something went awfully wrong: {e}")
              }
            }
          } else if kind == CommandType::Message {
            let result: Vec<Message> = messages.into_iter().collect();

            let factoid = factoids::get_factoid_by_display_name(guild_id.into(), name.into_string());
            if let Some(factoid) = factoid {
              if let Err(e) = context
                .reply_to_message(result.first().unwrap(), factoid.components)
                .await
              {
                println!("Something went awfully wrong: {e}")
              }
              let _ = context
                .send_plain(&"<:yes:1500417686981840957> Success!".to_string(), true)
                .await;
            }
          }
        }
      }
      _ => {}
    }
  }
}
