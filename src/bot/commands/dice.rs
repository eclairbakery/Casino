use crate::bot::Context;
use anyhow::Error;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, PngEncoder};
use image::imageops::FilterType;
use image::{
    DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageEncoder, ImageFormat, Rgb,
    RgbImage, Rgba, imageops,
};
use poise::{CreateReply, ReplyHandle, command};
use rand::RngExt;
use serenity::all::{
    ButtonStyle, ComponentInteraction, ComponentInteractionCollector, CreateAttachment,
    CreateButton, CreateEmbed, EditInteractionResponse, Interaction,
};
use serenity::builder::CreateActionRow;
use std::io::Cursor;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::task::spawn_blocking;
use tokio::time::sleep;

// ^^^ I WILL CLEAN THIS LATER ^^^

const DICE_IMAGES_PATHS: [&str; 6] = [
    "assets/images/dice/d6_red_1.png",
    "assets/images/dice/d6_red_2.png",
    "assets/images/dice/d6_red_3.png",
    "assets/images/dice/d6_red_4.png",
    "assets/images/dice/d6_red_5.png",
    "assets/images/dice/d6_red_6.png",
];

const DIE_AMOUNT: u32 = 3;
const DICE_MARGIN: u32 = 32;
const DICE_SIZE: u32 = 128;
const TOTAL_HEIGHT: u32 = 256;
const TOTAL_WIDTH: u32 = DIE_AMOUNT * DICE_SIZE + (DIE_AMOUNT + 1) * DICE_MARGIN;

static DICE_IMAGES: LazyLock<Vec<RgbImage>> = LazyLock::new(|| {
    DICE_IMAGES_PATHS
        .iter()
        .map(|p| {
            image::open(p)
                .unwrap()
                .resize(u32::MAX, DICE_SIZE, FilterType::Lanczos3)
                .to_rgb8()
        })
        .collect()
});

static BACKGROUND_IMAGE: LazyLock<RgbImage> = LazyLock::new(|| {
    let bg = image::open("assets/images/dice/background.jpg")
        .unwrap()
        .to_rgb8();

    let (bg_w, bg_h) = bg.dimensions();

    assert!(bg_w >= TOTAL_WIDTH);
    assert!(bg_h >= TOTAL_HEIGHT);

    let x = (bg_w - TOTAL_WIDTH) / 2;
    let y = (bg_h - TOTAL_HEIGHT) / 2;

    imageops::crop_imm(&bg, x, y, TOTAL_WIDTH, TOTAL_HEIGHT).to_image()
});

