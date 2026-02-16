use crate::bot::Context;
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
            Ok(amt) if amt > 0 => amt,
            _ => {
                ctx.send(
                    CreateReply::default()
                        .embed(
                            CreateEmbed::new()
                                .title("❌ Ale ty jesteś pacanem...")
                                .description(
                                    "Wpisuje się poprawną liczbę lub `all` kolego.".to_string(),
                                )
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
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("❌ Jesteś biedny")
                        .description(format!(
                            "Nie masz tyle gotówki w portfelu!\nPosiadasz: `{}` 💵",
                            user_data.user.cash
                        ))
                        .color(0xFF0000),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    if (amount_to_dep + user_data.user.bank) > BANK_LIMIT {
        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title("❌ Limit osiągnięty")
                .description("Nie możesz schować w banku więcej niż 100 tysięcy dolarów. Niestety, reszta musi pozostać w portfelu.")
                .color(0xFF0000))
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let success = db.deposit(user_id, amount_to_dep).await?;

    if success {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("🏦 Wpłata przyjęta")
                    .description("Pomyślnie wpłacono pieniądze do banku.".to_string())
                    .field("Kwota", format!("`{}` 💰", amount_to_dep), true)
                    .field(
                        "Nowy stan konta",
                        format!("`{}` 💳", user_data.user.bank + amount_to_dep),
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
