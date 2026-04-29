use poise::serenity_prelude::{
    CacheHttp, CreateComponent, CreateContainer, CreateContainerComponent, CreateSection,
    CreateSectionAccessory, CreateSectionComponent, CreateTextDisplay, CreateThumbnail,
    CreateUnfurledMediaItem, EventHandler, FullEvent,
};
use poise::{Framework, FrameworkOptions};
use poise::{async_trait, serenity_prelude as serenity};
use serenity::GatewayIntents;
use serenity::all::MessageFlags;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {}

static TIMESTAMP_BOOT: OnceLock<SystemTime> = OnceLock::new();

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
    let content = format!(
        "### Sentinel
**Version**
`0000` on `main`

**Build Time**
Now

**Boot Time**
<t:{timestamp_millis}:R>"
    );

    let url = "https://images-ext-1.discordapp.net/external/gqSkvZeXtYgqXI46ZxZIoZCgoQ4l0n79UhJqcmrBuLE/https/cdn.discordapp.com/avatars/1093034213999132743/4c1bb03fa385a3773c8c4c196b9c8fad.png?format=webp&quality=lossless";
    let section = CreateSection::new(
        vec![CreateSectionComponent::TextDisplay(CreateTextDisplay::new(
            content,
        ))],
        CreateSectionAccessory::Thumbnail(CreateThumbnail::new(CreateUnfurledMediaItem::new(url))),
    );

    ctx.send(
        poise::CreateReply::default()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(vec![CreateComponent::Container(
                CreateContainer::new(vec![CreateContainerComponent::Section(section)])
                    .accent_color(0xFFAA00),
            )]),
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
