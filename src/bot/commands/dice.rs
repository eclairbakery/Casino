use anyhow::Error;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{GenericImage, ImageEncoder, RgbImage, imageops};
use poise::{CreateReply, ReplyHandle, command};
use rand::RngExt;
use serenity::all::{
    ButtonStyle, Color, ComponentInteraction, ComponentInteractionCollector, CreateActionRow,
    CreateAttachment, CreateButton, CreateEmbed, EditInteractionResponse,
};
use std::array;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::task::spawn_blocking;
use tokio::time::sleep;

use crate::bot::Context;
use crate::bot::utils::msg;

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

    let x = (bg_w - TOTAL_WIDTH) / 2;
    let y = (bg_h - TOTAL_HEIGHT) / 2;

    imageops::crop_imm(&bg, x, y, TOTAL_WIDTH, TOTAL_HEIGHT).to_image()
});

#[command(
    slash_command,
    prefix_command,
    description_localized("pl", "Kości (Styl średniowieczny)"),
    aliases("kosci", "d", "k")
)]
pub async fn dice(
    ctx: Context<'_>,

    #[description_localized("pl", "Ile kasy stawiasz")] bet: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let db = &ctx.data().db.pool;

    let user_id = ctx.author().id.get() as i64;
    let user_data = ctx.data().db.ensure_member(user_id).await?;

    let bet = if bet.to_lowercase() == "all" {
        if user_data.user.wallet <= 0 {
            msg::reply_err(
                &ctx,
                "Brak pieniędzy",
                "Dosłownie masz pieniądze na minusie a chcesz grać hazard? Nic tylko pogratulować",
            )
            .await?;

            return Ok(());
        }
        user_data.user.wallet
    } else {
        let result = parse_money_to_cents(&bet);

        match result {
            Ok(bet) => {
                if bet <= 0 {
                    msg::reply_err(
                        &ctx,
                        "Niepoprawny zakład",
                        "Zakład musi być większy od zero jakbyś nie wiedział.",
                    )
                    .await?;
                    return Ok(());
                }

                if bet > user_data.user.wallet {
                    msg::reply_err(
                        &ctx,
                        "Nie wystarczająco pieniędzy",
                        "Nie masz tyle pieniędzy gałganie",
                    )
                    .await?;
                    return Ok(());
                }

                bet
            }
            Err(_) => {
                msg::reply_err(&ctx, "Niepoprawny zakład", "Źle wpisałeś liczbe czy coś.").await?;

                return Ok(());
            }
        }
    };

    let Some(payout) = bet.checked_mul(2) else {
        msg::reply_err(&ctx, "Za duży zakład", "Twój zakład powoduje integer overflow a przypomne że uzywamy i64, polecam wyjść na dwór zamiast grać w ekonomie cały dzień").await?;
        return Ok(());
    };

    let reply = send_init_msg(&ctx).await?;
    let msg = reply.message().await?;

    let collector = ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .message_id(msg.id)
        .timeout(Duration::from_secs(10));

    if let Some(mut interaction) = collector.await {
        interaction.defer(&ctx).await?;

        if interaction.data.custom_id == "throw" {
            user_data.user.change_wallet(db, -bet).await?;

            let dice = throw_dice();

            let (dice, img) = spawn_blocking(move || {
                let img = get_dice_image(&dice)?;
                Ok::<_, Error>((dice, img))
            })
            .await??;

            let attachment = CreateAttachment::bytes(img, "dice.jpg");

            let user_sum: u8 = dice.iter().map(|x| x + 1).sum();

            let mut color = Color::DARK_GREEN;
            let desc = format!("Wyrzuciłeś: {:?}! Łącznie: {user_sum}", dice);
            edit_msg(&ctx, &mut interaction, &desc, attachment, color).await?;

            sleep(Duration::from_secs(2)).await;

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
                user_data.user.change_wallet(db, payout).await?;

                format!(
                    "{desc}\nPrzeciwnik wyrzucił: {dice:?}! Łącznie: {enemy_sum}\n\n**Wygrałeś! {}!** 🎉",
                    format_minor(payout)
                )
            } else if user_sum == enemy_sum {
                user_data.user.change_wallet(db, bet).await?;

                color = Color::BLUE;
                format!(
                    "{desc}\nPrzeciwnik wyrzucił: {dice:?}! Łącznie: {enemy_sum}\n\n**Remis!** ⚖️"
                )
            } else {
                color = Color::RED;
                format!(
                    "{desc}\nPrzeciwnik wyrzucił: {dice:?}! Łącznie: {enemy_sum}\n\n**Przegrałeś...** 🥀"
                )
            };

            edit_msg(&ctx, &mut interaction, &desc, attachment, color).await?;
        }
    }

    Ok(())
}

fn throw_dice() -> [u8; DIE_AMOUNT as usize] {
    let mut rng = rand::rng();

    array::from_fn(|_| rng.random_range(0..6))
}

fn get_dice_image(dice: &[u8]) -> Result<Vec<u8>, Error> {
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

async fn send_init_msg<'a>(ctx: &'a Context<'a>) -> Result<ReplyHandle<'a>, Error> {
    let img = spawn_blocking(move || get_dice_image(&[])).await??;
    let attachment = CreateAttachment::bytes(img, "dice.jpg");

    let reply = ctx
        .send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("🎲 Kosteczka")
                        .description("Rzuć kością by wygrać (lub nie?)")
                        .color(0x00FF00)
                        .image("attachment://dice.jpg"),
                )
                .attachment(attachment)
                .reply(true)
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new("throw")
                        .label("Rzucaj 🔥")
                        .style(ButtonStyle::Primary),
                ])]),
        )
        .await?;

    Ok(reply)
}

async fn edit_msg(
    ctx: &Context<'_>,
    interaction: &mut ComponentInteraction,
    desc: &str,
    attachment: CreateAttachment,
    color: Color,
) -> Result<(), Error> {
    interaction
        .edit_response(
            ctx,
            EditInteractionResponse::new()
                .embed(
                    CreateEmbed::new()
                        .title("🎲 Dice!")
                        .description(format!("Throw the dice to win! (or not?)\n\n{desc}"))
                        .color(color)
                        .image("attachment://dice.jpg"),
                )
                .new_attachment(attachment)
                .components(vec![]),
        )
        .await?;

    Ok(())
}

fn parse_money_to_cents(input: &str) -> Result<i64, ()> {
    let mut parts = input.trim().split('.');

    let major: i64 = parts.next().ok_or(())?.parse().or(Err(()))?;
    let frac = parts.next().unwrap_or("");

    if parts.next().is_some() || frac.len() > 2 {
        return Err(());
    }

    let minor = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>().or(Err(()))? * 10,
        2 => frac.parse::<i64>().or(Err(()))?,
        _ => unreachable!(),
    };

    let cents = if major >= 0 { minor } else { -minor };

    major
        .checked_mul(100)
        .and_then(|v| v.checked_add(cents))
        .ok_or(())
}

fn format_minor(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };

    let abs = value.abs();
    let major = abs / 100;
    let minor = abs % 100;

    format!("{sign}{major}.{minor:02}zł")
}
