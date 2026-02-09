use poise::CreateReply;
use crate::bot::{Context, Error};
use rand::Rng;
use rand::prelude::IndexedRandom;

#[poise::command(
    slash_command,
    prefix_command,
    description_localized("pl", "Coś skrajnie nielegalnego. Bardzo mała szansa wygranej. Ale bardzo duży zysk.")
)]
pub async fn crime(ctx: Context<'_>) -> Result<(), Error> {
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
        .duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;

    let cooldown = 60 * 60;
    let time_passed = now - timeouts.last_crime;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;
        ctx.send(CreateReply::default()
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title("⏳ Może trochę rozwagi?")
                .description(format!("Zachciało Ci się coś porobić nielegalnego. Okej. Rozumiem. Nie będę Cię osądzać. Ale jeszcze jest za głośno o tamtej aferze. Ludzie cię szukają. Jesteś na listach policji, Interpolu, Europolu, wszędzie jesteś. Weź trochę zaczekaj jak nie chcesz zdradzić gdzie się ukrywasz. Musisz zaczekać {} sekund", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    let chance = rand::rng().random_range(0..100);
    let how_much = rand::rng().random_range(3000..=7000);

    if chance < 20 {
        let responses = vec![
            "Okradłeś bank, a ekspedienta, bojąc się, że ją zabijesz najnowszym AK-47 Remastered, wyskoczyła z {amount} dolarów. Właściwie to z większej kwoty. Ale ty nie chciałeś aż tak wielkiej afery i zabrałeś tylko to.",
            "Znowu skontaktowałeś się z Natalią by shackować losowe strony na internecie. I nie zgadłeś. Shackowałeś walone Neocities. Wszystkie pieniądze supportersów są twoje, czyli nawet {amount} dolarów.",
            "Sprzedałeś znalezionego na ziemii iPhone 17 ultra pro max super proffessional ultimate i zyskałeś {amount} dolarów.",
            "Właśnie wbiłeś na pokład samolotu i odpaliłeś tam bombę. Wszyscy zginęli. Ale ty miałeś spadochron. Tobie nic się nie stało, a nawet ukradłeś rzeczy o łącznej wartości {amount} dolarów.",
            "Tobie coś odwaliło. Udało ci się obrabować skarbiec królowej Anglii i zaj*... znaczy wziąć uczciwie... aż {amount} dolarów! Królowa natychmiast dodała kwadrylion nowych zabezpieczeń. Ciekawe czy przełamiesz je drugi raz, by wziąć wypłatę po raz kolejny.",
            "Właśnie shackowałeś swoją szkołę i wpisałeś każdemu uczniowi tryliard szóstek. Nauczycielom zajęło ponad 5 dni roboczych, by manualnie usunąć cały ten chaos. Przy okazji okazało się, że nigdy nie została back-up'owana baza danych. Nauczyciel informatyki wypłacił Ci bug bounty w wysokości {amount} dolarów. Pomyśleć, że to zostało zrobione w 10 minut używając Metasploit."
        ];
        let desc_templ = responses.choose(&mut rand::rng()).unwrap_or(&"message się zepsuł :wilted_rose: ale zarobiłeś {amount}");
        let desc = desc_templ.replace("{amount}", &how_much.to_string());

        db.add_cash(user_id, how_much).await?;
        db.update_timeout(user_id, "last_work", now).await?;

        ctx.send(CreateReply::default()
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title("⚒️ Przestępstwo się opłaciło")
                .description(desc)
                .color(0x00FF00)
            )
        ).await?;

        Ok(())
    } else {
        let loss = how_much / 4;
        let fail_responses = vec![
            "Zapłaciłeś lotnisku {amount} dolarów kary, za próbę wniesienia bomby na pokład samolotu.",
            "Sprzedawca zoorientował się, że wciskasz mu kradzionego iPhone 17 ultra pro max super proffessional ultimate; wezwał policję i zażądał od ciebie {amount} dolarów.",
            "Nie udało Ci się oscamować rządu Brazylii, że liczba dziesiętnaście istnieje i nasłali na Ciebie wywiad. Na szczęście przekupiłeś go grzywną w wysokości {amount} dolarów.",
            "Królowa Anglii się skapnęła, że ktoś jej grzebie w skarbcu. Wezwała FBI i CIA. FBI prawie Cię zabiło najnowszym karabinem maszynowym AK-47 Ultra Russian Version Remastered Pro Max i zaczęło wymagać {amount} dolarów, które ty zapłaciłeś, by cię nie zabili do końca. Ty z kolei pozwałeś FBI i cudem uniknąłeś kolejnej kary. Niestety pozwu nie wygrałeś.",
            "Pomyślałeś więc, że wejdziesz do urzędu skarbowego i nałożysz podatek w wysokości 78 kwadryliardów złotych na swojego somsiada, który puszcał muzykę w nocy. Niestety, byłeś głupi i zapomniałeś wyłączyć kamer narzędziem od Natalii, więc policja obywatelska... znaczy milicja obywatelska... znaczy policja, czy jakoś tak, zamknęła Cię w więzieniu. Wyszłeś za kaucją wynoszącą {amount} dolarów."
        ];

        let desc_templ = fail_responses.choose(&mut rand::rng()).unwrap_or(&"Coś poszło nie tak... straciłeś {amount}");
        let desc = desc_templ.replace("{amount}", &loss.to_string());

        db.remove_cash(user_id, loss).await?;
        db.update_timeout(user_id, "last_crime", now).await?;

        ctx.send(CreateReply::default()
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title("❌ FBI czy tam kto inny Ci przeszkodził i nałożył grzywnę")
                .description(desc)
                .color(0xFF0000)
            )
        ).await?;

        Ok(())
    }
}