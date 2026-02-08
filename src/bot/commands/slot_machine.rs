use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use rand::prelude::IndexedRandom;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("slotmachine", "automat"),
    description_localized("pl", "Spróbuj szczęścia w automatach!"),
)]
pub async fn slots(
    ctx: Context<'_>,
    #[description_localized("pl", "Ile stawiasz?")] bet: i64,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    if bet < 100 {
        ctx.send(CreateReply::default()
            .embed(serenity::CreateEmbed::new()
                .title("❌ Nie ma zysków bez ryzyka")
                .description(format!("Weź chociaż te 100 postaw."))
                .color(0xFF0000))
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    let (member, timeouts) = db.ensure_member(user_id).await?;
    if member.cash < bet {
        ctx.send(CreateReply::default()
            .embed(serenity::CreateEmbed::new()
                .title("❌ Jesteś biedny")
                .description(format!("Nie masz tyle gotówki w portfelu!\nPosiadasz: `{}` 💵", member.cash))
                .color(0xFF0000))
            .ephemeral(true)
        ).await?;
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
                .title(":hourglass_flowing_sand: Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    let symbols = vec!["🍎", "🍋", "🍒", "🍇", "💎", "7️⃣"];
    
    let s1 = *symbols.choose(&mut rand::rng()).unwrap();
    let s2 = *symbols.choose(&mut rand::rng()).unwrap();
    let s3 = *symbols.choose(&mut rand::rng()).unwrap();

    let (multiplier, message) = match (s1, s2, s3) {
        ("7️⃣", "7️⃣", "7️⃣") => (50, "🎰 JACKPOT!!! SIEDEM SIEDEM SIEDEM!"),
        ("💎", "💎", "💎") => (8, "💎 DIAMENTOWY STRZAŁ!"),
        (a, b, c) if a == b && b == c => (5, "✨ Trzy w linii! Pięknie!"),
        (a, b, _) if a == b => (2, "🍒 Dwa pierwsze pasują! Mały zysk."),
        _ => (0, "💀 Pusto... Może następnym razem?\n\nPamiętaj, że 99.6% hazardzistów odchodzi przed pierwszą dużą wygraną! Ty nie rezygnuj. Ty dasz radę!"),
    };

    let win_amount = bet * multiplier;
    let net_change = win_amount - bet;

    db.add_cash(user_id, net_change).await?;

    let color = if multiplier > 0 { 0x00FF00 } else { 0xFF0000 };
    
    ctx.send(CreateReply::default()
        .embed(serenity::CreateEmbed::new()
            .title("🎰 Maszynka do nieśmiertel... inwestycyjna!")
            .description(format!(
                "# **[ {} | {} | {} ]**\n\n{}\n\n**Zakład:** {}\n**Zysk:** {}",
                s1, s2, s3, message, bet, win_amount
            ))
            .color(color)
        )
    ).await?;

    Ok(())
}
