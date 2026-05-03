pub mod config;
pub mod modals;
pub mod wrapper;

#[derive(Clone)]
pub struct FactoidData {
  pub display_name: String,
  pub factoid_name: String,
  pub description: Option<String>,
  pub components: String,
}

pub struct Data {
  pub factoid: Option<FactoidData>
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

impl FactoidData {
  pub fn with_components(&self, components: String) -> Self {
    Self {
      display_name: self.display_name.clone(),
      factoid_name: self.factoid_name.clone(),
      description: self.description.clone(),
      components,
    }
  }
}
