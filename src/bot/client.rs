use crate::bot::{commands};
use crate::config::models::Config;
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};
use serenity::all::{GatewayIntents};
use sqlx::{Pool, Sqlite};
use std::error::Error;

pub struct Data {
    pub pool: Pool<Sqlite>,
    pub config: Config,
}

pub async fn run(config: Config, pool: Pool<Sqlite>) -> Result<(), Box<dyn Error>> {
    let token = config.bot.token.clone();
    let prefix = config.bot.prefix.clone();

    let commands = vec![
        commands::ping::ping(),
    ];

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands,
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(prefix),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { pool, config })
            })
        })
        .build();

    let client = serenity::Client::builder(token, GatewayIntents::all())
        .framework(framework)
        .await;

    client?.start().await?;
    Ok(())
}
