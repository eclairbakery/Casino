use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use rand::Rng;

#[poise::command(
    slash_command,
    prefix_command,
    name_localized("pl", "rob"),
    description_localized("pl", "Spróbuj okraść kogoś z jego ciężko zarobionej gotówki.")
)]
pub async fn rob(
    ctx: Context<'_>,
    #[description_localized("pl", "Kogo chcesz okraść?")] target_user: serenity::User,
) -> Result<(), Error> {
    let thief_id = ctx.author().id.get() as i64;
    let victim_id = target_user.id.get() as i64;
    let db = &ctx.data().db;

    if thief_id == victim_id {
        ctx.say("Nie możesz okraść samego siebie, geniuszu.")
            .await?;
        return Ok(());
    }

    let (_, thief_tm) = db.ensure_member(thief_id).await?;
    let (victim_mem, _) = db.ensure_member(victim_id).await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let cooldown = 3 * 60 * 60;

    if now - thief_tm.last_rob < cooldown {
        let remaining = cooldown - (now - thief_tm.last_rob);
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ Co ty taki porywczy?")
                .description(format!(
                    "Musisz się przyczaić. Następny skok może być bezpieczny dopiero za {} sekund.",
                    remaining
                ))
                .color(0x00FF00)
        )).await?;
        return Ok(());
    }

    if victim_mem.cash < 100 {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("❌ Nie do tej osoby")
                    .description("Ta osoba jest zbyt biedna, nie warto nawet wyciągać łomu.")
                    .color(0x00FF00),
            ),
        )
        .await?;
        return Ok(());
    }

    let chance = rand::rng().random_range(0..100);

    db.update_timeout(thief_id, "last_rob", now).await?;

    if chance < 40 {
        let percent = rand::rng().random_range(10..=50);
        let stolen_amount = (victim_mem.cash * percent) / 100;

        db.transfer(victim_id, thief_id, stolen_amount).await?;

        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
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
        let fine = rand::rng().random_range(100..=300);

        db.transfer(thief_id, victim_id, fine).await?;

        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
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
