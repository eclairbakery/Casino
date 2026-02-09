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
            .field("Komendy", "Generalnie na chwilę obecną możesz używać prawie każdej komendy oprócz hazardu jak w każdym innym bocie ekonomicznym tj. `bal`, `withdraw`, `work`, `slut`, `crime`, `deposit`, `ping`, `pay`, `rob`, `topmoney`, `shop`, `buy`. Są też popularne aliasy, np. `deposit` -> `dep`. To dalej alpha, więc trochę niedopracowane, ale lepsze to niż nic.", false)
            .field("Hazard", "- **automaty**: Generalnie używasz `slots` i możesz po tym podać kwotę jaką chcesz obstawić na automatach. Daje to bardzo duże zyski, ale jest mała szansa na wygraną...\n- **rzut monetą:** To jest useful w pierwszych fazach gry, ponieważ maksymalnie możesz zyskać 100 dolarów. Still warto używać jednak w początkowej fazie. Używasz tego generalnie tak, że `coinflip` i potem albo h albo t, a następnie no to ile stawiasz.\n- **blackjack**: Absolutny klasyk gatunku. Używasz `blackjack` i potem dajesz liczbę. Wtedy zyskasz super hajs, jak umiesz w to grać.\n- **dice**: Co tu dużo mówić... Losujemy Ci liczbę od 1 do 100 no i masz ten... jak zdobędziesz więcej niż 55 to wygrywasz. Używasz `dice` i potem dajesz zakład.", false)
            .field("Pomoc w tworzeniu", "Jeżeli chcesz pomóc w tworzeniu tego bota, no to możesz [zrobić pull request](<https://github.com/eclairbakery/Casino/pulls>) z jakąś funkcją, poprawką, czy czymkolwiek. Jakby coś, to tylko ekonomia. Główny bot, od prawie wszystkiego innego, jest [tutaj](<https://github.com/eclairbakery/EclairBOT>).", false)
        )
    ).await?;

    Ok(())
}
