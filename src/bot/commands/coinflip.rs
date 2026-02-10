use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use rand::Rng;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("cf"),
    description_localized("pl", "Rzuć monetą o hajs z BLIKiem (jk)!")
)]
pub async fn coinflip(ctx: Context<'_>, side: String, bet: i64) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let side_lower = side.to_lowercase();
    let is_heads = side_lower == "heads" || side_lower == "h" || side_lower == "o";
    let is_tails = side_lower == "tails" || side_lower == "t" || side_lower == "r";

    if !is_heads && !is_tails {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("❌ Wybierz stronę")
                    .description("Musisz wybrać `heads` (h) lub `tails` (t).")
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    if bet < 5 {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("❌ Za mało!")
                    .description("Minimalna stawka to 5 dolarów.")
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    let (member, timeouts) = db.ensure_member(user_id).await?;
    if member.cash < bet {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("❌ Jesteś biedny")
                    .description(format!("Masz tylko `{}` dolarów.", member.cash))
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    if (member.cash + member.bank) > 1000 {
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ To jest zbyt OP")
                .description("Ta gra nie ma sensu, gdy wyszedłeś z początkowej fazy bo dość łatwo jest dostać absurdalnie duże pieniądze.")
                .color(0xFF0000)
        )).await?;
        return Ok(());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let cooldown = 15;
    let time_passed = now - timeouts.last_hazarded;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;
        ctx.send(CreateReply::default()
            .embed(serenity::CreateEmbed::new()
                .title("⏳ Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    let chance = rand::rng().random_range(1..=100);

    let player_won = chance <= 47;
    // actually when you drop a coin, you have a higher chance for it to land
    // on one side, depending on from what side it was dropped (ahh this english),
    // so it's totally fair.

    let result_display = if player_won {
        if is_heads {
            "🦅 **Orzeł**"
        } else {
            "🪙 **Reszka**"
        }
    } else if is_heads {
        "🪙 **Reszka**"
    } else {
        "🦅 **Orzeł**"
    };

    if player_won {
        let profit = bet;
        db.add_cash(user_id, profit).await?;

        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("🎉 Wygrana!")
                    .description(format!(
                        "Wynik: {}\n\nWygrałeś **{}** dolarów!",
                        result_display, profit
                    ))
                    .color(0x00FF00),
            ),
        )
        .await?;
    } else {
        db.add_cash(user_id, -bet).await?;

        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("💀 Przegrana")
                    .description(format!(
                        "Wynik: {}\n\nStraciłeś **{}** dolarów.",
                        result_display, bet
                    ))
                    .color(0xFF0000),
            ),
        )
        .await?;
    }

    Ok(())
}
