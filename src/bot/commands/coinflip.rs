use crate::bot::Context;
use crate::bot::format_number::format_number;
use anyhow::Error;
use poise::CreateReply;
use rand::RngExt;
use serenity::all::CreateEmbed;
use std::time;
use std::time::SystemTime;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("cf"),
    description_localized("pl", "Rzuć monetą o hajs z BLIKiem (jk)!")
)]
pub async fn coinflip(
    ctx: Context<'_>,
    #[description_localized("pl", "Dajesz stronę. Albo H albo T. Wygrywasz lub nie.")] side: String,
    #[description_localized("pl", "Gadasz w tej chwili ile stawiasz.")] bet: i64,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let side_lower = side.to_lowercase();
    let is_heads = side_lower == "heads" || side_lower == "h" || side_lower == "o";
    let is_tails = side_lower == "tails" || side_lower == "t" || side_lower == "r";

    if !is_heads && !is_tails {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Wybierz stronę")
                    .description("Musisz wybrać `heads` (h) lub `tails` (t).")
                    .color(0xFF0000),
            ),
        )
        .await?;

        return Ok(());
    }

    if bet <= 0 {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Za mało!")
                    .description("Nice try.")
                    .color(0xFF0000),
            ),
        )
        .await?;

        return Ok(());
    }

    let user_data = db.ensure_member(user_id).await?;

    if user_data.user.wallet < bet {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Jesteś biedny")
                    .description(format!(
                        "Masz tylko `{}`zł.",
                        format_number(user_data.user.wallet)
                    ))
                    .color(0xFF0000),
            ),
        )
        .await?;

        return Ok(());
    }

    if (user_data.user.wallet + user_data.user.bank) > 1000_00 {
        ctx.send(CreateReply::default().embed(
            CreateEmbed::new()
                .title("❌ To jest zbyt OP")
                .description("Ta gra nie ma sensu, gdy wyszedłeś z początkowej fazy bo dość łatwo jest dostać absurdalnie duże pieniądze.")
                .color(0xFF0000)
        )).await?;

        return Ok(());
    }

    let now = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_secs() as i64;

    let cooldown = 15;
    let time_passed = now - user_data.timeouts.last_hazarded;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;

        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title("⏳ Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;

        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    let chance = rand::rng().random_range(1..=100);

    let player_won = chance <= 47;

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
        user_data.user.change_wallet(&db.pool, profit).await?;

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("🎉 Wygrana!")
                    .description(format!(
                        "Wynik: {}\n\nWygrałeś **{}**zł!",
                        result_display,
                        format_number(profit)
                    ))
                    .color(0x00FF00),
            ),
        )
        .await?;
    } else {
        user_data.user.change_wallet(&db.pool, -bet).await?;

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("💀 Przegrana")
                    .description(format!(
                        "Wynik: {}\n\nStraciłeś **{}**zł.",
                        result_display,
                        format_number(bet)
                    ))
                    .color(0xFF0000),
            ),
        )
        .await?;
    }

    Ok(())
}
