use crate::bot::Context;
use anyhow::Error;
use poise::CreateReply;
use rand::RngExt;
use serenity::all::CreateEmbed;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("kostka", "d"),
    description_localized(
        "pl",
        "Możesz rucić kością; nietypową bo od 1 do 100, ale dalej. Wynik powyżej 60 wygrywa!"
    )
)]
pub async fn dice(
    ctx: Context<'_>,
    #[description_localized("pl", "Gadasz w tej chwili ile stawiasz.")] bet: f64,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    if bet <= 0.00 {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Weź chociaż trochę postaw...")
                    .description("Stawka musi być większa niż 0.00.")
                    .color(0xFF0000),
            ),
        )
        .await?;

        return Ok(());
    }

    let user_data = db.ensure_member(user_id).await?;
    if user_data.user.cash < bet {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Jesteś biedny")
                    .description(format!(
                        "Nie masz tyle kasy! Posiadasz: `{}` dolarów.",
                        user_data.user.cash
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
    let time_passed = now - user_data.timeouts.last_hazarded;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;
        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title(":hourglass_flowing_sand: Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;

        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    let (won, roll) = {
        let mut rng = rand::rng();

        let roll = rng.random_range(1..=6);
        (roll == 6 || roll == 1, roll)
    };

    let mut embed = CreateEmbed::new().title("🎲 EDCM - Extended Dice Casino Machine (1-100)");

    if won {
        let profit = bet;
        db.change_cash(user_id, profit).await?;

        embed = embed
            .description(format!(
                "# {}\n\nGratulacje! Wygrałeś **{}** dolarów!",
                roll, profit
            ))
            .color(0x00FF00);
    } else {
        db.change_cash(user_id, -bet).await?;

        embed = embed
            .description(format!("# {}\n\nNiestety, przegrałeś **{}** dolców. Musisz wyrzucić co najmniej 60.\n\n**Pamiętaj, że 99.6% hazardzistów odchodzi przed pierwszą dużą wygraną! Ty nie rezygnuj. Ty dasz radę!**", roll, bet))
            .color(0xFF0000);
    }

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
