use crate::bot::format_number::format_number;
use crate::bot::{Context, Error};
use crate::services::shop::registry::get_shop_registry;
use anyhow::anyhow;
use poise::CreateReply;
use serenity::all::{CreateEmbed, RoleId};

#[poise::command(
    slash_command,
    prefix_command,
    description_localized("pl", "No kupić rolę tu możesz.")
)]
pub async fn buy(
    ctx: Context<'_>,
    #[description_localized(
        "pl",
        "Podaj ID ze sklepu roli którą chcesz nabyć (od chińskich inwestorów)."
    )]
    item_id: i32,
) -> Result<(), Error> {
    let registry = get_shop_registry();
    let item = registry.iter().find(|i| i.id == item_id);

    let item = match item {
        Some(i) => i,
        None => {
            ctx.send(
                CreateReply::default().embed(
                    CreateEmbed::new()
                        .title("❌ Błąd")
                        .description("Przedmiot o tym ID nie istnieje.")
                        .color(0xFF0000),
                ),
            )
            .await?;
            return Ok(());
        }
    };

    let author = ctx.author();
    let user_raw_id = author.id.get() as i64;
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("Command must be used in guild"));
    let db = &ctx.data().db;

    let user_data = db.ensure_member(user_raw_id).await?;

    if db.process_purchase(user_raw_id, item.price).await? {
        if let Some(role_id) = item.role_id {
            let role = RoleId::new(role_id);

            let member = guild_id?.member(&ctx, author.id).await?;

            if member.add_role(&ctx, role).await.is_err() {
                user_data.user.change_cash(&db.pool, -item.price).await?;
                ctx.send(
                    CreateReply::default().embed(
                        CreateEmbed::new()
                            .title("❌ Błąd")
                            .description("Ktoś coś namieszał i nie mogłem dodać roli 🥀")
                            .color(0xFF0000),
                    ),
                )
                .await?;
                return Ok(());
            }
        }

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("✅ Zakup udany!")
                    .description(format!(
                        "Kupiłeś **{}** za **{}**zł!",
                        item.name,
                        format_number(item.price)
                    ))
                    .color(0x00FF00),
            ),
        )
        .await?;
    } else {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Jesteś biedny")
                    .description("Nie masz wystarczającej ilości gotówki w portfelu!")
                    .color(0xFF0000),
            ),
        )
        .await?;
    }

    Ok(())
}
