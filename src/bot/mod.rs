use crate::bot::client::Data;
use anyhow::Error;

pub mod client;
pub mod commands;
pub mod errors;
pub mod format_number;
pub mod utils;

pub type Context<'a> = poise::Context<'a, Data, Error>;
