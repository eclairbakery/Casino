use poise::CreateReply;
use crate::bot::{Context, Error};
use rand::Rng;
use rand::prelude::IndexedRandom;

#[poise::command(
    slash_command,
    prefix_command,
    name_localized("pl", "pracuj"),
    description_localized("pl", "Pracuj ciężko na swój chleb.")
)]
pub async fn work(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let (_member, timeouts) = db.ensure_member(user_id).await?;
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;

    let cooldown = 30;
    let time_passed = now - timeouts.last_work;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;
        ctx.send(CreateReply::default()
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title("⏳ Nie możesz stać się pracoholikiem.")
                .description(format!("Twoi koledzy powiedzieli ci, że \"nadgorliwość gorsza od faszyzmu\". Pewnie dlatego, że nie chcą, by szefowstwo zobaczyło co umiesz i zaczęło więcej wymagać. Nie chcą pracować ig. Ty niestety nie masz odwagi by im się sprzeciwić. :wilted_rose:\n\nWróć za: **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    let how_much = rand::rng().random_range(100..=300);
    let responses = vec![
        "Skosiłeś trawnik u sąsiada. Jest on wdzięczny i zapłacił ci okrągłe {amount} dolarów!",
        "Sprzedawałeś lemoniadę na rogu. Mało oryginalna praca i duża konkurencja ze strony darmozja... znaczy, kolegów, ale zarobiłeś {amount} dolarów.",
        "Jakiś gość ci zapłacił {amount} dolarów za naprawę komputera, gdzie po prostu trzeba było wywalić bloatware z menu start. :wilted_rose:",
        "Schronisko dla psów się odezwało i zaoferowało {amount} dolarów za sprzątanie po psich kupach, a ty zaakceptowałeś tą ofertę i to zrobiłeś."
    ];
    let desc_templ = responses.choose(&mut rand::rng()).unwrap_or(&"message się zepsuł :wilted_rose: ale zarobiłeś {amount}");
    let desc = desc_templ.replace("{amount}", &how_much.to_string());

    db.add_cash(user_id, how_much).await?;
    db.update_timeout(user_id, "last_work", now).await?;

    ctx.send(CreateReply::default()
        .embed(poise::serenity_prelude::CreateEmbed::new()
            .title("⚒️ Udało się!")
            .description(desc)
            .color(0x00FF00)
        )
    ).await?;

    Ok(())
}