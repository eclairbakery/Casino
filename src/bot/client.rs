use crate::bot::{commands};
use crate::config::models::Config;
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};
use serenity::all::{GatewayIntents};
use sqlx::{Pool, Sqlite};
use std::error::Error;
use crate::services::database::abstraction::DbManager;

pub struct Data {
    pub pool: Pool<Sqlite>,
    pub config: Config,
    pub db: DbManager
}

pub async fn run(config: Config, pool: Pool<Sqlite>) -> Result<(), Box<dyn Error>> {
    let db = DbManager::new(pool.clone());
    let token = config.bot.token.clone();
    let prefix = config.bot.prefix.clone();

    let commands = vec![
        commands::ping::ping(),
        commands::work::work(),
        commands::bal::balance(),
        commands::slut::slut(),
        commands::crime::crime(),
        commands::deposit::deposit(),
        commands::withdraw::withdraw(),
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
                Ok(Data { pool, config, db })
            })
        })
        .build();

    let client = serenity::Client::builder(token, GatewayIntents::all())
        .framework(framework)
        .await;

    client?.start().await?;
    Ok(())
}
