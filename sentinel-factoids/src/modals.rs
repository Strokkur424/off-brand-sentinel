use poise::serenity_prelude::{
  CreateInputText, CreateInteractionResponse, CreateLabel, CreateModal, CreateModalComponent, InputTextStyle,
  ModalInteractionData,
};
use sentinel_common::modals::{send_modal, FactoidCreateData};
use sentinel_common::{Error, FactoidData};
use crate::Data;

fn create_factoid_create_modal(title: String, custom_id: String) -> CreateInteractionResponse<'static> {
  let description = CreateLabel::input_text(
    "Description",
    CreateInputText::new(InputTextStyle::Short, "description").required(false),
  );
  let components = CreateLabel::input_text(
    "Components (JSON)",
    CreateInputText::new(InputTextStyle::Paragraph, "components").required(true),
  );

  CreateInteractionResponse::Modal(CreateModal::new(custom_id, title).components(vec![
    CreateModalComponent::Label(description),
    CreateModalComponent::Label(components),
  ]))
}

fn parse_factoid_create_modal(mut data: ModalInteractionData) -> FactoidCreateData {
  FactoidCreateData {
    components: poise::find_modal_text(&mut data, "components").expect("Missing components"),
    description: poise::find_modal_text(&mut data, "description"),
  }
}

fn create_factoid_edit_modal(factoid: &FactoidData) -> CreateInteractionResponse<'static> {
  let display_name = CreateLabel::input_text(
    "Display Name",
    CreateInputText::new(InputTextStyle::Short, "display_name")
      .placeholder(factoid.display_name.clone())
      .required(false),
  );

  let factoid_name = CreateLabel::input_text(
    "Id",
    CreateInputText::new(InputTextStyle::Short, "factoid_name")
      .placeholder(factoid.factoid_name.clone())
      .required(false),
  );

  let mut desc_input = CreateInputText::new(InputTextStyle::Short, "description");
  if let Some(desc) = factoid.description.clone() {
    desc_input = desc_input.placeholder(desc);
  }
  let description = CreateLabel::input_text("Description", desc_input.required(false));

  let components = CreateLabel::input_text(
    "Components (JSON)",
    CreateInputText::new(InputTextStyle::Paragraph, "components")
      .placeholder("Leave blank for current.")
      .required(false),
  );

  CreateInteractionResponse::Modal(
    CreateModal::new(factoid.factoid_name.clone(), format!("Edit {}", factoid.display_name)).components(vec![
      CreateModalComponent::Label(display_name),
      CreateModalComponent::Label(factoid_name),
      CreateModalComponent::Label(description),
      CreateModalComponent::Label(components),
    ]),
  )
}

fn parse_factoid_edit_modal(mut data: ModalInteractionData, factoid: &FactoidData) -> FactoidData {
  let description: Option<String>;
  match poise::find_modal_text(&mut data, "description") {
    Some(desc) => description = Some(desc),
    None => description = factoid.description.clone(),
  }

  FactoidData {
    display_name: poise::find_modal_text(&mut data, "display_name").unwrap_or(factoid.display_name.clone()),
    factoid_name: poise::find_modal_text(&mut data, "factoid_name").unwrap_or(factoid.factoid_name.clone()),
    components: poise::find_modal_text(&mut data, "components").unwrap_or(factoid.components.clone()),
    description,
  }
}

pub async fn send_factoid_create_modal(
  ctx: &poise::ApplicationContext<'_, Data, Error>,
  title: String,
  custom_id: String,
) -> Result<Option<FactoidCreateData>, Error> {
  let modal = create_factoid_create_modal(title, custom_id.clone());
  send_modal(ctx, modal, custom_id, parse_factoid_create_modal).await
}

pub async fn send_factoid_edit_modal(
  ctx: &poise::ApplicationContext<'_, Data, Error>,
  factoid: FactoidData,
) -> Result<Option<FactoidData>, Error> {
  let modal = create_factoid_edit_modal(&factoid);
  send_modal(ctx, modal, factoid.factoid_name.clone(), |d| {
    parse_factoid_edit_modal(d, &factoid)
  })
  .await
}
