use crate::config::Configuration;
use poise::futures_util::future::join_all;
use poise::serenity_prelude::{CacheHttp, EventHandler, FullEvent, GatewayIntents};
use poise::{async_trait, serenity_prelude as serenity, Framework, FrameworkOptions};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

mod commands;
mod config;
mod database;
mod punishments;

static TIMESTAMP_BOOT: OnceLock<SystemTime> = OnceLock::new();
pub(crate) static WORKING_DIRECTORY: OnceLock<String> = OnceLock::new();
pub(crate) static CONFIG: OnceLock<Configuration> = OnceLock::new();

pub async fn run(working_dir: Option<String>) {
  let working_dir = working_dir.unwrap_or("./".to_string());
  WORKING_DIRECTORY.set(working_dir.clone()).expect("This shouldn't happen (SET WORKING DIR)");

  if let Err(msg) = database::setup_database() {
    eprintln!("Failed to setup database: {msg}");
    return;
  }

  let config = config::load_config(working_dir.as_str());
  if let Err(msg) = config {
    eprintln!("Failed to load config: {msg}");
    return;
  }
  CONFIG.set(config.unwrap()).expect("This shouldn't happen (SET CONFIG)");
  let config = CONFIG.get().expect("This shouldn't happen (GET CONFIG)");

  let mut tasks = Vec::new();

  if !config.tokens.sentinel.is_empty() {
    println!("Starting Sentinel.");
    tasks.push(init_sentinel(&config.tokens.sentinel));
  } else {
    println!("Did not start Sentinel bot: No token provided.");
  }

  if !config.tokens.factoids.is_empty() {
    println!("Starting Factoids (noop).");
  } else {
    println!("Did not start Factoids bot: No token provided.")
  }

  join_all(tasks).await;
}

async fn init_sentinel(token: &String) -> Result<(), &str> {
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

  TIMESTAMP_BOOT.set(SystemTime::now()).ok();
  client.unwrap().start().await.unwrap();

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
              Ok(()) => println!("Successfully registered commands."),
              Err(error) => println!("Failed to register commands: {error:?}"),
            }
          }
        }
        .await
      }
      _ => {}
    }
  }
}
