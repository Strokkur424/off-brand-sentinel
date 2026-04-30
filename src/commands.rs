use crate::TIMESTAMP_BOOT;
use crate::database::{Duration, PunishmentType};
use poise::CreateReply;
use poise::serenity_prelude::{CreateComponent, CreateContainer, CreateContainerComponent, CreateEmbed, CreateTextDisplay, Member, MessageFlags, Timestamp};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::wrapper::UserIdWrapper;

pub struct Data {}

#[derive(poise::ChoiceParameter)]
enum DurationChoices {
  #[name = "60 Seconds"]
  Dur60Seconds,
  #[name = "5 Minutes"]
  Dur5Minutes,
  #[name = "10 Minutes"]
  Dur10Minutes,
  #[name = "1 Hour"]
  Dur1Hour,
  #[name = "1 Day"]
  Dur1Day,
  #[name = "1 Week"]
  Dur1Week,
  #[name = "1 Month"]
  Dur1Month,
}

impl DurationChoices {
  fn to_duration(&self) -> Duration {
    match self {
      DurationChoices::Dur60Seconds => Duration::new("60 seconds", std::time::Duration::from_secs(60)),
      DurationChoices::Dur5Minutes => Duration::new("5 minutes", std::time::Duration::from_mins(5)),
      DurationChoices::Dur10Minutes => Duration::new("10 minutes", std::time::Duration::from_mins(10)),
      DurationChoices::Dur1Hour => Duration::new("1 hour", std::time::Duration::from_hours(1)),
      DurationChoices::Dur1Day => Duration::new("1 day", std::time::Duration::from_hours(1 * 24)),
      DurationChoices::Dur1Week => Duration::new("7 days", std::time::Duration::from_hours(1 * 24 * 7)),
      DurationChoices::Dur1Month => Duration::new("1 month", std::time::Duration::from_hours(1 * 24 * 30)),
    }
  }
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const BUILD_TIME_SEC: &str = env!("BUILD_TIME_SEC");

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
  vec![about(), timeout()]
}

#[poise::command(slash_command)]
async fn about(ctx: Context<'_>) -> Result<(), Error> {
  const THUMBNAIL_URL: &str = "https://images-ext-1.discordapp.net/external/gqSkvZeXtYgqXI46ZxZIoZCgoQ4l0n79UhJqcmrBuLE/\
  https/cdn.discordapp.com/avatars/1093034213999132743/4c1bb03fa385a3773c8c4c196b9c8fad.png?format=webp&quality=lossless";

  let timestamp_millis = TIMESTAMP_BOOT.get().unwrap().duration_since(UNIX_EPOCH)?.as_secs();

  ctx
    .send(
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

#[poise::command(slash_command, guild_only, required_permissions = "MODERATE_MEMBERS")]
async fn timeout(
  ctx: Context<'_>,
  #[description = "The member to timeout"] mut member: Member,
  #[description = "The reason for timing out the member"] reason: String,
  #[description = "The duration"] duration: DurationChoices,
) -> Result<(), Error> {
  let dur = duration.to_duration();
  let timestamp_unix = (SystemTime::now() + dur.std_duration).duration_since(UNIX_EPOCH)?.as_secs();

  let punishment = crate::database::insert_punishment(
    UserIdWrapper(member.user.id.get()),
    UserIdWrapper(ctx.author().id.get()),
    PunishmentType::TIMEOUT,
    Some(dur.clone()),
    Some(reason.clone()),
  )?;
  member
    .disable_communication_until(ctx.http(), Timestamp::from_unix_timestamp(timestamp_unix as i64)?)
    .await?;

  ctx
    .send(CreateReply::new().flags(MessageFlags::IS_COMPONENTS_V2).components(vec![CreateComponent::Container(
      CreateContainer::new(vec![
        CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!("### Punishment {}", punishment.punishment_id.as_simple()))),
        CreateContainerComponent::TextDisplay(CreateTextDisplay::new("**Type**: <:dot_purple:1499540059672805546> timeout")),
        CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!("**Issued to**: <@{}> (`@{}` / `{}`)", member.user.id.get(), member.user.name, member.user.id.get()))),
        CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!("**Reason**: {}", reason))),
        CreateContainerComponent::TextDisplay(CreateTextDisplay::new(format!("**Duration**: ~{} (expiry: <t:{}:F>", dur.display, timestamp_unix))),
      ])
        .accent_colour(0x9A59B4),
    )]))
    .await?;

  Ok(())
}
