use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

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
    user: Option<serenity::User>,
) -> Result<(), Error> {
    let target = user.as_ref().unwrap_or(ctx.author());
    let user_id = target.id.get() as i64;
    let db = &ctx.data().db;

    let (member, _) = db.ensure_member(user_id).await?;

    let total = member.cash + member.bank;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title(format!("Pieniądze materialisty {}", target.name))
                .field("Gotówka", format!("`{}` 💵", member.cash), true)
                .field("Bank", format!("`{}` 💳", member.bank), true)
                .field("Suma", format!("**`{}`** 💰", total), false)
                .color(0x00AEFF)
                .thumbnail(target.face()),
        ),
    )
    .await?;

    Ok(())
}
