use crate::bot::{Context, format_number::format_number};
use anyhow::Error;
use poise::CreateReply;
use serenity::all::CreateEmbed;

const BANK_LIMIT: i64 = 100_000_00;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("dep"),
    description_localized("pl", "Wpłać pieniądze do banku, aby były bezpieczne.")
)]
pub async fn deposit(
    ctx: Context<'_>,
    #[description_localized("pl", "Kwota do wpłacenia (lub 'all')")] amount_str: String,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let user_data = db.ensure_member(user_id).await?;

    let amount_to_dep = match amount_str.to_lowercase().as_str() {
        "all" => user_data.user.cash,
        _ => match amount_str.parse::<i64>() {
            Ok(amt) if amt > 0 => amt * 100, 
            _ => {
                ctx.send(
                    CreateReply::default().embed(
                        CreateEmbed::new()
                            .title("❌ Ale ty jesteś pacanem...")
                            .description("Wpisz poprawną liczbę lub `all`.")
                            .color(0xFF0000),
                    )
                    .ephemeral(true),
                )
                .await?;
                return Ok(());
            }
        },
    };

    if amount_to_dep > user_data.user.cash {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Jesteś biedny")
                    .description(format!(
                        "Nie masz tyle gotówki!\nPosiadasz: `{}` 💵",
                        format_number(user_data.user.cash)
                    ))
                    .color(0xFF0000),
            )
            .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    if user_data.user.bank + amount_to_dep > BANK_LIMIT {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Limit osiągnięty")
                    .description("Nie możesz mieć w banku więcej niż 100 000 zł. Reszta musi pozostać w portfelu.")
                    .color(0xFF0000),
            )
            .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    if db.deposit(user_id, amount_to_dep).await? {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("🏦 Wpłata przyjęta")
                    .description("Pomyślnie wpłacono pieniądze do banku.")
                    .field("Kwota", format!("`{}` 💰", format_number(amount_to_dep)), true)
                    .field(
                        "Nowy stan konta",
                        format!("`{}` 💳", format_number(user_data.user.bank + amount_to_dep)),
                        true,
                    )
                    .color(0x00FF00),
            ),
        )
        .await?;
    } else {
        ctx.say("Coś poszło nie tak podczas operacji bankowej. Spróbuj ponownie.")
            .await?;
    }

    Ok(())
}

