use crate::Error;
use poise::serenity_prelude::{
  CacheHttp, CommandInteraction, Context, ErrorResponse, Http, HttpError, LightMethod, Message, Request, Route,
  StatusCode,
};

pub(crate) struct FactoidContext<'a> {
  pub ctx: &'a Context,
  pub interaction: &'a CommandInteraction,
}

impl FactoidContext<'_> {
  pub async fn send_plain(&self, message: &String, ephemeral: bool) -> Result<(), Error> {
    let route = Route::InteractionResponse {
      interaction_id: self.interaction.id,
      token: self.interaction.token.as_str(),
    };

    let flags = if ephemeral { 1 << 6 } else { 0 };
    let body = format!(
      "{{ \"type\": 4, \"data\": {{ \"flags\": {}, \"content\": \"{}\", \"allowed_mentions\": {{ \"parse\": [] }} }} }}",
      flags, message
    );

    let request = Request::new(route, LightMethod::Post).body(Some(body.clone().into_bytes()));

    wind(self.ctx.http(), request).await?;
    Ok(())
  }

  pub async fn respond_manually_components(&self, components: String) -> Result<(), Error> {
    let route = Route::InteractionResponse {
      interaction_id: self.interaction.id,
      token: self.interaction.token.as_str(),
    };

    let flags = 1 << 15;
    let body = format!(
      "{{ \"type\": 4, \"data\": {{ \"flags\": {}, \"allowed_mentions\": {{ \"parse\": [] }}, \"components\": {} }} }}",
      flags, components
    );

    let request = Request::new(route, LightMethod::Post).body(Some(body.clone().into_bytes()));

    wind(self.ctx.http(), request).await?;
    Ok(())
  }

  pub async fn reply_to_message(&self, message: &Message, components: String) -> Result<(), Error> {
    let flags = 1 << 15;
    let body = format!(
      "{{ \"flags\": {}, \"message_reference\": {{ \"message_id\": \"{}\" }}, \"components\": {} }}",
      flags,
      message.id.get(),
      components
    );

    let request = Request::new(
      Route::ChannelMessages {
        channel_id: message.channel_id,
      },
      LightMethod::Post,
    )
    .body(Some(body.clone().into_bytes()));

    wind(self.ctx.http(), request).await?;
    Ok(())
  }
}

async fn wind(http: &Http, req: Request<'_>) -> Result<(), Error> {
  let cloned_req = req.clone();
  let route = req.route_ref();
  let method = req.method_ref().reqwest_method();
  let response = http.request(cloned_req).await?;

  let status = response.status();
  if status.is_success() {
    if status != StatusCode::NO_CONTENT {
      let route = route.path();
      println!("[WARN] Mismatched successful response status from {route}! Expected 'No Content' but got {status}");
    }

    return Ok(());
  }

  Err(Error::from(HttpError::UnsuccessfulRequest(
    ErrorResponse::from_response(response, method).await,
  )))
}
