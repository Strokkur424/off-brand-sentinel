mod commands;
mod database;
mod punishments;
mod wrapper;
mod modals;

use poise::serenity_prelude::{CacheHttp, EventHandler, FullEvent};
use poise::{Framework, FrameworkOptions};
use poise::{async_trait, serenity_prelude as serenity};
use serenity::GatewayIntents;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

static TIMESTAMP_BOOT: OnceLock<SystemTime> = OnceLock::new();

#[tokio::main]
async fn main() {
  if let Err(msg) = database::setup_database() {
    eprintln!("Failed to setup database: {msg}");
    return;
  }

  let token = serenity::Token::from_env("BOT_TOKEN").expect("No bot token provided. Please ensure the environment variable `BOT_TOKEN` is set.");
  let intents = GatewayIntents::non_privileged();

  let framework = Framework::builder()
    .options(FrameworkOptions {
      commands: commands::get_commands(),
      ..Default::default()
    })
    .build();

  let client = serenity::ClientBuilder::new(token, intents)
    .framework(Box::new(framework))
    .event_handler(Arc::new(SentinelEventHandler {
      has_registered_commands: AtomicBool::new(false),
    }))
    .await;

  TIMESTAMP_BOOT.set(SystemTime::now()).ok();
  client.unwrap().start().await.unwrap();
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
