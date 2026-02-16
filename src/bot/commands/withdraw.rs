use crate::bot::{Context, format_number::format_number};
use anyhow::Error;
use poise::CreateReply;
use serenity::all::CreateEmbed;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("wd", "with"),
    description_localized("pl", "Wypłać pieniądze z banku. Musisz szastać hajsem, prawda?")
)]
pub async fn withdraw(
    ctx: Context<'_>,
    #[description_localized("pl", "Kwota do wypłacenia (lub 'all')")] amount_str: String,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let user_data = db.ensure_member(user_id).await?;

    let amount_to_with = match amount_str.to_lowercase().as_str() {
        "all" => user_data.user.bank,
        _ => match amount_str.parse::<i64>() {
            Ok(amount) if amount > 0 => amount,
            _ => {
                ctx.send(
                    CreateReply::default()
                        .embed(
                            CreateEmbed::new()
                                .title("❌ Ale ty jesteś pacanem...")
                                .description("Wpisuje się poprawną liczbę lub `all` kolego.")
                                .color(0xFF0000),
                        )
                        .ephemeral(true),
                )
                .await?;

                return Ok(());
            }
        },
    };

    if amount_to_with > user_data.user.bank {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("❌ Jesteś biedny")
                        .description(format!(
                            "Nie masz tyle kasy w banku, nędzarzu!\nW banku masz: `{}` 💳",
                            format_number(user_data.user.bank)
                        ))
                        .color(0xFF0000),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    match db.withdraw(user_id, amount_to_with).await {
        Ok(..) => {
            ctx.send(
                CreateReply::default().embed(
                    CreateEmbed::new()
                        .title("🏦 Wypłata zrealizowana")
                        .description(
                            "Właśnie wyciągnąłeś swoje ciężko (może nie?) zarobione pieniądze."
                                .to_string(),
                        )
                        .field("Kwota", format!("`{}` 💵", format_number(amount_to_with)), true)
                        .field(
                            "Reszta w banku",
                            format!("`{}` 💳", format_number(user_data.user.bank - amount_to_with)),
                            true,
                        )
                        .color(0xFFFF00),
                ),
            )
            .await?;
        }
        Err(..) => {
            ctx.say("Bankier uciekł z Twoją kasą (błąd bazy danych).")
                .await?;
        }
    }

    Ok(())
}
