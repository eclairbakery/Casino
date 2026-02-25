use crate::bot::Context;
use crate::bot::utils::models::LogType;
use anyhow::Error;
use poise::{CreateReply, ReplyHandle};
use serenity::all::{Color, Colour, CreateEmbed};

pub fn build_embed(log_type: LogType, title: &str, desc: &str) -> CreateEmbed {
    let mut embed = CreateEmbed::new().title(title).description(desc);

    embed = match log_type {
        LogType::Success => embed.color(Colour::DARK_GREEN),
        LogType::Failure => embed.color(Color::RED),
    };

    embed
}

pub async fn reply<'a>(
    ctx: &Context<'a>,
    log_type: LogType,
    title: &str,
    desc: &str,
) -> Result<ReplyHandle<'a>, Error> {
    let msg = ctx
        .send(CreateReply::default().embed(build_embed(log_type, title, desc)))
        .await?;

    Ok(msg)
}
