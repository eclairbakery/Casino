use crate::bot::{Context, format_number::format_number};
use anyhow::Error;
use poise::CreateReply;
use poise::futures_util::StreamExt;
use poise::serenity_prelude::ButtonStyle;
use rand::RngExt;
use serenity::all::{
    ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
    CreateInteractionResponse,
};
use std::time::{Duration, SystemTime};

fn remove_player(ctx: &Context, user_id: &i64) {
    match ctx.data().active_players.lock() {
        Ok(mut active_players) => {
            active_players.remove(user_id);
        }
        Err(_err) => {
            todo!()
        }
    }
}

#[poise::command(
    slash_command,
    prefix_command,
    description_localized(
        "pl",
        "Zainwestuj pieniądze w shady akcje i patrz jak rosną... Ucieknij, zanim spadną na łeb na szyję."
    )
)]
pub async fn crash(
    ctx: Context<'_>,
    #[description_localized("pl", "Gadasz w tej chwili ile stawiasz.")] bet: i64,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let already_playing = {
        let active = ctx.data().active_players.lock().map_err(|_| "Mutex error");

        match active {
            Ok(mut active) => {
                if active.contains(&user_id) {
                    true
                } else {
                    active.insert(user_id);
                    false
                }
            }
            Err(_) => todo!(),
        }
    };

    if already_playing {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Ale co ty odwalasz?")
                    .description("Dokończ tą poprzednią grę w tej chwili!")
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    let user_data = db.ensure_member(user_id).await?;

    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let cooldown = 15;

    if user_data.user.cash < bet || bet <= 0 {
        remove_player(&ctx, &user_id);

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Jesteś biedny!")
                    .description("Nie masz tyle kasy, pajacu...")
                    .color(0xFFC0000),
            ),
        )
        .await?;

        return Ok(());
    }

    if now - user_data.timeouts.last_hazarded < cooldown {
        let remaining = cooldown - (now - user_data.timeouts.last_hazarded);

        remove_player(&ctx, &user_id);

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
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
    user_data.user.change_cash(-bet, &db.pool).await?;

    let mut multiplier = 0.1;
    let ctx_id = ctx.id();
    let stop_id = format!("{}stop", ctx_id);

    let embed = CreateEmbed::new()
        .title("🚀 Crash")
        .description(format!(
            "Mnożnik: **{:.2}x**\nZysk: **{:.0}**zł!",
            multiplier,
            format_number((bet as f64 * multiplier) as i64 - bet)
        ))
        .color(0xFFFF00);

    let reply = ctx
        .send(
            CreateReply::default()
                .embed(embed)
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(&stop_id)
                        .label("WYPŁAĆ")
                        .style(ButtonStyle::Success),
                ])]),
        )
        .await?;

    let author_id = ctx.author().id;

    let mut collector = ComponentInteractionCollector::new(ctx)
        .filter(move |p| p.data.custom_id == stop_id && p.user.id == author_id)
        .timeout(Duration::from_secs(120))
        .stream();

    let mut won = false;

    loop {
        tokio::select! {
            Some(press) = collector.next() => {
                let _ = press.create_response(ctx, CreateInteractionResponse::Acknowledge).await;
                won = true;
                break;
            }

            _ = tokio::time::sleep(Duration::from_millis(250)) => {
                let crash_chance = if multiplier < 1.0 { 2 } else if multiplier < 2.0 { 10 } else if multiplier < 5.0 { 18 } else { 30 };

                if rand::rng().random_range(0..100) < crash_chance {
                    break;
                }

                multiplier += if multiplier < 3.0 { 0.1 } else if multiplier < 5.0 { 0.2 } else { 0.5 };

                let _ = reply.edit(ctx, CreateReply::default()
                    .embed(CreateEmbed::new()
                        .title("🚀 Crash")
                        .description(format!("Mnożnik: **{:.2}x**\nZysk: **{:.0}**zł!", multiplier, format_number((bet as f64 * multiplier) as i64 - bet )))
                        .color(0xFFFF00)
                    )
                ).await;
            }
        }
    }

    let final_embed = if won {
        let win_amount = (bet as f64 * multiplier) as i64;

        user_data.user.change_cash(win_amount, &db.pool).await?;
        if win_amount < bet {
            CreateEmbed::new()
                .title("💥 Jesteś dzbanem!")
                .description(format!(
                    "Wyszedłeś przy **{:.2}x**, czyli straciłeś **{}**zł.",
                    multiplier,
                    format_number(bet - win_amount)
                ))
                .color(0xFF0000)
        } else {
            CreateEmbed::new()
                .title("📈 Zysk!")
                .description(format!(
                    "Wypłacono przy **{:.2}x**!\nWygrałeś **{}**zł!",
                    multiplier,
                    format_number(win_amount - bet)
                ))
                .color(0x00FF00)
        }
    } else {
        CreateEmbed::new()
            .title("💥 BOOM!")
            .description(format!(
                "Wszystko się j*bło przy **{:.2}x**!\nStraciłeś **{}**zł, które użyłeś na ten zakład.",
                multiplier, format_number(bet)
            ))
            .color(0xFF0000)
    };

    let _ = reply
        .edit(
            ctx,
            CreateReply::default().embed(final_embed).components(vec![]),
        )
        .await;

    remove_player(&ctx, &user_id);

    Ok(())
}
