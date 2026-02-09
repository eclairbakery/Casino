use crate::bot::{Context, Error};
use crate::services::shop::registry::get_shop_registry;
use poise::CreateReply;
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command, 
    prefix_command, 
    description_localized("pl", "Czas wydać wypłatę z kasyna!")
)]
pub async fn shop(ctx: Context<'_>) -> Result<(), Error> {
    let items = get_shop_registry();
    let mut embed = serenity::CreateEmbed::new()
        .title("🛒 Żabka")
        .description("Drogo, ale można coś wydać przynajmiej... Używasz `buy` i potem item, by coś kupić.")
        .color(0x00FFFF);

    for item in items {
        embed = embed.field(
            format!("{}. {}", item.id, item.name),
            format!("_{}_\nZa jedyne: **{} dolarów**", item.description, item.price),
            false
        );
    }

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}