use crate::bot::Context;
use anyhow::Error;
use poise::CreateReply;
use rand::RngExt;
use rand::prelude::IndexedRandom;
use serenity::all::CreateEmbed;
use std::time;
use std::time::SystemTime;

const COOLDOWN: i64 = 30;

const RESPONSES: [&str; 4] = [
    "Skosiłeś trawnik u sąsiada. Jest on wdzięczny i zapłacił ci okrągłe {amount} dolarów!",
    "Sprzedawałeś lemoniadę na rogu. Mało oryginalna praca i duża konkurencja ze strony darmozja... znaczy, kolegów, ale zarobiłeś {amount} dolarów.",
    "Jakiś gość ci zapłacił {amount} dolarów za naprawę komputera, gdzie po prostu trzeba było wywalić bloatware z menu start. :wilted_rose:",
    "Schronisko dla psów się odezwało i zaoferowało {amount} dolarów za sprzątanie po psich kupach, a ty zaakceptowałeś tą ofertę i to zrobiłeś.",
];

#[poise::command(
    slash_command,
    prefix_command,
    description_localized("pl", "Nie odpoczywaj. Pracuj obywatelu. Na korzyść państwa!")
)]
pub async fn work(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let user_data = db.ensure_member(user_id).await?;

    let now = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_secs() as i64;

    let time_passed = now - user_data.timeouts.last_work;

    if time_passed < COOLDOWN {
        let remaining = COOLDOWN - time_passed;

        ctx.send(CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("⏳ Nie możesz stać się pracoholikiem.")
                    .description(format!("Twoi koledzy powiedzieli ci, że \"nadgorliwość gorsza od faszyzmu\". Pewnie dlatego, że nie chcą, by szefowstwo zobaczyło co umiesz i zaczęło więcej wymagać. Nie chcą pracować ig. Ty niestety nie masz odwagi by im się sprzeciwić. :wilted_rose:\n\nWróć za: **{} sekund**.", remaining))
                    .color(0xFF0000))
        ).await?;

        return Ok(());
    }

    let (how_much, desc) = {
        let mut rng = rand::rng();

        let how_much = rng.random_range(24.66..200.00);

        let desc_templ = RESPONSES
            .choose(&mut rng)
            .unwrap_or(&"message się zepsuł :wilted_rose: ale zarobiłeś {amount}");

        let desc = desc_templ.replace("{amount}", &how_much.to_string());

        (how_much, desc)
    };

    db.add_cash(user_id, how_much).await?;
    db.update_timeout(user_id, "last_work", now).await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("⚒️ Udało się!")
                .description(desc)
                .color(0x00FF00),
        ),
    )
    .await?;

    Ok(())
}
