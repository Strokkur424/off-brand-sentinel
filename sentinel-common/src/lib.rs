pub mod modals;
pub mod wrapper;
pub mod config;

pub struct Data {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
