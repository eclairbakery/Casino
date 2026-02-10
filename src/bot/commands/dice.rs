use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use rand::Rng;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("kostka", "d"),
    description_localized(
        "pl",
        "Możesz rucić kością; nietypową bo od 1 do 100, ale dalej. Wynik powyżej 55 wygrywa!"
    )
)]
pub async fn dice(ctx: Context<'_>, bet: i64) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    if bet <= 50 {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("❌ Weź chociaż trochę postaw...")
                    .description("Stawka musi być większa niż 50.")
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
                    .description(format!(
                        "Nie masz tyle kasy! Posiadasz: `{}` dolarów.",
                        member.cash
                    ))
                    .color(0xFF0000),
            ),
        )
        .await?;
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
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title(":hourglass_flowing_sand: Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    let roll = rand::rng().random_range(1..=100);
    let won = roll > 60;

    let mut embed =
        serenity::CreateEmbed::new().title("🎲 EDCM - Extended Dice Casino Machine (1-100)");

    if won {
        let profit = bet;
        db.add_cash(user_id, profit).await?;

        embed = embed
            .description(format!(
                "# {}\n\nGratulacje! Wygrałeś **{}** dolarów!",
                roll, profit
            ))
            .color(0x00FF00);
    } else {
        db.add_cash(user_id, -bet).await?;

        embed = embed
            .description(format!("# {}\n\nNiestety, przegrałeś **{}** dolców. Musisz wyrzucić co najmniej 60.\n\n**Pamiętaj, że 99.6% hazardzistów odchodzi przed pierwszą dużą wygraną! Ty nie rezygnuj. Ty dasz radę!**", roll, bet))
            .color(0xFF0000);
    }

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
