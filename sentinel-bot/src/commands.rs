use crate::database::{Duration, PartialPunishment, Punishment, PunishmentType};
use crate::punishments::{execute_ban, execute_kick, PunishmentDisplay};
use crate::{database, punishments, CONFIG, TIMESTAMP_BOOT};
use poise::serenity_prelude::{
  CreateAllowedMentions, CreateComponent, CreateContainer, CreateContainerComponent, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, CreateSection,
  CreateSectionAccessory, CreateSectionComponent, CreateSeparator, CreateTextDisplay, CreateThumbnail, CreateUnfurledMediaItem, GenericChannelId, Member, Message, MessageFlags,
  Timestamp, User, UserId,
};
use poise::CreateReply;
use punishments::send_messages;
use sentinel_common::wrapper::{GuildIdWrapper, UserIdWrapper};
use sentinel_common::{modals, Context, Data, Error};
use std::time::UNIX_EPOCH;
use uuid::Uuid;

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

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const BUILD_TIME_SEC: &str = env!("BUILD_TIME_SEC");

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
  vec![
    about(),
    timeout(),
    ban(),
    kick(),
    warn(),
    note(),
    ban_context(),
    kick_context(),
    quick_ban_context(),
    report_context(),
    modmail(),
    punishment(),
  ]
}

