use poise::serenity_prelude::{ErrorResponse, Http, HttpError, LightMethod, Request, Route};
use poise::ApplicationContext;
use sentinel_common::{Data, Error};

pub async fn respond_manually_components(
  ctx: ApplicationContext<'_, Data, Error>,
  components: String,
) -> Result<(), Error> {
  let has_sent_initial_response = ctx.has_sent_initial_response.load(std::sync::atomic::Ordering::SeqCst);

  let route = if has_sent_initial_response {
    Route::WebhookFollowupMessages {
      application_id: ctx
        .http()
        .application_id()
        .ok_or_else(|| Error::from("No ApplicationId"))?,
      token: ctx.interaction.token.as_str(),
    }
  } else {
    Route::InteractionResponse {
      interaction_id: ctx.interaction.id,
      token: ctx.interaction.token.as_str(),
    }
  };

  let flags = (1 << 6) | (1 << 15);
  let body = format!(
    "{{ \"flags\": {}, \"allowed_mentions\": {{ \"parse\": [] }}, \"components\": {} }}",
    flags, components
  );


  let request = Request::new(route, LightMethod::Post).body(Some(body.clone().into_bytes()));
  // println!("Request body: <{}>", body);

  fire(ctx.http(), request).await?;
  ctx
    .has_sent_initial_response
    .store(true, std::sync::atomic::Ordering::SeqCst);

  Ok(())
}

async fn fire(http: &Http, req: Request<'_>) -> Result<(), Error> {
  let cloned_req = req.clone();
  let method = cloned_req.method_ref().reqwest_method();
  let response = http.request(req).await?;

  let status = response.status();
  if status.is_success() {
    return Ok(());
  }

  Err(Error::from(HttpError::UnsuccessfulRequest(
    ErrorResponse::from_response(response, method).await,
  )))
}
