use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use rand::Rng;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("cf"),
    description_localized("pl", "Rzuć monetą o hajs!")
)]
pub async fn coinflip(
    ctx: Context<'_>,
    side: String,
    bet: i64,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let side_lower = side.to_lowercase();
    if side_lower != "heads" && side_lower != "tails" && side_lower != "h" && side_lower != "t" && side_lower != "o" && side_lower != "r" {
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ Ej no, ale weź coś podaj")
                .description("Podajesz `heads` (h) lub `tails` (t). To tyle.")
                .color(0xFF0000)
        )).await?;
        return Ok(());
    }

    if bet < 5 {
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ Co ty odwalasz?")
                .description("Weź postaw te 5 co najmniej.")
                .color(0xFF0000)
        )).await?;
        return Ok(());
    }

    if bet > 50 {
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ Kasyno musi zarabiać...")
                .description("Ja Ci nie pozwolę postawić więcej niż 50, bo to dla mnie strata będzie.")
                .color(0xFF0000)
        )).await?;
        return Ok(());
    }

    let (member, timeouts) = db.ensure_member(user_id).await?;
    if member.cash < bet {
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ Jesteś biedny")
                .description(format!("Masz tylko `{}` 💵 niestety.", member.cash))
                .color(0xFF0000)
        )).await?;
        return Ok(());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;

    let cooldown = 15;
    let time_passed = now - timeouts.last_hazarded;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;
        ctx.send(CreateReply::default()
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title("⏳ Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    let roll = rand::rng().random_range(0..2);
    let result_name = if roll == 0 { "heads" } else { "tails" };
    let result_display = if roll == 0 { "🦅 **Orzeł**" } else { "🪙 **Reszka**" };

    let won = (side_lower == "h" && roll == 0) || 
              (side_lower == "t" && roll == 1) || 
              (side_lower == "o" && roll == 0) ||
              (side_lower == "r" && roll == 1) ||
              (side_lower == result_name);

    if won {
        let profit = (bet as f64 * 0.9) as i64;
        db.add_cash(user_id, profit).await?;

        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("🎉 Zwycięzca!")
                .description(format!("Wynik: {}\n\nDostałeś **{}** dolarów!", result_display, profit))
                .color(0x00FF00)
        )).await?;
    } else {
        db.add_cash(user_id, -bet).await?;

        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("💀 Przegrany...")
                .description(format!("Wynik: {}\n\nStraciłeś **{}** dolarów.", result_display, bet))
                .color(0xFF0000)
        )).await?;
    }

    Ok(())
}
