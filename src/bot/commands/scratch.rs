use std::time::Duration;

use crate::bot::{Context, Error};
use poise::{CreateReply, command};
use serenity::all::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateAttachment, CreateButton,
    CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
};

#[command(
    slash_command,
    prefix_command,
    description_localized("pl", "Zdrap zdrapke!")
)]
pub async fn scratch(ctx: Context<'_>) -> Result<(), Error> {
    let scratch_card = CreateAttachment::path("assets/images/scratch_card.png").await?;
    let scratch_card_scratched =
        CreateAttachment::path("assets/images/scratch_card_scratched.png").await?;
    let scratch_card_name = scratch_card.filename.clone();
    let scratch_card_scratched_name = scratch_card_scratched.filename.clone();

    let scratch_card_msg = ctx
        .send(
            CreateReply::default()
                .attachment(scratch_card)
                .embed(
                    CreateEmbed::new()
                        .title("Zdrap zdrapke! 🎟️️")
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
        press
            .create_response(
                ctx.serenity_context(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .add_file(scratch_card_scratched)
                        .embed(
                            CreateEmbed::new()
                                .title("Zdrap zdrapke! 🎟️️")
                                .color(0x00FF00)
                                .image(format!("attachment://{}", scratch_card_scratched_name)),
                        )
                        .components(Vec::new()),
                ),
            )
            .await?;
    }

    Ok(())
}
