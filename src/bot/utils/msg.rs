#![allow(unused)]
use ::serenity::all::Color;
use poise::{CreateReply, ReplyHandle, serenity_prelude as serenity};
use serenity::Colour;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor};

use crate::bot::Context;
use crate::bot::utils::models::{LogConfig, LogType};


fn get_settings(log_type: LogType) -> LogConfig {
    match log_type {
        LogType::Success => LogConfig::new("✅".to_owned(), Color::from_rgb(12, 237, 94)),
        LogType::Info => LogConfig::new("ℹ️".to_owned(), Color::from_rgb(255, 200, 0)),
        LogType::Tip => LogConfig::new("💡".to_owned(), Color::from_rgb(212, 0, 255)),
        LogType::Warn => LogConfig::new("⚠️".to_owned(), Color::from_rgb(237, 115, 0)),
        LogType::Error => LogConfig::new("💔".to_owned(), Color::from_rgb(219, 26, 0)),
    }
}

pub fn build_embed(
    log_type: LogType,
    title: impl Into<String>,
    desc: impl Into<String>,
) -> CreateEmbed {
    let LogConfig { emoji, color } = get_settings(log_type);

    CreateEmbed::new()
        .title(format!("{} {}", emoji, title.into()))
        .description(desc.into())
        .color(color)
        .author(CreateEmbedAuthor::new("Eclair Casino"))
}

pub fn reply<'a>(
    ctx: &Context<'a>,
    log_type: LogType,
    title: impl Into<String>,
    desc: impl Into<String>,
) -> impl Future<Output = Result<ReplyHandle<'a>, serenity::Error>> {
    ctx.send(CreateReply::default().embed(build_embed(log_type, title, desc)))
}

pub fn reply_err<'a>(
    ctx: &Context<'a>,
    title: impl Into<String>,
    desc: impl Into<String>,
) -> impl Future<Output = Result<ReplyHandle<'a>, serenity::Error>> {
    reply(ctx, LogType::Error, title, desc)
}

pub fn reply_warn<'a>(
    ctx: &Context<'a>,
    title: impl Into<String>,
    desc: impl Into<String>,
) -> impl Future<Output = Result<ReplyHandle<'a>, serenity::Error>> {
    reply(ctx, LogType::Warn, title, desc)
}

pub fn reply_info<'a>(
    ctx: &Context<'a>,
    title: impl Into<String>,
    desc: impl Into<String>,
) -> impl Future<Output = Result<ReplyHandle<'a>, serenity::Error>> {
    reply(ctx, LogType::Info, title, desc)
}

pub fn reply_success<'a>(
    ctx: &Context<'a>,
    title: impl Into<String>,
    desc: impl Into<String>,
) -> impl Future<Output = Result<ReplyHandle<'a>, serenity::Error>> {
    reply(ctx, LogType::Success, title, desc)
}

pub fn reply_tip<'a>(
    ctx: &Context<'a>,
    title: impl Into<String>,
    desc: impl Into<String>,
) -> impl Future<Output = Result<ReplyHandle<'a>, serenity::Error>> {
    reply(ctx, LogType::Tip, title, desc)
}
