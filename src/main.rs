use poise::serenity_prelude::{CacheHttp, CreateEmbed, EventHandler, FullEvent};
use poise::{Framework, FrameworkOptions};
use poise::{async_trait, serenity_prelude as serenity};
use serenity::GatewayIntents;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {}

static TIMESTAMP_BOOT: OnceLock<SystemTime> = OnceLock::new();

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const BUILD_TIME_SEC: &str = env!("BUILD_TIME_SEC");

#[tokio::main]
async fn main() {
    let token = serenity::Token::from_env("BOT_TOKEN").expect(
        "No bot token provided. Please ensure the environment variable `BOT_TOKEN` is set.",
    );
    let intents = GatewayIntents::non_privileged();

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![about()],
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

#[poise::command(slash_command)]
async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let timestamp_millis = TIMESTAMP_BOOT
        .get()
        .unwrap()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    let url = "https://images-ext-1.discordapp.net/external/gqSkvZeXtYgqXI46ZxZIoZCgoQ4l0n79UhJqcmrBuLE/https/cdn.discordapp.com/avatars/1093034213999132743/4c1bb03fa385a3773c8c4c196b9c8fad.png?format=webp&quality=lossless";

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::new()
                .color(0xEB4968)
                .title("Sentinel (Off Brand)")
                .thumbnail(url)
                .field("Version", format!("`{GIT_HASH}` on `{GIT_BRANCH}`"), false)
                .field("Build Time", format!("<t:{BUILD_TIME_SEC}:F>"), false)
                .field("Boot Time", format!("<t:{timestamp_millis}:R>"), false),
        ),
    )
    .await?;
    Ok(())
}

struct SentinelEventHandler {
    has_registered_commands: AtomicBool,
}

#[async_trait]
impl EventHandler for SentinelEventHandler {
    async fn dispatch(&self, ctx: &serenity::Context, event: &FullEvent) {
        match event {
            FullEvent::Ready {
                data_about_bot: _, ..
            } => {
                async {
                    if self
                        .has_registered_commands
                        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                        .is_ok()
                    {
                        match poise::builtins::register_globally(ctx.http(), &vec![about()]).await {
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
