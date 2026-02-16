use crate::bot::{Context, format_number::format_number};
use anyhow::Error;
use poise::CreateReply;
use rand::prelude::IndexedRandom;
use serenity::all::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
    CreateInteractionResponse,
};
use std::time::Duration;
use tokio::time::sleep;

const SYMBOLS: [&str; 6] = ["🍎", "🍋", "🍒", "🍇", "💎", "7️⃣"];

#[poise::command(
    slash_command,
    prefix_command,
    aliases("slotmachine", "automat"),
    description_localized("pl", "Spróbuj szczęścia w automatach!")
)]
pub async fn slots(
    ctx: Context<'_>,
    #[description_localized("pl", "Ile stawiasz?")] bet: i64,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;
    let gif_url =
        "https://tenor.com/view/jago33-slot-machine-slot-online-casino-medan-gif-25082594";

    let mut handle: Option<poise::ReplyHandle<'_>> = None;

    loop {
        let user_data = db.ensure_member(user_id).await?;
        if user_data.user.cash < bet {
            let error_embed = CreateEmbed::new()
                .title("🥀 Jesteś biedny")
                .description(format!(
                    "Masz tylko `{}`zł. Idź do pracy, czy coś.",
                    format_number(user_data.user.cash)
                ))
                .color(0xFF0000);

            if let Some(h) = handle {
                h.edit(
                    ctx,
                    CreateReply::default().embed(error_embed).components(vec![]),
                )
                .await?;
            } else {
                ctx.send(CreateReply::default().embed(error_embed).ephemeral(true))
                    .await?;
            }
            break;
        }

        let spinning_embed = CreateEmbed::new()
            .title("🎰 Maszyna losuje...")
            .image(gif_url)
            .color(0xFFFF00);

        let components = vec![CreateActionRow::Buttons(vec![
            CreateButton::new("spin_again")
                .label("Kręć dalej!")
                .style(ButtonStyle::Primary)
                .disabled(true),
        ])];

        if let Some(h) = handle {
            h.edit(
                ctx,
                CreateReply::default()
                    .embed(spinning_embed)
                    .components(components.clone()),
            )
            .await?;
            handle = Some(h);
        } else {
            let h = ctx
                .send(
                    CreateReply::default()
                        .embed(spinning_embed)
                        .components(components.clone()),
                )
                .await?;
            handle = Some(h);
        }

        sleep(Duration::from_secs(2)).await;

        let (s1, s2, s3) = {
            let mut rng = rand::rng();

            let s1 = SYMBOLS.choose(&mut rng).unwrap();
            let s2 = SYMBOLS.choose(&mut rng).unwrap();
            let s3 = SYMBOLS.choose(&mut rng).unwrap();

            (s1, s2, s3)
        };

        let (multiplier, message) = match (*s1, *s2, *s3) {
            ("7️⃣", "7️⃣", "7️⃣") => (50, "🎰 JACKPOT!!! SIEDEM SIEDEM SIEDEM!"),
            ("💎", "💎", "💎") => (8, "💎 DIAMENTOWY STRZAŁ!"),
            (a, b, c) if a == b && b == c => (5, "✨ Trzy w linii! Pięknie!"),
            (a, b, _) if a == b => (2, "🍒 Dwa pierwsze pasują! Mały zysk."),
            _ => (
                0,
                "💀 Pusto... Może następnym razem?\n\nPamiętaj, że 99.6% hazardzistów odchodzi przed pierwszą dużą wygraną! Ale ty nie odchodź! Ty dasz radę!",
            ),
        };

        let win_amount = bet * multiplier;
        user_data
            .user
            .change_cash(win_amount - bet, &db.pool)
            .await?;

        let result_embed = CreateEmbed::new()
            .title("🎰 Maszynka do nieśmier... inwestycyjna!")
            .description(format!(
                "# **[ {} | {} | {} ]**\n\n{}\n\n**Zakład:** {}\n**Zysk:** {}",
                s1,
                s2,
                s3,
                message,
                format_number(bet),
                format_number(win_amount)
            ))
            .color(if multiplier > 0 { 0x00FF00 } else { 0xFF0000 });

        let final_components = vec![CreateActionRow::Buttons(vec![
            CreateButton::new("spin_again")
                .label("Zagraj ponownie!")
                .style(ButtonStyle::Success)
                .disabled(false),
        ])];

        let msg_handle = handle.as_mut().unwrap();
        msg_handle
            .edit(
                ctx,
                CreateReply::default()
                    .embed(result_embed)
                    .components(final_components),
            )
            .await?;

        let interaction = ComponentInteractionCollector::new(ctx.serenity_context())
            .author_id(ctx.author().id)
            .message_id(msg_handle.message().await?.id)
            .timeout(Duration::from_secs(30))
            .await;

        if let Some(m) = interaction {
            m.create_response(
                ctx.serenity_context(),
                CreateInteractionResponse::Acknowledge,
            )
            .await?;

            continue;
        } else {
            let msg = msg_handle.message().await?;
            let last_embed = msg.embeds.first().cloned().map(CreateEmbed::from);

            let mut reply = CreateReply::default().components(vec![]);

            if let Some(e) = last_embed {
                reply = reply.embed(e);
            }

            msg_handle.edit(ctx, reply).await?;
            break;
        }
    }

    Ok(())
}
