use crate::bot::format_number::format_number;
use crate::bot::{Context, Error};
use crate::services::shop::registry::get_shop_registry;
use poise::CreateReply;
use serenity::all::CreateEmbed;

#[poise::command(
    slash_command,
    prefix_command,
    description_localized("pl", "Czas wydać wypłatę z kasyna!")
)]
pub async fn shop(ctx: Context<'_>) -> Result<(), Error> {
    let items = get_shop_registry();
    let mut embed = CreateEmbed::new()
        .title("🛒 Żabka")
        .description(
            "Drogo, ale można coś wydać przynajmiej... Używasz `buy` i potem item, by coś kupić.",
        )
        .color(0x00FFFF);

    for item in items {
        embed = embed.field(
            format!("{}. {}", item.id, item.name),
            format!(
                "_{}_\nZa jedyne: **{}zł**",
                item.description,
                format_number(item.price)
            ),
            false,
        );
    }

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}
