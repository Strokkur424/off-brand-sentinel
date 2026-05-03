use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use poise::{async_trait, FrameworkOptions};
use poise::serenity_prelude::{CacheHttp, EventHandler, FullEvent, GatewayIntents};
use sentinel_common::Error;
use poise::serenity_prelude as serenity;

pub mod commands;
mod database;
mod modals;
mod util;
mod factoids;

static INITIALISED: AtomicBool = AtomicBool::new(false);
static WORKING_DIRECTORY: OnceLock<String> = OnceLock::new();

pub async fn run_factoids(working_dir: &String, token: &String) -> Result<(), Error> {
  WORKING_DIRECTORY.set((*working_dir).clone())?;

  let framework = poise::Framework::builder()
    .options(FrameworkOptions {
      commands: commands::get_factoid_commands(),
      ..Default::default()
    })
    .build();
  let intents = GatewayIntents::non_privileged();

  let token = token.as_str().parse().map_err(|_| "Invalid token defined for sentinel.")?;
  let mut client = serenity::ClientBuilder::new(token, intents)
    .framework(Box::new(framework))
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
  WORKING_DIRECTORY.get().ok_or_else(|| Error::from("No working directory set."))
}

struct FactoidEventHandler {
  has_registered_commands: AtomicBool,
}

#[async_trait]
impl EventHandler for FactoidEventHandler {
  async fn dispatch(&self, ctx: &serenity::Context, event: &FullEvent) {
    match event {
      FullEvent::Ready { data_about_bot: _, .. } => {
        async {
          if self.has_registered_commands.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            match poise::builtins::register_globally(ctx.http(), &commands::get_factoid_commands()).await {
              Ok(()) => println!("[Factoids] Successfully registered commands."),
              Err(error) => println!("[Factoids] Failed to register commands: {error:?}"),
            }
          }
        }
          .await
      }
      _ => {}
    }
  }
}

