use crate::bot::{Context, format_number::format_number};
use anyhow::Error;
use poise::CreateReply;
use serenity::all::{CreateEmbed, User};

#[poise::command(
    slash_command,
    prefix_command,
    aliases("daj", "przelej", "give"),
    description_localized("pl", "Przelej pieniądze innemu użytkownikowi.")
)]
pub async fn pay(
    ctx: Context<'_>,
    #[description_localized("pl", "Komu chcesz przelać pieniądze?")] receiver: User,
    #[description_localized("pl", "Ile pieniędzy chcesz przelać?")] amount: i64,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let receiver_id = receiver.id.get() as i64;
    let db = &ctx.data().db;

    if amount <= 0 {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("❌ Ale ty jesteś pacanem...")
                        .description("Wpisuje się poprawną liczbę lub `all` kolego.".to_string())
                        .color(0xFF0000),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    if user_id == receiver_id {
        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title("❌ Ale co ty odwalasz...")
                .description("Nie możesz przelać pieniędzy samemu sobie. To nie pranie brudnych pieniędzy.")
                .color(0xFF0000))
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    let user_data = db.ensure_member(user_id).await?;

    if user_data.user.wallet < 0 || user_data.user.bank < 0 {
        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title("❌ Najpierw napraw kasę")
                .description("Nie oszukasz mnie. Najpierw weź ustaw tak, byś ani w banku, ani w portfelu nie miał ujemnych pieniędzy.")
                .color(0xFF0000))
            .ephemeral(true)
        ).await?;

        return Ok(());
    }

    if user_data.user.wallet < amount {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("❌ Brak środków")
                        .description(format!(
                            "Nie masz tyle gotówki w portfelu! Brakuje Ci: **{}** złociszy",
                            format_number(amount - user_data.user.wallet)
                        ))
                        .color(0xFF0000),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    db.ensure_member(receiver_id).await?;

    db.transfer(user_id, receiver_id, amount).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("💸 Przelew wysłany!")
                .description(format!(
                    "Pomyślnie przekazałeś pieniądze użytkownikowi <@{}>.",
                    receiver_id
                ))
                .field("Kwota", format!("`{}` zł", format_number(amount)), true)
                .field("Nadawca", format!("<@{}>", user_id), true)
                .color(0x00FF00),
        ),
    )
    .await?;

    Ok(())
}
