use anyhow::Error;
use poise::CreateReply;
use rand::RngExt;
use serenity::all::{CreateEmbed, User};
use crate::bot::Context;

const COOLDOWN: i64 = 3 * 60 * 60;

#[poise::command(
    slash_command,
    prefix_command,
    name_localized("pl", "rob"),
    description_localized("pl", "Spróbuj okraść kogoś z jego ciężko zarobionej gotówki.")
)]
pub async fn rob(
    ctx: Context<'_>,
    #[description_localized("pl", "Kogo chcesz okraść?")] target_user: User,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let victim_id = target_user.id.get() as i64;
    let db = &ctx.data().db;

    if user_id == victim_id {
        ctx.say("Nie możesz okraść samego siebie, geniuszu.")
            .await?;

        return Ok(());
    }

    let user_data = db.ensure_member(user_id).await?;
    let victim_data = db.ensure_member(victim_id).await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    if now - user_data.timeouts.last_rob < COOLDOWN {
        let remaining = COOLDOWN - (now - user_data.timeouts.last_rob);

        ctx.send(CreateReply::default().embed(
            CreateEmbed::new()
                .title("❌ Co ty taki porywczy?")
                .description(format!(
                    "Musisz się przyczaić. Następny skok może być bezpieczny dopiero za {} sekund.",
                    remaining
                ))
                .color(0x00FF00)
        )).await?;

        return Ok(());
    }

    let (chance, stolen_amount, fine) = {
        let mut rng = rand::rng();

        let chance = rng.random_range(0..100);

        let percent = rng.random_range(10.0..=25.0);
        let stolen_amount = (victim_data.user.cash * percent) / 100.0;

        let fine = rng.random_range(50.00..=350.00);

        (chance, stolen_amount, fine)
    };

    db.update_timeout(user_id, "last_rob", now).await?;

    if chance < 40 {
        db.transfer(victim_id, user_id, stolen_amount).await?;

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("🥷 Udany skok!")
                    .description(format!(
                        "Zakradłeś się do portfela <@{}> i zwędziłeś mu **{}** 💰!",
                        victim_id, stolen_amount
                    ))
                    .color(0x00FF00),
            ),
        )
        .await?;
    } else {
        db.transfer(user_id, victim_id, fine).await?;

        ctx.send(CreateReply::default().embed(
            CreateEmbed::new()
                .title("🚨 Złapany na gorącym uczynku!")
                .description(format!(
                    "<@{}> cię zauważył! Podczas ucieczki upuściłeś portfel, a ofiara znalazła w nim **{}** 💰 i zabrała jako odszkodowanie.",
                    victim_id, fine
                ))
                .color(0xFF0000)
        )).await?;
    }

    Ok(())
}
