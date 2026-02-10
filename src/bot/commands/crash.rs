use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::futures_util::StreamExt;
use poise::serenity_prelude as serenity;
use rand::Rng;
use std::time::Duration;

#[poise::command(
    slash_command,
    prefix_command,
    description_localized(
        "pl",
        "Zainwestuj pieniądze w shady akcje i patrz jak rosną... Ucieknij, zanim spadną na łeb na szyję."
    )
)]
pub async fn crash(ctx: Context<'_>, bet: i64) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let already_playing = {
        let mut active = ctx
            .data()
            .active_players
            .lock()
            .map_err(|_| "Mutex error")?;
        if active.contains(&user_id) {
            true
        } else {
            active.insert(user_id);
            false
        }
    };

    if already_playing {
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("❌ Ale co ty odwalasz?")
                    .description("Dokończ tą poprzednią grę w tej chwili!")
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    let (member, timeouts) = db.ensure_member(user_id).await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let cooldown = 15;

    if member.cash < bet || bet <= 0 {
        ctx.data().active_players.lock().unwrap().remove(&user_id);
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("❌ Jesteś biedny!")
                    .description("Nie masz tyle kasy, pajacu...")
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    if now - timeouts.last_hazarded < cooldown {
        let remaining = cooldown - (now - timeouts.last_hazarded);
        ctx.data().active_players.lock().unwrap().remove(&user_id);
        ctx.send(
            CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("⏳ Czekaj chwile")
                    .description(format!(
                        "Kasyno zawsze wygrywa. A przynajmniej tak ma być. Wróć za {}s",
                        remaining
                    ))
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;
    db.add_cash(user_id, -bet).await?;

    let mut multiplier = 1.0;
    let ctx_id = ctx.id();
    let stop_id = format!("{}stop", ctx_id);

    let embed = serenity::CreateEmbed::new()
        .title("🚀 Crash")
        .description(format!(
            "Mnożnik: **{:.2}x**\nZysk: **{:.0}** 💰",
            multiplier,
            bet as f64 * multiplier
        ))
        .color(0xFFFF00);

    let reply = ctx
        .send(CreateReply::default().embed(embed).components(vec![
            serenity::CreateActionRow::Buttons(vec![
                    serenity::CreateButton::new(&stop_id)
                        .label("WYPŁAĆ")
                        .style(serenity::ButtonStyle::Success),
                ]),
        ]))
        .await?;

    let author_id = ctx.author().id;

    let mut collector = serenity::ComponentInteractionCollector::new(ctx)
        .filter(move |p| p.data.custom_id == stop_id && p.user.id == author_id)
        .timeout(Duration::from_secs(120))
        .stream();

    let mut won = false;

    loop {
        tokio::select! {
            // break
            Some(press) = collector.next() => {
                let _ = press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await;
                won = true;
                break;
            }

            // continue
            _ = tokio::time::sleep(Duration::from_millis(1500)) => {
                let crash_chance = if multiplier < 2.0 { 10 } else if multiplier < 5.0 { 18 } else { 30 };

                if rand::rng().random_range(0..100) < crash_chance {
                    break;
                }

                // if less than 3 then getting more money is harder
                multiplier += if multiplier < 3.0 { 0.2 } else { 0.5 };

                let _ = reply.edit(ctx, CreateReply::default()
                    .embed(serenity::CreateEmbed::new()
                        .title("🚀 Crash")
                        .description(format!("Mnożnik: **{:.2}x**\nZysk: **{:.0}** dolarów!", multiplier, bet as f64 * multiplier))
                        .color(0xFFFF00)
                    )
                ).await;
            }
        }
    }

    let final_embed = if won {
        if multiplier > 1.4 {
            let win_amount = (bet as f64 * multiplier) as i64;
            db.add_cash(user_id, win_amount).await?;
            serenity::CreateEmbed::new()
                .title("📈 Zysk!")
                .description(format!(
                    "Wypłacono przy **{:.2}x**!\nWygrałeś **{}** dolarów!",
                    multiplier, win_amount
                ))
                .color(0x00FF00)
        } else {
            serenity::CreateEmbed::new()
                .title("🥀 Coś za szybko!")
                .description(format!(
                    "Próbowałeś wypłacić przy **{:.2}x**! To niestety zbyt szybko. Ja nie przyłożę ręki do glitchu z money duplication. Jako karę no to wszystko straciłeś...", multiplier
                ))
                .color(0xFFFF00)
        }
    } else {
        serenity::CreateEmbed::new()
            .title("💥 BOOM!")
            .description(format!(
                "Wszystko się j*bło przy **{:.2}x**!\nStraciłeś **{}** dolarów.",
                multiplier, bet
            ))
            .color(0xFF0000)
    };

    let _ = reply
        .edit(
            ctx,
            CreateReply::default().embed(final_embed).components(vec![]),
        )
        .await;
    ctx.data().active_players.lock().unwrap().remove(&user_id);

    Ok(())
}
