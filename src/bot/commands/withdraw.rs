use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

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

    let (member, _) = db.ensure_member(user_id).await?;

    let amount_to_with = match amount_str.to_lowercase().as_str() {
        "all" => member.bank,
        _ => match amount_str.parse::<i64>() {
            Ok(amt) if amt > 0 => amt,
            _ => {
                ctx.send(
                    CreateReply::default()
                        .embed(
                            serenity::CreateEmbed::new()
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

    if amount_to_with > member.bank {
        ctx.send(
            CreateReply::default()
                .embed(
                    serenity::CreateEmbed::new()
                        .title("❌ Jesteś biedny")
                        .description(format!(
                            "Nie masz tyle kasy w banku, nędzarzu!\nW banku masz: `{}` 💳",
                            member.bank
                        ))
                        .color(0xFF0000),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let success = db.withdraw(user_id, amount_to_with).await?;

    if success {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("🏦 Wypłata zrealizowana")
                    .description(format!(
                        "Właśnie wyciągnąłeś swoje ciężko (może nie?) zarobione pieniądze."
                    ))
                    .field("Kwota", format!("`{}` 💵", amount_to_with), true)
                    .field(
                        "Reszta w banku",
                        format!("`{}` 💳", member.bank - amount_to_with),
                        true,
                    )
                    .color(0xFFFF00),
            ),
        )
        .await?;
    } else {
        ctx.say("Bankier uciekł z Twoją kasą (błąd bazy danych).")
            .await?;
    }

    Ok(())
}
