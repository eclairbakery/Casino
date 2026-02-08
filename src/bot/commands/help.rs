use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command, 
    prefix_command, 
    description_localized("pl", "No pomogę Ci czy coś"),
)]
pub async fn help(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.send(CreateReply::default()
        .embed(serenity::CreateEmbed::new()
            .title("Witaj w ekonomii!")
            .field("Komendy", "Generalnie na chwilę obecną możesz używać prawie każdej komendy oprócz hazardu jak w każdym innym bocie ekonomicznym tj. `bal`, `withdraw`, `work`, `slut`, `crime`, `deposit`, `ping`, `pay`, `rob`, `topmoney`. Są też popularne aliasy, np. `deposit` -> `dep`. To dalej alpha, więc np. hazardu nie ma, ale chyba będzie (raczej).", false)
            .field("Hazard", "Na razie go nie ma :wilted_rose:", false)
            .field("Pomoc w tworzeniu", "Jeżeli chcesz pomóc w tworzeniu tego bota, no to możesz [zrobić pull request](<https://github.com/eclairbakery/Casino/pulls>) z jakąś funkcją, poprawką, czy czymkolwiek. Jakby coś, to tylko ekonomia. Główny bot, od prawie wszystkiego innego, jest [tutaj](<https://github.com/eclairbakery/EclairBOT>).", false)
        )
    ).await?;

    Ok(())
}
