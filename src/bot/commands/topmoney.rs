use crate::bot::{Context, Error};
use poise::CreateReply;
use serenity::all::{Timestamp, CreateEmbed, CreateEmbedFooter};

#[poise::command(
    slash_command,
    prefix_command,
    aliases("leaderboard", "topka", "top", "topeco"),
    description_localized(
        "pl",
        "Tutaj możesz zobaczyć jak nisko jesteś na drabinie społecznej Miasta Stołecznego Warszawa"
    )
)]
pub async fn topmoney(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;

    let top_members = db.get_top_members(12).await?;

    if top_members.is_empty() {
        ctx.say("tu był taki edge case co się raczej nie zdarzy więc nie robie embeda tym zjebanym sposobem 💔").await?;
        return Ok(());
    }

    let mut leaderboard_text = String::new();

    for (index, member) in top_members.iter().enumerate() {
        let total = member.cash + member.bank;
        leaderboard_text.push_str(&format!(
            "{}. <@{}> - **`{}`** 💰\n",
            index + 1,
            member.id,
            total
        ));
    }

    ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
            .title("🏆 Janusze kasyna. Może też janusze biznesu.")
            .description(leaderboard_text)
            .color(0xFFD700)
            .footer(CreateEmbedFooter::new("Chcesz tu być? To masz problem, bo to nie jest miejsce dla ciebie. Nigdy nim nie miało być. No chyba, że trochę pookradasz ludzi... znaczy zarobisz, to sie zastanowię."))
            .timestamp(Timestamp::now())
        )
    ).await?;

    Ok(())
}
