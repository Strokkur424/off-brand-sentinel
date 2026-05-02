use crate::database::{Duration, PunishmentType};
use crate::punishments::{execute_ban, execute_kick};
use crate::wrapper::UserIdWrapper;
use crate::{TIMESTAMP_BOOT, modals, punishments};
use poise::CreateReply;
use poise::serenity_prelude::{CreateEmbed, Member, Message, MessageFlags, Timestamp};
use punishments::send_messages;
use std::time::UNIX_EPOCH;

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

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const BUILD_TIME_SEC: &str = env!("BUILD_TIME_SEC");

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
  vec![about(), timeout(), ban(), kick(), warn(), note(), ban_context(), kick_context(), quick_ban_context()]
}

/// Displays information about Sentinel
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

/// Timeout a member
#[poise::command(slash_command, guild_only, required_permissions = "MODERATE_MEMBERS")]
async fn timeout(
  ctx: Context<'_>,
  #[description = "The member to timeout"] mut member: Member,
  #[description = "The reason for timing out the member"] reason: String,
  #[description = "The duration"] duration: DurationChoices,
) -> Result<(), Error> {
  let dur = duration.to_duration();
  let timestamp_unix = dur.to_unix_time_from_now();

  let punishment = crate::database::insert_punishment(
    UserIdWrapper(member.user.id.get()),
    UserIdWrapper(ctx.author().id.get()),
    PunishmentType::TIMEOUT,
    Some(dur),
    Some(reason),
  )?;
  member
    .disable_communication_until(ctx.http(), Timestamp::from_unix_timestamp(timestamp_unix as i64)?)
    .await?;

  send_messages(&ctx, &punishment, &member.user).await?;
  Ok(())
}

/// Ban a member
#[poise::command(slash_command, guild_only, required_permissions = "BAN_MEMBERS")]
async fn ban(
  ctx: Context<'_>,
  #[description = "The member to ban"] member: Member,
  #[description = "The reason for banning the member"] reason: Option<String>,
  #[description = "If recent messages should be deleted - defaults to true"] delete_messages: Option<bool>,
) -> Result<(), Error> {
  execute_ban(&ctx, &member.user, delete_messages.unwrap_or(true), reason).await?;
  Ok(())
}

/// Kick a member
#[poise::command(slash_command, guild_only, required_permissions = "KICK_MEMBERS")]
async fn kick(
  ctx: Context<'_>,
  #[description = "The member to kick"] member: Member,
  #[description = "The reason for kicking the member"] reason: Option<String>,
) -> Result<(), Error> {
  execute_kick(&ctx, &member.user, reason).await?;
  Ok(())
}

/// Warn a member
#[poise::command(slash_command, guild_only, required_permissions = "MODERATE_MEMBERS")]
async fn warn(ctx: Context<'_>, #[description = "The member to warn"] member: Member, #[description = "The reason for warning the member"] reason: String) -> Result<(), Error> {
  let punishment = crate::database::insert_punishment(
    UserIdWrapper(member.user.id.get()),
    UserIdWrapper(ctx.author().id.get()),
    PunishmentType::WARN,
    None,
    Some(reason),
  )?;

  send_messages(&ctx, &punishment, &member.user).await?;
  Ok(())
}

/// Add a note to a member
#[poise::command(slash_command, guild_only, required_permissions = "MODERATE_MEMBERS")]
async fn note(ctx: Context<'_>, #[description = "The member to add a note to"] member: Member, #[description = "The note content"] reason: String) -> Result<(), Error> {
  let punishment = crate::database::insert_punishment(
    UserIdWrapper(member.user.id.get()),
    UserIdWrapper(ctx.author().id.get()),
    PunishmentType::NOTE,
    None,
    Some(reason),
  )?;

  let (component, _) = punishments::get_messages(&ctx, &punishment, &member.user)?;
  ctx.send(CreateReply::new().flags(MessageFlags::IS_COMPONENTS_V2).components(vec![component])).await?;
  Ok(())
}

#[poise::command(context_menu_command = "Ban", guild_only, required_permissions = "BAN_MEMBERS")]
async fn ban_context(ctx: poise::ApplicationContext<'_, Data, Error>, msg: Message) -> Result<(), Error> {
  let name = msg.author.clone().name;
  let id = format!("ban_{}", name);
  let title = format!("Ban {}", name);

  let reason = modals::send_reason_modal(&ctx, title, id).await?;
  if let Some(reason) = reason {
    let ctx = Context::from(ctx);
    execute_ban(&ctx, &msg.author, true, Some(reason)).await?;
  }

  Ok(())
}

#[poise::command(context_menu_command = "Kick", guild_only, required_permissions = "KICK_MEMBERS")]
async fn kick_context(ctx: poise::ApplicationContext<'_, Data, Error>, msg: Message) -> Result<(), Error> {
  let name = msg.author.clone().name;
  let id = format!("kick_{}", name);
  let title = format!("Kick {}", name);

  let reason = modals::send_reason_modal(&ctx, title, id).await?;
  if let Some(reason) = reason {
    let ctx = Context::from(ctx);
    execute_kick(&ctx, &msg.author, Some(reason)).await?;
  }

  Ok(())
}

#[poise::command(context_menu_command = "Quick Ban", guild_only, required_permissions = "BAN_MEMBERS")]
async fn quick_ban_context(ctx: Context<'_>, msg: Message) -> Result<(), Error> {
  execute_ban(&ctx, &msg.author, true, Some(format!("Quick-banned for sending a message in <#{}>", msg.channel_id))).await?;
  Ok(())
}
