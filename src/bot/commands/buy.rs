use crate::bot::{Context, Error};
use crate::bot::items::get_shop_registry;
use poise::CreateReply;
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, prefix_command, name_localized("pl", "buy"))]
pub async fn buy(ctx: Context<'_>, item_id: i32) -> Result<(), Error> {
    let registry = get_shop_registry();
    let item = registry.iter().find(|i| i.id == item_id);

    let item = match item {
        Some(i) => i,
        None => {
            ctx.send(CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("❌ Błąd")
                    .description("Czy ja mam ci tłumaczyć jak się używa tej komendy? No musisz liczbę przy przedmiocie mi podać.")
                    .color(0xFF0000)
            )).await?;
            return Ok(());
        }
    };

    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    if db.process_purchase(user_id, item.price).await? {
        if let Some(role_id) = item.role_id {
            let role = serenity::RoleId::new(role_id);
            if let Err(_) = ctx.author().add_role(&ctx, role).await {
                db.add_cash(user_id, item.price).await?;
                ctx.send(CreateReply::default().embed(
                    serenity::CreateEmbed::new()
                        .title("❌ Błąd")
                        .description("Ktoś coś namieszał i nie mogłem dodać roli 🥀")
                        .color(0xFF0000)
                )).await?;
                return Ok(());
            }
        }

        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("✅ Zakup udany!")
                .description(format!("Kupiłeś **{}** za **{}** dolarów!", item.name, item.price))
                .color(0x00FF00)
        )).await?;
    } else {
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ Jesteś biedny")
                .description("Nie masz wystarczającej ilości gotówki w portfelu!")
                .color(0xFF0000)
        )).await?;
    }

    Ok(())
}