use std::io::Cursor;
use std::sync::LazyLock;
use anyhow::Error;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageEncoder, ImageFormat, Rgb, RgbImage, Rgba};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use poise::{command, CreateReply};
use serenity::all::{CreateAttachment, CreateEmbed};
use tokio::task::spawn_blocking;
use crate::bot::Context;

const IMAGE_PATHS: [&str; 6] = [
	"assets/images/dice/d6_red_1.png",
	"assets/images/dice/d6_red_2.png",
	"assets/images/dice/d6_red_3.png",
	"assets/images/dice/d6_red_4.png",
	"assets/images/dice/d6_red_5.png",
	"assets/images/dice/d6_red_6.png",
];

const DICE_MARGIN:u32 = 16;
const TARGET_HEIGHT: u32 = 256;

static IMAGES: LazyLock<Vec<RgbImage>> = LazyLock::new(|| {
	IMAGE_PATHS
		.iter()
		.map(|p| {
			let img = image::open(p).unwrap().to_rgb8();
			let resized = image::imageops::resize(
				&img,
				img.width() * TARGET_HEIGHT / img.height(),
				TARGET_HEIGHT,
				image::imageops::Lanczos3,
			);
			resized
		})
		.collect()
});

static DICE_DIMENSIONS: LazyLock<(u32, u32)> = LazyLock::new(|| {
	IMAGES[0].dimensions()
});

static DICE_WIDTH: LazyLock<u32> = LazyLock::new(|| {
	let (dice_width	, _) = *DICE_DIMENSIONS;

	dice_width
});

static DICE_HEIGHT: LazyLock<u32> = LazyLock::new(|| {
	let (_, dice_height) = *DICE_DIMENSIONS;

	dice_height
});

#[command(
	slash_command,
	prefix_command,
	description_localized("en-US", "Dice"),
	description_localized("pl", "Kości"),
)]
pub async fn dice(ctx: Context<'_>) -> Result<(), Error> {
	ctx.defer().await?;

	let img = spawn_blocking(move || {
		get_dice_image(vec![0, 1, 2, 3, 4])
	}).await??;

	let attachment = CreateAttachment::bytes(img, "dice.jpg");

	ctx.send(
		CreateReply::default().embed(
			CreateEmbed::new()
				.title("🎲 Dice!")
				.color(0x00FF00)
				.image("attachment://dice.jpg")
		).attachment(attachment),
	)
		.await?;

	Ok(())
}

fn get_dice_image(dice: Vec<u8>) -> Result<Vec<u8>, Error> {
	let total_dice_width = *DICE_WIDTH * 5;

	let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
		ImageBuffer::new(total_dice_width, *DICE_HEIGHT);

	for (i, die) in dice.iter().enumerate() {
		img.copy_from(&IMAGES[*die as usize], *DICE_WIDTH * i as u32, 0)?;
	}

	let mut bytes = Vec::with_capacity(
		(img.width() * img.height() * 3) as usize
	);

	let encoder = JpegEncoder::new_with_quality(&mut bytes, 85);

	encoder.write_image(
		img.as_raw(),
		img.width(),
		img.height(),
		image::ExtendedColorType::Rgb8,
	)?;

	Ok(bytes)
}
