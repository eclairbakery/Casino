use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use rand::prelude::IndexedRandom;
use std::time::Duration;

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
        let (member, _) = db.ensure_member(user_id).await?;
        if member.cash < bet {
            let error_embed = serenity::CreateEmbed::new()
                .title("🥀 Jesteś biedny")
                .description(format!(
                    "Masz tylko `{}` dolarów. Idź do pracy, czy coś.",
                    member.cash
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

        let spinning_embed = serenity::CreateEmbed::new()
            .title("🎰 Maszyna losuje...")
            .image(gif_url)
            .color(0xFFFF00);

        let components = vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("spin_again")
                .label("Kręć dalej!")
                .style(serenity::ButtonStyle::Primary)
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

        tokio::time::sleep(Duration::from_secs(2)).await;

        let symbols = ["🍎", "🍋", "🍒", "🍇", "💎", "7️⃣"];
        let s1 = *symbols.choose(&mut rand::rng()).unwrap();
        let s2 = *symbols.choose(&mut rand::rng()).unwrap();
        let s3 = *symbols.choose(&mut rand::rng()).unwrap();

        let (multiplier, message) = match (s1, s2, s3) {
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
        db.add_cash(user_id, win_amount - bet).await?;

        let result_embed = serenity::CreateEmbed::new()
            .title("🎰 Maszynka do nieśmier... inwestycyjna!")
            .description(format!(
                "# **[ {} | {} | {} ]**\n\n{}\n\n**Zakład:** {}\n**Zysk:** {}",
                s1, s2, s3, message, bet, win_amount
            ))
            .color(if multiplier > 0 { 0x00FF00 } else { 0xFF0000 });

        let final_components = vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("spin_again")
                .label("Zagraj ponownie!")
                .style(serenity::ButtonStyle::Success)
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

        let interaction = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
            .author_id(ctx.author().id)
            .message_id(msg_handle.message().await?.id)
            .timeout(Duration::from_secs(30))
            .await;

        if let Some(m) = interaction {
            m.create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::Acknowledge,
            )
            .await?;
            continue;
        } else {
            let msg = msg_handle.message().await?;
            let last_embed = msg.embeds.first().cloned().map(serenity::CreateEmbed::from);

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
