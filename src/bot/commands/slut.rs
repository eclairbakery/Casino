use crate::bot::{Context, Error};
use poise::CreateReply;
use rand::Rng;
use rand::prelude::IndexedRandom;

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

    let (member, timeouts) = db.ensure_member(user_id).await?;
    if (member.cash + member.bank) < 100 {
        ctx.send(CreateReply::default()
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title("⏳ Jeszcze nie odblokowałeś slut i crime")
                .description("Wróć jak zdobędziesz co najmniej 100$ łącznie w portfelu i w banku. Po prostu łatwo jest tu przewalić hajs do minusowego poziomu, więc to taka blokada bezpieczeństwa.")
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let cooldown = 300;
    let time_passed = now - timeouts.last_slut;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;
        ctx.send(CreateReply::default()
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title("⏳ Ahhhhhh ta twoja niecierpliwość!")
                .description(format!("Myślałeś, że nie masz ADHD? Że umiesz wysiedzieć w jednym miejscu bez wiercenia się i bez pracy? Nie, nie umiesz. :wilted_rose:\n\nDaję ci challenge - wróć za: **{} sekund**, a nie za 5 femtosekund.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    let chance = rand::rng().random_range(0..100);
    let how_much = rand::rng().random_range(300..=1500);

    if chance < 60 {
        let responses = vec![
            "Wygrałeś w \"przyjacielskiego\" pokera i przegrany wyskoczył z {amount} dolarów, które ty otrzymałeś!",
            "Nudziło ci się i wraz z Natalią shackowałeś Pentagon, leakując rządowe dokumenty do dark-webowej grupy hakerów, która zapłaciła za nie okrągłe {amount} dolarów.",
            "Gratulacje! Właśnie wygrałeś kartę podarunkową Biedronki O OSZAŁAMIAJĄCEJ WARTOŚCI 200 DOLARÓW! Czy ty nie widzisz, ile za to kupisz? Czy ty nie widzisz tej mocy prezentów dla rodziny? Tyle możesz zrobić z taką kartą podarunkową! Wystarczy, że ostatnio otrzymany hajs w wysokości {amount}, jakim jest twoja nadwyżka podatkowa z lat 2001-2004 przelejesz na konto naszego CEO, któremu recently zmarła babcia.",
            "Kasyno w okolicy dało Ci {amount} dolarów za wystawienie pewnej rzeczy in public. Ci którzy wiedzą, wiedzą...",
        ];
        let desc_templ = responses
            .choose(&mut rand::rng())
            .unwrap_or(&"message się zepsuł :wilted_rose: ale zarobiłeś {amount}");
        let desc = desc_templ.replace("{amount}", &how_much.to_string());

        db.add_cash(user_id, how_much).await?;
        db.update_timeout(user_id, "last_slut", now).await?;

        ctx.send(
            CreateReply::default().embed(
                poise::serenity_prelude::CreateEmbed::new()
                    .title("⚒️ Praca dorywcza czasem przynosi efekty...")
                    .description(desc)
                    .color(0x00FF00),
            ),
        )
        .await?;

        Ok(())
    } else {
        let loss = how_much / 2;
        let fail_responses = vec![
            "Próbowałeś grać w pokera w ogrodzie, ale wjechałeś w krasnala ogrodowego. Właściciel się wkurzył i kazał zapłacić {amount} dolarów kary!",
            "Sanepid zamknął Twoje stoisko z lemoniadą. Chamski, prawda? Zabija młodych przedsiębiorców. Jeszcze grzywnę nałożył. Aż {amount} dolarów. O ja piernicze...",
            "Nie mając prawa jazdy wjechałeś w pieszego. Nic się mu nie stało, ale za uszkodzenie ciała, uszczerbek na zdrowiu i jazdę bez biletu... znaczy prawa jazdy musiałeś zapłacić {amount} dolarów.",
        ];

        let desc_templ = fail_responses
            .choose(&mut rand::rng())
            .unwrap_or(&"Coś poszło nie tak... straciłeś {amount}");
        let desc = desc_templ.replace("{amount}", &loss.to_string());

        db.remove_cash(user_id, loss).await?;
        db.update_timeout(user_id, "last_work", now).await?;

        ctx.send(
            CreateReply::default().embed(
                poise::serenity_prelude::CreateEmbed::new()
                    .title("❌ Za dużo byś chciał. Nie tym razem.")
                    .description(desc)
                    .color(0xFF0000),
            ),
        )
        .await?;

        Ok(())
    }
}
