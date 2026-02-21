use std::sync::LazyLock;
use anyhow::Error;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use poise::command;
use rand::RngExt;
use crate::bot::Context;

const IMAGE_PATH: [&str; 6] = [
	"assets/images/dice/d6_red_1.png",
	"assets/images/dice/d6_red_2.png",
	"assets/images/dice/d6_red_3.png",
	"assets/images/dice/d6_red_4.png",
	"assets/images/dice/d6_red_5.png",
	"assets/images/dice/d6_red_6.png",
];

static IMAGES: LazyLock<Vec<DynamicImage>> = LazyLock::new(|| {
	let mut images = Vec::new();

	for path in IMAGE_PATH {
		images.push(image::open(path).unwrap());
	}

	images
});

static DICE_DIMENSIONS: LazyLock<(u32, u32)> = LazyLock::new(|| {
	IMAGES[0].dimensions()
});

#[command(
	slash_command,
	prefix_command,
	description_localized("en-US", "Dice"),
	description_localized("pl", "Kości"),
)]
pub async fn dice(ctx: Context<'_>) -> Result<(), Error> {
	let (dice_width, dice_height) = *DICE_DIMENSIONS;

	let total_dice_width = dice_width * 5;

	let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
		ImageBuffer::new(total_dice_width, dice_height);

	let dice = {
		let mut dice = Vec::new();

		let mut rng = rand::rng();

		for _ in 0..5 {
			dice.push(rng.random_range(0..6));
		}

		dice
	};

	for (i, die) in dice.iter().enumerate() {
		img.copy_from(&IMAGES[*die as usize], dice_width * i as u32, 0)?;
	}

	img.save("/tmp/combined.png")?;

	Ok(())
}