/// Displays information about Sentinel
#[poise::command(slash_command)]
async fn about(ctx: Context<'_>) -> Result<(), Error> {
  const THUMBNAIL_URL: &str = "https://images-ext-1.discordapp.net/external/gqSkvZeXtYgqXI46ZxZIoZCgoQ4l0n79UhJqcmrBuLE/\
  https/cdn.discordapp.com/avatars/1093034213999132743/4c1bb03fa385a3773c8c4c196b9c8fad.png?format=webp&quality=lossless";

  let timestamp_millis = TIMESTAMP_BOOT.get().unwrap().duration_since(UNIX_EPOCH)?.as_secs();

  ctx
    .send(
      CreateReply::default().embed(
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

  let punishment = database::insert_punishment(
    GuildIdWrapper(member.guild_id.get()),
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
  let punishment = database::insert_punishment(
    GuildIdWrapper(member.guild_id.get()),
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
  let punishment = database::insert_punishment(
    GuildIdWrapper(member.guild_id.get()),
    UserIdWrapper(member.user.id.get()),
    UserIdWrapper(ctx.author().id.get()),
    PunishmentType::NOTE,
    None,
    Some(reason),
  )?;

  let (component, _) = punishments::get_messages(&ctx, &punishment, &member.user)?;
  ctx
    .send(
      CreateReply::new()
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .allowed_mentions(CreateAllowedMentions::default().empty_roles().empty_users())
        .components(vec![component]),
    )
    .await?;
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

#[poise::command(context_menu_command = "Report", guild_only)]
async fn report_context(ctx: Context<'_>, msg: Message) -> Result<(), Error> {
  let config = CONFIG.get().expect("This shouldn't happen (GET CONFIG)");

  if let Some(channels) = config.guilds.get(&ctx.guild_id().unwrap().to_string()) {
    if let Some(report_channel) = channels.channel_report.clone() {
      let user_avatar = msg
        .author
        .avatar_url()
        .unwrap_or_else(|| format!("https://cdn.discordapp.com/embed/avatars/{}.png", (msg.author.id.get() >> 22) % 6));
      let author_avatar = ctx
        .author()
        .avatar_url()
        .unwrap_or_else(|| format!("https://cdn.discordapp.com/embed/avatars/{}.png", (ctx.author().id.get() >> 22) % 6));

      ctx
        .http()
        .send_message(
          GenericChannelId::new(report_channel.parse::<u64>()?),
          Vec::new(),
          &CreateMessage::new().embed(
            CreateEmbed::new()
              .author(CreateEmbedAuthor::new(format!("{} ({})", msg.author.name, msg.author.id)).icon_url(user_avatar))
              .title("A message has been reported")
              .description(format!("**Reported Content**: \n\n{}", msg.content))
              .field("Message", format!("{}", msg.link()), false)
              .footer(CreateEmbedFooter::new(format!("{} ({})", ctx.author().name, ctx.author().id)).icon_url(author_avatar))
              .timestamp(Timestamp::now())
              .color(0x38393B),
          ),
        )
        .await?;
      ctx
        .send(
          CreateReply::new()
            .content(format!("Successfully reported a message by <@{}>.", msg.author.id))
            .ephemeral(true),
        )
        .await?;
      return Ok(());
    }
  }

  ctx
    .send(
      CreateReply::new()
        .content("Reporting has not been setup, please consult your local administrator.")
        .ephemeral(true),
    )
    .await?;
  Ok(())
}

/// Sends a message privately to the moderators
#[poise::command(slash_command, guild_only)]
async fn modmail(ctx: Context<'_>, #[description = "The message to send the moderators"] message: String) -> Result<(), Error> {
  let config = CONFIG.get().expect("This shouldn't happen (GET CONFIG)");

  if let Some(channels) = config.guilds.get(&ctx.guild_id().unwrap().to_string()) {
    if let Some(mod_msg_channel) = channels.channel_mod_msg.clone() {
      let author_avatar = ctx
        .author()
        .avatar_url()
        .unwrap_or_else(|| format!("https://cdn.discordapp.com/embed/avatars/{}.png", (ctx.author().id.get() >> 22) % 6));

      ctx
        .http()
        .send_message(
          GenericChannelId::new(mod_msg_channel.parse::<u64>()?),
          Vec::new(),
          &CreateMessage::new().embed(
            CreateEmbed::new()
              .title("A new modmail message has been submitted")
              .description(message)
              .footer(CreateEmbedFooter::new(format!("{} ({})", ctx.author().name, ctx.author().id)).icon_url(author_avatar))
              .timestamp(Timestamp::now())
              .color(0x38393B),
          ),
        )
        .await?;
      ctx
        .send(
          CreateReply::new()
            .content("Your modmail was sent successfully. Please wait patiently for a response.")
            .ephemeral(true),
        )
        .await?;
      return Ok(());
    }
  }

  ctx
    .send(
      CreateReply::new()
        .content("The mod mail has not been setup, please consult your local administrator.")
        .ephemeral(true),
    )
    .await?;
  Ok(())
}

#[poise::command(
  slash_command,
  subcommands("punishment_reason", "punishment_stale", "punishment_show", "punishment_search"),
  subcommand_required,
  guild_only,
  required_permissions = "MODERATE_MEMBERS"
)]
async fn punishment(_: Context<'_>) -> Result<(), Error> {
  Ok(()) // never called
}

/// Set the reason of a punishment
#[poise::command(slash_command, rename = "reason")]
async fn punishment_reason(ctx: Context<'_>, #[string] punishment: Uuid, reason: String) -> Result<(), Error> {
  if let Err(e) = database::update_punishment_reason(punishment, reason) {
    ctx.send(CreateReply::new().content(format!("Something went wrong: {e}")).ephemeral(true)).await?;
    return Ok(());
  }

  ctx.reply(format!("Punishment `{}` was successfully updated.", punishment.as_simple())).await?;
  Ok(())
}

async fn construct_show_component(ctx: Context<'_>, punishment: &Punishment) -> Result<CreateComponent<'static>, Error> {
  let display = PunishmentDisplay::from_punishment_type(&punishment.punishment_type);

  fn bool_to_emoji<'a>(val: bool) -> &'a str {
    if val { "<:yes:1500220801108934786>" } else { "<:no:1500220802841182211>" }
  }

  let issued_by_name = ctx.http().get_user(UserId::new(punishment.issued_by.0)).await?.name;
  let issued_to_name = ctx.http().get_user(UserId::new(punishment.issued_to.0)).await?.name;

  let mut fields = Vec::new();
  fields.push(format!("### Punishment {}", punishment.punishment_id.simple()));
  fields.push(format!("**Type**: {}", display.display));
  fields.push(format!("**Stale**: {}", bool_to_emoji(punishment.stale)));
  if let Some(stale_time) = punishment.stale_time_sec {
    fields.push(format!("**Stale Time**: <t:{}:F>", stale_time));
  }
  if let Some(stale_reason) = punishment.stale_reason.clone() {
    fields.push(format!("**Stale Reason**: {}", stale_reason));
  }

  fields.push(format!("**Time**: <t:{}:F>", punishment.time_sec));
  fields.push(format!(
    "**Issued by**: <@{}> (`@{}` / `{}`)",
    punishment.issued_by.0, issued_by_name, punishment.issued_by.0
  ));
  fields.push(format!(
    "**Issued to**: <@{}> (`@{}` / `{}`)",
    punishment.issued_to.0, issued_to_name, punishment.issued_to.0
  ));
  fields.push(format!("**Reason**: {}", punishment.reason.clone().unwrap_or_else(|| String::from("*No reason provided*"))));

  let fields = fields.into_iter().map(|s| CreateContainerComponent::TextDisplay(CreateTextDisplay::new(s)));
  let fields: Vec<CreateContainerComponent> = fields.collect();

  Ok(CreateComponent::Container(CreateContainer::new(fields).accent_color(display.color)))
}

async fn ensure_valid_punishment(ctx: Context<'_>, punishment: Result<Option<Punishment>, Error>) -> Result<Punishment, Error> {
  if let Err(e) = punishment {
    ctx.send(CreateReply::new().content(format!("Something went wrong: {e}")).ephemeral(true)).await?;
    return Err(e);
  }
  let punishment = punishment?;

  if let None = punishment {
    ctx.send(CreateReply::new().content("No punishment found.").ephemeral(true)).await?;
    return Err(Error::from("No punishment."));
  }

  Ok(punishment.unwrap())
}

/// Show the details of a punishment
#[poise::command(slash_command, rename = "show")]
async fn punishment_show(ctx: Context<'_>, #[string] punishment: Uuid) -> Result<(), Error> {
  let punishment = ensure_valid_punishment(ctx, database::fetch_single_punishment(punishment, GuildIdWrapper(ctx.guild_id().unwrap().get()))).await;
  if punishment.is_err() {
    return Ok(());
  }

  let punishment = punishment?;
  let component = construct_show_component(ctx, &punishment).await?;

  ctx
    .send(
      CreateReply::new()
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .allowed_mentions(CreateAllowedMentions::default().empty_roles().empty_users())
        .components(vec![component]),
    )
    .await?;
  Ok(())
}

/// Mark a punishment as stale
#[poise::command(slash_command, rename = "stale")]
async fn punishment_stale(ctx: Context<'_>, #[string] punishment: Uuid, reason: Option<String>) -> Result<(), Error> {
  let punishment = ensure_valid_punishment(ctx, database::stale_punishment(punishment, GuildIdWrapper(ctx.guild_id().unwrap().get()), reason)).await;
  if punishment.is_err() {
    return Ok(());
  }

  let punishment = punishment?;
  match punishment.punishment_type {
    PunishmentType::TIMEOUT => {
      if let Ok(mut member) = ctx.http().get_member(ctx.guild_id().unwrap(), UserId::new(punishment.issued_to.0)).await {
        member.enable_communication(ctx.http()).await?;
      }
    }
    PunishmentType::BAN => {
      ctx
        .http()
        .remove_ban(ctx.guild_id().unwrap(), UserId::new(punishment.issued_to.0), Some("Punishment stale."))
        .await?;
    }
    _ => {}
  };

  let component = construct_show_component(ctx, &punishment).await?;

  ctx
    .send(
      CreateReply::new()
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .allowed_mentions(CreateAllowedMentions::default().empty_roles().empty_users())
        .components(vec![
          component,
          CreateComponent::TextDisplay(CreateTextDisplay::new(format!(
            "Punishment `{}` was successfully updated.",
            punishment.punishment_id.simple()
          ))),
        ]),
    )
    .await?;

  Ok(())
}

#[poise::command(slash_command, rename = "search", subcommands("punishment_search_user"), subcommand_required)]
async fn punishment_search(_: Context<'_>) -> Result<(), Error> {
  Ok(()) // never called
}

/// Search for punishments issued to a user
#[poise::command(slash_command, rename = "user")]
async fn punishment_search_user(ctx: Context<'_>, user: User) -> Result<(), Error> {
  let entries: Vec<PartialPunishment> = database::fetch_punishments(UserIdWrapper(user.id.get()), GuildIdWrapper(ctx.guild_id().unwrap().get()))?;

  let user_avatar = user
    .avatar_url()
    .unwrap_or_else(|| format!("https://cdn.discordapp.com/embed/avatars/{}.png", (user.id.get() >> 22) % 6));

  let top_section = CreateSection::new(
    vec![CreateSectionComponent::TextDisplay(CreateTextDisplay::new(format!(
      "## Punishment history for {}",
      user.name
    )))],
    CreateSectionAccessory::Thumbnail(CreateThumbnail::new(CreateUnfurledMediaItem::new(user_avatar))),
  );

  let mut fields = Vec::new();
  if entries.is_empty() {
    fields.push("No results found.".to_string())
  } else {
    for entry in entries {
      let display = PunishmentDisplay::from_punishment_type(&entry.punishment_type);
      fields.push(format!("{} (`{}`) <t:{}:F>", display.display, entry.punishment_id.simple(), entry.time_sec));
      if let Some(reason) = entry.reason {
        fields.push(format!("⟶ `{}`", reason));
      }
    }
  }

  let fields = fields.into_iter().map(|s| CreateContainerComponent::TextDisplay(CreateTextDisplay::new(s)));
  let fields: Vec<CreateContainerComponent> = fields.collect();

  let mut container_components: Vec<CreateContainerComponent> = Vec::new();
  container_components.push(CreateContainerComponent::Section(top_section));
  container_components.push(CreateContainerComponent::Separator(CreateSeparator::new()));
  for field in fields {
    container_components.push(field)
  }

  ctx
    .send(
      CreateReply::new()
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .allowed_mentions(CreateAllowedMentions::default().empty_roles().empty_users())
        .components(vec![CreateComponent::Container(CreateContainer::new(container_components))]),
    )
    .await?;

  Ok(())
}
