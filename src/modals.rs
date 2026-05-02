use crate::commands::{Data, Error};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CreateInputText, CreateInteractionResponse, CreateLabel, CreateModal, CreateModalComponent, InputTextStyle, ModalInteractionData};

fn create_reason_modal(title: String, custom_id: String) -> CreateInteractionResponse<'static> {
  let reason_label = CreateLabel::input_text("Reason", CreateInputText::new(InputTextStyle::Short, "reason").required(true));
  CreateInteractionResponse::Modal(CreateModal::new(custom_id, title).components(vec![CreateModalComponent::Label(reason_label)]))
}

fn parse_reason_modal(mut data: ModalInteractionData) -> String {
  poise::find_modal_text(&mut data, "reason").expect("missing reason")
}

pub async fn send_reason_modal(ctx: &poise::ApplicationContext<'_, Data, Error>, title: String, custom_id: String) -> Result<Option<String>, Error> {
  let modal = create_reason_modal(title, custom_id.clone());
  ctx.interaction.create_response(ctx.http(), modal).await?;

  let response = serenity::collector::ModalInteractionCollector::new(ctx.serenity_context())
    .filter(move |d| d.data.custom_id.as_str() == custom_id)
    .timeout(std::time::Duration::from_mins(2))
    .await;
  let response = match response {
    Some(x) => x,
    None => return Ok(None),
  };

  response.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
  let reason = Some(parse_reason_modal(response.data));
  ctx.has_sent_initial_response.store(true, std::sync::atomic::Ordering::SeqCst);

  Ok(reason)
}
