use crate::TIMESTAMP_BOOT;
use poise::serenity_prelude::CreateEmbed;
use std::time::UNIX_EPOCH;

pub struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const BUILD_TIME_SEC: &str = env!("BUILD_TIME_SEC");

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![about()]
}

#[poise::command(slash_command)]
async fn about(ctx: Context<'_>) -> Result<(), Error> {
    const THUMBNAIL_URL: &str = "https://images-ext-1.discordapp.net/external/gqSkvZeXtYgqXI46ZxZIoZCgoQ4l0n79UhJqcmrBuLE/https/cdn.discordapp.com/avatars/1093034213999132743/4c1bb03fa385a3773c8c4c196b9c8fad.png?format=webp&quality=lossless";

    let timestamp_millis = TIMESTAMP_BOOT
        .get()
        .unwrap()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::new()
                .color(0xEB4968)
                .title("Sentinel (Off Brand)")
                .thumbnail(THUMBNAIL_URL)
                .field("Version", format!("`{GIT_HASH}` on `{GIT_BRANCH}`"), false)
                .field("Build Time", format!("<t:{BUILD_TIME_SEC}:F>"), false)
                .field("Boot Time", format!("<t:{timestamp_millis}:R>"), false),
        ),
    )
    .await?;
    Ok(())
}