#[command(
    slash_command,
    prefix_command,
    description_localized("en-US", "Dice game (Medieval style)"),
    description_localized("pl", "Kości (Styl średniowieczny)"),
    aliases("kosci", "d", "k")
)]
pub async fn dice(
    ctx: Context<'_>,

    #[description = "Bet amount"]
    #[description_localized("pl", "Rozmiar zakładu")]
    bet: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let db = &ctx.data().db.pool;

    let user_id = ctx.author().id.get() as i64;
    let user_data = ctx.data().db.ensure_member(user_id).await?;

    let bet = if bet.to_lowercase() == "all" {
        user_data.user.cash
    } else {
        let result = bet.parse::<i64>();

        match result {
            Ok(bet) => {
                if bet <= 0 {
                    send_err_msg(&ctx, "Bet must be positive!").await?;

                    return Ok(());
                }

                if bet > user_data.user.cash {
                    send_err_msg(&ctx, "You don't have enough money!").await?;

                    return Ok(());
                }

                bet
            },
            Err(_) => {
                send_err_msg(&ctx, "Invalid bet amount!").await?;

                return Ok(());
            }
        }
    };

    send_init_msg(&ctx).await?;

    while let Some(mut interaction) = ComponentInteractionCollector::new(&ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        interaction.defer(&ctx).await?;

        if interaction.data.custom_id == "throw" {
            let dice = throw_dice();

            let (dice, img) = spawn_blocking(move || {
                let img = get_dice_image(&dice)?;
                Ok::<_, Error>((dice, img))
            })
            .await??;

            let attachment = CreateAttachment::bytes(img, "dice.jpg");

            let dice = dice.iter().map(|x| x + 1).collect::<Vec<u8>>();
            let user_sum: u8 = dice.iter().sum();

            let mut color = 0x00FF00;
            let desc = format!("You rolled: {:?}! Sum: {user_sum}", dice);
            edit_msg(&ctx, &mut interaction, &desc, attachment, color).await?;

            sleep(Duration::from_secs(3)).await;

            let dice = throw_dice();

            let (dice, img) = spawn_blocking(move || {
                let img = get_dice_image(&dice)?;
                Ok::<_, Error>((dice, img))
            })
            .await??;

            let attachment = CreateAttachment::bytes(img, "dice.jpg");

            let dice = dice.iter().map(|x| x + 1).collect::<Vec<u8>>();
            let enemy_sum: u8 = dice.iter().sum();

            let desc = if user_sum > enemy_sum {
                format!("{desc}\n\nEnemy rolled: {dice:?}! Sum: {enemy_sum}\n\n**You won!** 🎉")
            } else if user_sum == enemy_sum {
                color = 0x0000FF;
                format!("{desc}\n\nEnemy rolled: {dice:?}! Sum: {enemy_sum}\n\n**It's a tie!** ⚖️")
            } else {
                color = 0xFF0000;
                format!("{desc}\n\nEnemy rolled: {dice:?}! Sum: {enemy_sum}\n\n**You lost...** 🥀")
            };

            edit_msg(&ctx, &mut interaction, &desc, attachment, color).await?;
        }
    }

    Ok(())
}

fn throw_dice() -> Vec<u8> {
    let mut rng = rand::rng();

    let mut dice = Vec::new();

    for _ in 0..DIE_AMOUNT {
        dice.push(rng.random_range(0..6));
    }

    dice
}

fn get_dice_image(dice: &Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut img = BACKGROUND_IMAGE.clone();

    for (i, die) in dice.iter().enumerate() {
        let x = DICE_MARGIN + i as u32 * (DICE_SIZE + DICE_MARGIN);
        let y = (TOTAL_HEIGHT / 2) - (DICE_SIZE / 2);

        img.copy_from(&DICE_IMAGES[*die as usize], x, y)?;
    }

    let mut bytes = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut bytes, 85);

    encoder.write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgb8,
    )?;

    Ok(bytes)
}

async fn send_init_msg(ctx: &Context<'_>) -> Result<(), Error> {
    let img = spawn_blocking(move || get_dice_image(&vec![])).await??;
    let attachment = CreateAttachment::bytes(img, "dice.jpg");

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("🎲 Dice!")
                    .description("Throw the dice to win! (or not?)")
                    .color(0x00FF00)
                    .image("attachment://dice.jpg"),
            )
            .attachment(attachment)
            .reply(true)
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new("throw")
                    .label("Throw")
                    .style(ButtonStyle::Primary),
            ])]),
    )
    .await?;

    Ok(())
}

async fn edit_msg(
    ctx: &Context<'_>,
    interaction: &mut ComponentInteraction,
    desc: &str,
    attachment: CreateAttachment,
    color: u32,
) -> Result<(), Error> {
    interaction
        .message
        .edit(
            ctx,
            serenity::builder::EditMessage::new()
                .embed(
                    CreateEmbed::new()
                        .title("🎲 Dice!")
                        .description(format!("Throw the dice to win! (or not?)\n{desc}"))
                        .color(color)
                        .image("attachment://dice.jpg"),
                )
                .new_attachment(attachment)
                .components(vec![]),
        )
        .await?;

    Ok(())
}

async fn send_err_msg(ctx: &Context<'_>, msg: &str) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("🚫 Error!")
                    .description(msg)
                    .color(0xFF0000)
            )
            .reply(true)
    )
        .await?;

    Ok(())
}
