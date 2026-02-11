use ab_glyph::{FontArc, PxScale};
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rand::Rng;
use serenity::all::CreateAttachment;
use std::io::Cursor;
use std::time::Duration;

use crate::bot::{Context, Error};
use poise::{CreateReply, command};
use serenity::all::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

#[command(
    slash_command,
    prefix_command,
    description_localized("pl", "Zdrap zdrapke! 🎟️ Symbole: L, M, G, 7")
)]
pub async fn scratch(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;
    let user_id = ctx.author().id.get() as i64;
    let (member, timeouts) = db.ensure_member(user_id).await?;

    if member.cash < 2 {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Jesteś biedny")
                    .description(format!("Masz tylko `{}` dolarów.", member.cash))
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
            .embed(CreateEmbed::new()
                .title(":hourglass_flowing_sand: Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    db.remove_cash(user_id, 2).await?;

    let scratch_card = CreateAttachment::path("assets/images/scratch_card.png").await?;
    let scratch_card_name = scratch_card.filename.clone();

    let scratch_card_msg = ctx
        .send(
            CreateReply::default()
                .attachment(scratch_card)
                .embed(
                    CreateEmbed::new()
                        .title("Zdrap zdrapke! 🎟️")
                        .description("Sprawdź czy wygrałeś w najnowszym lotto...")
                        .color(0x00FF00)
                        .image(format!("attachment://{}", scratch_card_name)),
                )
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new("scratched")
                        .label("Zdrap!")
                        .style(ButtonStyle::Primary),
                ])]),
        )
        .await?;

    let message = scratch_card_msg.message().await?;
    let author_id = ctx.author().id;

    if let Some(press) = ComponentInteractionCollector::new(ctx.serenity_context().shard.clone())
        .author_id(author_id)
        .message_id(message.id)
        .custom_ids(vec!["scratched".into()])
        .timeout(Duration::from_secs(45))
        .await
    {
        let (buffer, symbols, win) =
            generate_scratch_card_in_memory("assets/images/scratch_card_scratched.png")?;

        let attachment = CreateAttachment::bytes(buffer, "scratch_card_scratched.png");

        press
            .create_response(
                ctx.serenity_context(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .add_file(attachment)
                        .embed(
                            CreateEmbed::new()
                                .title(format!(
                                    "{} Twoja zdrapka!",
                                    if win > 0 { "😀" } else { "❌" }
                                ))
                                .description(format!(
                                    "Symbole: {}\nWygrana: **{win} dolarów**",
                                    symbols.iter().collect::<String>()
                                ))
                                .color(if win > 0 { 0x00FF00 } else { 0xFF0000 })
                                .image("attachment://scratch_card_scratched.png"),
                        )
                        .components(Vec::new()),
                ),
            )
            .await?;

        if win != 0 {
            db.add_cash(user_id, win).await?;
        }
    }

    Ok(())
}

fn generate_scratch_card_in_memory(base_path: &str) -> Result<(Vec<u8>, Vec<char>, i64), Error> {
    let img = image::open(base_path)?.to_rgba8();
    let mut img_buf: RgbaImage = img.clone();

    let font = FontArc::try_from_vec(std::fs::read("assets/fonts/zdrapka.ttf")?)?;
    let symbol_scale = PxScale::from(35.0);
    let cash_scale = PxScale::from(30.0);

    let positions = vec![
        (280, 65),
        (280, 110),
        (280, 160),
        (280, 200),
        (280, 250),
        (280, 300),
        (280, 345),
    ];

    let symbols_weights = vec![
        ('0', 0.1),
        ('1', 0.1),
        ('2', 0.1),
        ('3', 0.1),
        ('4', 0.1),
        ('5', 0.1),
        ('6', 0.1),
        ('7', 0.1),
        ('8', 0.1),
        ('9', 0.1),
    ];

    let mut symbols = Vec::new();
    let mut total_prize: i64 = 0;
    let mut rng = rand::rng();

    for &(x, y) in &positions {
        let symbol = random_weighted_symbol(&symbols_weights, &mut rng);
        symbols.push(symbol);

        let field_cash: i64 = rng.random_range(3..=16);

        if symbol == '7' {
            total_prize += field_cash;
        }

        draw_text_mut(
            &mut img_buf,
            if symbol == '7' {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 0, 255])
            },
            x,
            y,
            symbol_scale,
            &font,
            &symbol.to_string(),
        );

        let cash_text = format!("{}zl", field_cash);
        draw_text_mut(
            &mut img_buf,
            Rgba([100, 100, 100, 255]),
            375,
            y,
            cash_scale,
            &font,
            &cash_text,
        );
    }

    let mut buffer: Vec<u8> = Vec::new();
    DynamicImage::ImageRgba8(img_buf)
        .write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)?;

    Ok((buffer, symbols, total_prize))
}

fn random_weighted_symbol(symbols: &Vec<(char, f64)>, rng: &mut impl Rng) -> char {
    let mut roll: f64 = rng.random();
    for &(symbol, weight) in symbols {
        if roll < weight {
            return symbol;
        }
        roll -= weight;
    }
    symbols.last().unwrap().0
}
