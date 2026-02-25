use crate::bot::Context;
use crate::bot::utils::models::LogType;
use crate::bot::utils::msg::reply;
use anyhow::Error;
use poise::{ChoiceParameter, command};
use serenity::all::UserId;

#[derive(ChoiceParameter)]
pub enum BalanceType {
    #[name = "wallet"]
    Wallet,
    #[name = "bank"]
    Bank,
}

impl BalanceType {
    pub fn to_str(&self) -> &'static str {
        match self {
            BalanceType::Wallet => "wallet",
            BalanceType::Bank => "bank",
        }
    }
}

#[command(
    slash_command,
    prefix_command,
    description_localized("en-US", "Set balance."),
    description_localized("pl", "Ustaw saldo."),
    aliases("setbal", "sb")
)]
pub async fn set_balance(
    ctx: Context<'_>,

    #[description_localized("en-US", "New balance")]
    #[description_localized("pl", "Nowe saldo")]
    balance: i64,

    #[description_localized("en-US", "Balance type")]
    #[description_localized("pl", "Rodzaj salda")]
    balance_type: BalanceType,

    #[description_localized("en-US", "User")]
    #[description_localized("pl", "Użytkownik")]
    user: Option<UserId>,
) -> Result<(), Error> {
    let (user_id, user_name) = match user {
        Some(user_id) => {
            let user_obj = user_id.to_user(ctx.serenity_context()).await?;
            (user_id.get() as i64, user_obj.name)
        }
        None => {
            let author = ctx.author();
            (author.id.get() as i64, author.name.clone())
        }
    };

    let db = &ctx.data().db;

    let user_data = db.ensure_member(user_id).await?;

    match balance_type {
        BalanceType::Wallet => {
            user_data.user.set_wallet(&db.pool, balance).await?;
        }
        BalanceType::Bank => {
            user_data.user.set_bank(&db.pool, balance).await?;
        }
    }

    reply(
        &ctx,
        LogType::Success,
        "💵 Balance set!",
        format!(
            "Successfully set {} balance of {user_name} to {balance}!",
            balance_type.to_str(),
        )
        .as_str(),
    )
    .await?;

    Ok(())
}
