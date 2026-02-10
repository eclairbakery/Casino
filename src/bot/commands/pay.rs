use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    prefix_command,
    name_localized("pl", "pay"),
    aliases("daj", "przelej", "give"),
    description_localized("pl", "Przelej pieniądze innemu użytkownikowi.")
)]
pub async fn pay(
    ctx: Context<'_>,
    #[description_localized("pl", "Komu chcesz przelać pieniądze?")] receiver: serenity::User,
    #[description_localized("pl", "Ile pieniędzy chcesz przelać?")] amount: i64,
) -> Result<(), Error> {
    let sender_id = ctx.author().id.get() as i64;
    let receiver_id = receiver.id.get() as i64;
    let db = &ctx.data().db;

    if amount <= 0 {
        ctx.send(
            CreateReply::default()
                .embed(
                    serenity::CreateEmbed::new()
                        .title("❌ Ale ty jesteś pacanem...")
                        .description(format!("Wpisuje się poprawną liczbę lub `all` kolego."))
                        .color(0xFF0000),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    if sender_id == receiver_id {
        ctx.send(CreateReply::default()
            .embed(serenity::CreateEmbed::new()
                .title("❌ Ale co ty odwalasz...")
                .description("Nie możesz przelać pieniędzy samemu sobie. To nie pranie brudnych pieniędzy.")
                .color(0xFF0000))
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let (sender_mem, _) = db.ensure_member(sender_id).await?;

    if sender_mem.cash < 0 || sender_mem.bank < 0 {
        ctx.send(CreateReply::default()
            .embed(serenity::CreateEmbed::new()
                .title("❌ Najpierw napraw kasę")
                .description("Nie oszukasz mnie. Najpierw weź ustaw tak, byś ani w banku, ani w portfelu nie miał ujemnych pieniędzy.")
                .color(0xFF0000))
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    if sender_mem.cash < amount {
        ctx.send(
            CreateReply::default()
                .embed(
                    serenity::CreateEmbed::new()
                        .title("❌ Brak środków")
                        .description(format!(
                            "Nie masz tyle gotówki w portfelu! Brakuje Ci: **{}** 💰",
                            amount - sender_mem.cash
                        ))
                        .color(0xFF0000),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    db.ensure_member(receiver_id).await?;

    db.transfer(sender_id, receiver_id, amount).await?;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("💸 Przelew wysłany!")
                .description(format!(
                    "Pomyślnie przekazałeś pieniądze użytkownikowi <@{}>.",
                    receiver_id
                ))
                .field("Kwota", format!("`{}` 💰", amount), true)
                .field("Nadawca", format!("<@{}>", sender_id), true)
                .color(0x00FF00),
        ),
    )
    .await?;

    Ok(())
}
