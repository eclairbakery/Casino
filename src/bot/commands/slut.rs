use crate::bot::Context;
use anyhow::Error;
use poise::CreateReply;
use rand::RngExt;
use rand::prelude::IndexedRandom;
use serenity::all::CreateEmbed;
use std::time;
use std::time::SystemTime;

const RESPONSES: [&str; 4] = [
    "Wygrałeś w \"przyjacielskiego\" pokera i przegrany wyskoczył z {amount} dolarów, które ty otrzymałeś!",
    "Nudziło ci się i wraz z Natalią shackowałeś Pentagon, leakując rządowe dokumenty do dark-webowej grupy hakerów, która zapłaciła za nie okrągłe {amount} dolarów.",
    "Gratulacje! Właśnie wygrałeś kartę podarunkową Biedronki O OSZAŁAMIAJĄCEJ WARTOŚCI 200 DOLARÓW! Czy ty nie widzisz, ile za to kupisz? Czy ty nie widzisz tej mocy prezentów dla rodziny? Tyle możesz zrobić z taką kartą podarunkową! Wystarczy, że ostatnio otrzymany hajs w wysokości {amount}, jakim jest twoja nadwyżka podatkowa z lat 2001-2004 przelejesz na konto naszego CEO, któremu recently zmarła babcia.",
    "Kasyno w okolicy dało Ci {amount} dolarów za wystawienie pewnej rzeczy in public. Ci którzy wiedzą, wiedzą...",
];

const FAIL_RESPONSES: [&str; 3] = [
    "Próbowałeś grać w pokera w ogrodzie, ale wjechałeś w krasnala ogrodowego. Właściciel się wkurzył i kazał zapłacić {amount} dolarów kary!",
    "Sanepid zamknął Twoje stoisko z lemoniadą. Chamski, prawda? Zabija młodych przedsiębiorców. Jeszcze grzywnę nałożył. Aż {amount} dolarów. O ja piernicze...",
    "Nie mając prawa jazdy wjechałeś w pieszego. Nic się mu nie stało, ale za uszkodzenie ciała, uszczerbek na zdrowiu i jazdę bez biletu... znaczy prawa jazdy musiałeś zapłacić {amount} dolarów.",
];

#[poise::command(
    slash_command,
    prefix_command,
    description_localized(
        "pl",
        "Pr*ca, średnio legalna, dorywcza, ale możliwe, że fajna (spoiler: nie)."
    )
)]
pub async fn slut(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let user_data = db.ensure_member(user_id).await?;

    if (user_data.user.cash + user_data.user.bank) < 100.00 {
        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title("⏳ Jeszcze nie odblokowałeś slut i crime")
                .description("Wróć jak zdobędziesz co najmniej 100$ łącznie w portfelu i w banku. Po prostu łatwo jest tu przewalić hajs do minusowego poziomu, więc to taka blokada bezpieczeństwa.")
                .color(0xFF0000))
        ).await?;

        return Ok(());
    }

    let now = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_secs() as i64;

    let cooldown = 300;
    let time_passed = now - user_data.timeouts.last_slut;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;

        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title("⏳ Ahhhhhh ta twoja niecierpliwość!")
                .description(format!("Myślałeś, że nie masz ADHD? Że umiesz wysiedzieć w jednym miejscu bez wiercenia się i bez pracy? Nie, nie umiesz. :wilted_rose:\n\nDaję ci challenge - wróć za: **{} sekund**, a nie za 5 femtosekund.", remaining))
                .color(0xFF0000))
        ).await?;

        return Ok(());
    }

    let (chance, amount, desc) = {
        let mut rng = rand::rng();

        let chance = rng.random_range(0..100);
        let mut amount = rng.random_range(50.00..=350.00);

        let desc_templ = if chance < 60 {
            RESPONSES
                .choose(&mut rng)
                .unwrap_or(&"message się zepsuł :wilted_rose: ale zarobiłeś {amount}")
        } else {
            amount /= 2.0;

            FAIL_RESPONSES
                .choose(&mut rng)
                .unwrap_or(&"Coś poszło nie tak... straciłeś {amount}")
        };

        let desc = desc_templ.replace("{amount}", &amount.to_string());

        (chance, amount, desc)
    };

    if chance < 60 {
        db.add_cash(user_id, amount).await?;
        db.update_timeout(user_id, "last_slut", now).await?;

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("⚒️ Praca dorywcza czasem przynosi efekty...")
                    .description(desc)
                    .color(0x00FF00),
            ),
        )
        .await?;
    } else {
        db.add_cash(user_id, -amount).await?;
        db.update_timeout(user_id, "last_work", now).await?;

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Za dużo byś chciał. Nie tym razem.")
                    .description(desc)
                    .color(0xFF0000),
            ),
        )
        .await?;
    }

    Ok(())
}
