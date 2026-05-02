use crate::commands::{Context, Error};
use crate::database::{Punishment, PunishmentType};
use crate::wrapper::UserIdWrapper;
use poise::serenity_prelude::small_fixed_array::FixedString;
use poise::serenity_prelude::{
  CreateAllowedMentions, CreateComponent, CreateContainer, CreateContainerComponent, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, CreateTextDisplay,
  MessageFlags, User,
};
use poise::CreateReply;

pub struct PunishmentDisplay<'a> {
  pub display: &'a str,
  pub color: u32,
  pub dm_text: &'a str,
}

impl PunishmentDisplay<'_> {
  fn from<'a>(display: &'a str, color: u32, dm_text: &'a str) -> PunishmentDisplay<'a> {
    PunishmentDisplay { display, color, dm_text }
  }

  pub fn from_punishment_type<'a>(kind: &PunishmentType) -> PunishmentDisplay<'a> {
    match kind {
      PunishmentType::KICK => PunishmentDisplay::from("<:dot_gray:1499540057172742174> kick", 0xB3968D, "You have been kicked."),
      PunishmentType::WARN => PunishmentDisplay::from("<:dot_orange:1499540055402745876> warn", 0xFAA620, "You have been warned."),
      PunishmentType::BAN => PunishmentDisplay::from("<:dot_red:1499540053737865457> ban", 0xEF4949, "You have been banned."),
      PunishmentType::TIMEOUT => PunishmentDisplay::from("<:dot_purple:1499540059672805546> timeout", 0x9A59B4, "You have been timed out."),
      PunishmentType::NOTE => PunishmentDisplay::from("<:dot_blue:1499540058330632234> note", 0x3797DA, ""),
    }
  }
}

pub async fn execute_ban(ctx: &Context<'_>, member: &User, delete_messages: bool, reason: Option<String>) -> Result<(), Error> {
  let delete_seconds = if delete_messages { std::time::Duration::from_hours(1).as_secs() } else { 0 };

  let punishment = crate::database::insert_punishment(
    UserIdWrapper(member.id.get()),
    UserIdWrapper(ctx.author().id.get()),
    PunishmentType::BAN,
    None,
    reason.clone(),
  )?;

  send_messages(&ctx, &punishment, &member).await?;
  ctx
    .http()
    .ban_user(ctx.guild_id().expect("Context outside of guild"), member.id, delete_seconds as u32, reason.as_deref())
    .await?;
  Ok(())
}

pub async fn execute_kick(ctx: &Context<'_>, member: &User, reason: Option<String>) -> Result<(), Error> {
  let punishment = crate::database::insert_punishment(
    UserIdWrapper(member.id.get()),
    UserIdWrapper(ctx.author().id.get()),
    PunishmentType::KICK,
    None,
    reason.clone(),
  )?;

  send_messages(&ctx, &punishment, &member).await?;
  ctx
    .http()
    .kick_member(ctx.guild_id().expect("Context outside of guild"), member.id, reason.as_deref())
    .await?;
  Ok(())
}

fn get_embeds<'a>(punishment: &Punishment, target: &User, guild_name: FixedString, guild_icon: String) -> Result<(CreateComponent<'a>, Option<CreateMessage<'a>>), Error> {
  let display = PunishmentDisplay::from_punishment_type(&punishment.punishment_type);

  let mut components: Vec<String> = Vec::new();
  components.push(format!("### Punishment {}", punishment.punishment_id.as_simple()));
  components.push(format!("**Type**: {}", display.display));
  components.push(format!("**Issued to**: <@{}> (`@{}` / `{}`)", target.id.get(), target.name, target.id.get()));
  components.push(format!("**Reason**: {}", punishment.reason.clone().unwrap_or(String::from("*No reason provided*"))));

  if let Ok(duration) = punishment.duration.clone().ok_or_else(|| "Not present") {
    components.push(format!("**Duration**: ~{} (expiry: <t:{}:F>", duration.display, duration.to_unix_time_from_now()));
  }

  let components: Vec<CreateContainerComponent> = components.into_iter().map(|s| CreateContainerComponent::TextDisplay(CreateTextDisplay::new(s))).collect();
  let components = CreateComponent::Container(CreateContainer::new(components).accent_colour(display.color));

  if punishment.punishment_type == PunishmentType::NOTE {
    Ok((components, None))
  } else {
    let msg = CreateMessage::new().embed(
      CreateEmbed::new()
        .color(display.color)
        .author(CreateEmbedAuthor::new(guild_name).icon_url(guild_icon))
        .title(display.dm_text)
        .field("Reason", punishment.reason.clone().unwrap_or(String::from("(not specified)")), true)
        .footer(CreateEmbedFooter::new(format!("Punishment: {}", punishment.punishment_id.as_simple()))),
    );
    Ok((components, Some(msg)))
  }
}

pub fn get_messages<'a>(ctx: &Context<'_>, punishment: &Punishment, member: &User) -> Result<(CreateComponent<'a>, Option<CreateMessage<'a>>), Error> {
  let (guild_name, guild_icon) = {
    let guild = ctx.guild().unwrap();

    let guild_name = (&guild.name).clone();
    let guild_icon_url = guild.icon_url().unwrap();

    (guild_name, guild_icon_url)
  };

  get_embeds(&punishment, &member, guild_name, guild_icon)
}

pub async fn send_messages(ctx: &Context<'_>, punishment: &Punishment, member: &User) -> Result<(), Error> {
  let (components, dm_message) = get_messages(&ctx, &punishment, &member)?;
  ctx
    .send(
      CreateReply::new()
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .allowed_mentions(CreateAllowedMentions::default().empty_roles().empty_users())
        .components(vec![components]),
    )
    .await?;

  if let Ok(msg) = dm_message.ok_or_else(|| "not present") {
    if let Ok(channel) = member.create_dm_channel(ctx.http()).await {
      ctx.http().send_message(channel.id.widen(), Vec::new(), &msg).await?;
    }
  }

  Ok(())
}
