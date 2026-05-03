use futures_util::future::BoxFuture;
use poise::futures_util::future::join_all;
use poise::serenity_prelude::{CacheHttp, EventHandler, FullEvent, GatewayIntents};
use poise::{async_trait, serenity_prelude as serenity, Framework, FrameworkOptions};
use sentinel_common::{config, Error};
use sentinel_factoids::run_factoids;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

mod commands;
mod database;
mod punishments;

static TIMESTAMP_BOOT: OnceLock<SystemTime> = OnceLock::new();
pub(crate) static WORKING_DIRECTORY: OnceLock<String> = OnceLock::new();

pub async fn run(working_dir: Option<String>) -> Result<(), Error> {
  let working_dir = working_dir.unwrap_or("./".to_string());
  WORKING_DIRECTORY.set(working_dir.clone()).expect("This shouldn't happen (SET WORKING DIR)");

  config::load_config(working_dir.as_str())?;
  let config = config::get_config_certain()?;

  let mut tasks = Vec::<BoxFuture<'_, Result<(), Error>>>::new();
  if !config.tokens.sentinel.is_empty() {
    if !config.tokens.factoids.is_empty() {
      println!("Starting Factoids.");
      tasks.push(Box::pin(run_factoids(&working_dir, &config.tokens.factoids)));
      sentinel_factoids::set_initialised(true);
    }
    println!("Starting Sentinel.");
    tasks.push(Box::pin(run_sentinel(&config.tokens.sentinel)));
  }

  join_all(tasks).await;
  Ok(())
}

async fn run_sentinel(token: &String) -> Result<(), Error> {
  let token = token.as_str().parse().map_err(|_| "Invalid token defined for sentinel.")?;

  let framework = Framework::builder()
    .options(FrameworkOptions {
      commands: commands::get_commands(),
      ..Default::default()
    })
    .build();
  let intents = GatewayIntents::non_privileged();

  let client = serenity::ClientBuilder::new(token, intents)
    .framework(Box::new(framework))
    .event_handler(Arc::new(SentinelEventHandler {
      has_registered_commands: AtomicBool::new(false),
    }))
    .await;

  database::setup_database()?;
  TIMESTAMP_BOOT.set(SystemTime::now()).ok();
  client?.start().await?;

  Ok(())
}

struct SentinelEventHandler {
  has_registered_commands: AtomicBool,
}

#[async_trait]
impl EventHandler for SentinelEventHandler {
  async fn dispatch(&self, ctx: &serenity::Context, event: &FullEvent) {
    match event {
      FullEvent::Ready { data_about_bot: _, .. } => {
        async {
          if self.has_registered_commands.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            match poise::builtins::register_globally(ctx.http(), &commands::get_commands()).await {
              Ok(()) => println!("[Sentinel] Successfully registered commands."),
              Err(error) => println!("[Sentinel] Failed to register commands: {error:?}"),
            }
          }
        }
        .await
      }
      _ => {}
    }
  }
}
