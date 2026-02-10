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
    // --- 1. Wyślij bazową zdrapkę ---
    let scratch_card = CreateAttachment::path("assets/images/scratch_card.png").await?;
    let scratch_card_name = scratch_card.filename.clone();

    let scratch_card_msg = ctx
        .send(
            CreateReply::default()
                .attachment(scratch_card)
                .embed(
                    CreateEmbed::new()
                        .title("Zdrap zdrapke! 🎟️")
                        .description("Kliknij 'Zdrap!', aby odsłonić symbole.")
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

    // --- 2. Czekamy na kliknięcie ---
    if let Some(press) = ComponentInteractionCollector::new(ctx.serenity_context().shard.clone())
        .author_id(author_id)
        .message_id(message.id)
        .custom_ids(vec!["scratched".into()])
        .timeout(Duration::from_secs(45))
        .await
    {
        // --- 3. Generowanie scratched w pamięci ---
        let (buffer, symbols) =
            generate_scratch_card_in_memory("assets/images/scratch_card_scratched.png")?;

        let attachment = CreateAttachment::bytes(buffer, "scratch_card_scratched.png");

        // --- 4. Odpowiedź z wylosowanymi symbolami ---
        press
            .create_response(
                ctx.serenity_context(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .add_file(attachment)
                        .embed(
                            CreateEmbed::new()
                                .title("Zdrap zdrapke! 🎟️")
                                .description(format!(
                                    "Symbole: {}",
                                    symbols.iter().collect::<String>()
                                ))
                                .color(0x00FF00)
                                .image("attachment://scratch_card_scratched.png"),
                        )
                        .components(Vec::new()),
                ),
            )
            .await?;
    }

    Ok(())
}

/// Funkcja generująca scratched w pamięci i zwracająca bufor PNG oraz symbole
fn generate_scratch_card_in_memory(base_path: &str) -> Result<(Vec<u8>, Vec<char>), Error> {
    // Wczytanie obrazka scratched (tło)
    let img = image::open(base_path)?.to_rgba8();
    let mut img_buf: RgbaImage = img.clone();

    // Font kompatybilny z draw_text_mut
    let font = FontArc::try_from_vec(std::fs::read("assets/fonts/zdrapka.ttf")?)?;
    let scale = PxScale::from(30.0);

    // POSITIONS
    // the image is 450px x 450px
    let positions = vec![
        (375, 65),
        (150, 50),
        (250, 50),
        (50, 150),
        (150, 150),
        (250, 150),
    ];

    // Symbole i wagi
    let symbols_weights = vec![('L', 0.4), ('M', 0.3), ('G', 0.25), ('7', 0.05)];

    // Losowanie symboli i rysowanie ich
    let mut symbols = Vec::new();
    let mut rng = rand::rng();
    for &(x, y) in &positions {
        let symbol = random_weighted_symbol(&symbols_weights, &mut rng);
        symbols.push(symbol);
        draw_text_mut(
            &mut img_buf,
            Rgba([0, 0, 0, 255]),
            x,
            y,
            scale,
            &font,
            &symbol.to_string(),
        );
    }

    // Konwersja do PNG w pamięci
    let mut buffer: Vec<u8> = Vec::new();
    DynamicImage::ImageRgba8(img_buf)
        .write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)?;

    Ok((buffer, symbols))
}

/// Losowanie symbolu według wag
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
