use poise::CreateReply;
use serenity::all::{CreateEmbed, User};

use crate::bot::{Context, Error, format_number::format_number};

#[poise::command(
    slash_command,
    prefix_command,
    description_localized(
        "pl",
        "Zobacz ile już zaje*bałeś losowym ludziom kasy... znaczy... ile w pełni uczciwie zarobiłeś!"
    ),
    aliases("bal")
)]
pub async fn balance(
    ctx: Context<'_>,
    #[description_localized(
        "pl",
        "Użytkownik taki fajny, którego uczciwość chcesz sprawdzić w Krajowym Systemie Długów"
    )]
    user: Option<User>,
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or(ctx.author());
    let user_id = user.id.get() as i64;
    let db = &ctx.data().db;

    let user_data = db.ensure_member(user_id).await?;

    let total = user_data.user.cash + user_data.user.bank;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title(format!("Pieniądze materialisty {}", user.name))
                .field("Gotówka", format!("`{}` 💵", format_number(user_data.user.cash)), true)
                .field("Bank", format!("`{}` 💳", format_number(user_data.user.bank)), true)
                .field("Suma", format!("**`{}`** 💰", format_number(total)), false)
                .color(0x00AEFF)
                .thumbnail(user.face()),
        ),
    )
    .await?;

    Ok(())
}
