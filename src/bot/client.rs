use crate::bot::commands;
use crate::bot::errors::on_error;
use crate::config::models::Config;
use crate::services::database::abstraction::DbManager;
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions, builtins};
use serenity::all::GatewayIntents;
use sqlx::{Pool, Sqlite};
use std::error::Error;

pub struct Data {
    pub db: DbManager,
    pub active_players: std::sync::Mutex<std::collections::HashSet<i64>>,
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
        commands::help::help(),
        commands::rob::rob(),
        commands::pay::pay(),
        commands::topmoney::topmoney(),
        commands::slot_machine::slots(),
        commands::coinflip::coinflip(),
        commands::blackjack::blackjack(),
        commands::shop::shop(),
        commands::buy::buy(),
        commands::dice::dice(),
        commands::crash::crash(),
        commands::scratch::scratch(),
    ];

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands,
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(prefix),
                ..Default::default()
            },
            on_error: |err| Box::pin(on_error(err)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    db,
                    active_players: std::sync::Mutex::new(std::collections::HashSet::new()),
                })
            })
        })
        .build();

    let client = serenity::Client::builder(token, GatewayIntents::all())
        .framework(framework)
        .await;

    client?.start().await?;
    Ok(())
}
