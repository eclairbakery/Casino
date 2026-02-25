use crate::bot::Context;
use crate::bot::format_number::format_number;
use anyhow::Error;
use poise::CreateReply;
use rand::RngExt;
use rand::prelude::IndexedRandom;
use serenity::all::CreateEmbed;

const RESPONSES: [&str; 6] = [
    "Okradłeś bank, a ekspedienta, bojąc się, że ją zabijesz najnowszym AK-47 Remastered, wyskoczyła z {amount}zł. Właściwie to z większej kwoty. Ale ty nie chciałeś aż tak wielkiej afery i zabrałeś tylko to.",
    "Znowu skontaktowałeś się z Natalią by shackować losowe strony na internecie. I nie zgadłeś. Shackowałeś walone Neocities. Wszystkie pieniądze supportersów są twoje, czyli nawet {amount}zł.",
    "Sprzedałeś znalezionego na ziemii iPhone 17 ultra pro max super proffessional ultimate i zyskałeś {amount}zł.",
    "Właśnie wbiłeś na pokład samolotu i odpaliłeś tam bombę. Wszyscy zginęli. Ale ty miałeś spadochron. Tobie nic się nie stało, a nawet ukradłeś rzeczy o łącznej wartości {amount}zł.",
    "Tobie coś odwaliło. Udało ci się obrabować skarbiec królowej Anglii i zaj*... znaczy wziąć uczciwie... aż {amount}zł! Królowa natychmiast dodała kwadrylion nowych zabezpieczeń. Ciekawe czy przełamiesz je drugi raz, by wziąć wypłatę po raz kolejny.",
    "Właśnie shackowałeś swoją szkołę i wpisałeś każdemu uczniowi tryliard szóstek. Nauczycielom zajęło ponad 5 dni roboczych, by manualnie usunąć cały ten chaos. Przy okazji okazało się, że nigdy nie została back-up'owana baza danych. Nauczyciel informatyki wypłacił Ci bug bounty w wysokości {amount}zł. Pomyśleć, że to zostało zrobione w 10 minut używając Metasploit.",
];

const FAIL_RESPONSES: [&str; 5] = [
    "Zapłaciłeś lotnisku {amount}zł kary, za próbę wniesienia bomby na pokład samolotu.",
    "Sprzedawca zoorientował się, że wciskasz mu kradzionego iPhone 17 ultra pro max super proffessional ultimate; wezwał policję i zażądał od ciebie {amount}zł.",
    "Nie udało Ci się oscamować rządu Brazylii, że liczba dziesiętnaście istnieje i nasłali na Ciebie wywiad. Na szczęście przekupiłeś go grzywną w wysokości {amount}zł.",
    "Królowa Anglii się skapnęła, że ktoś jej grzebie w skarbcu. Wezwała FBI i CIA. FBI prawie Cię zabiło najnowszym karabinem maszynowym AK-47 Ultra Russian Version Remastered Pro Max i zaczęło wymagać {amount}zł, które ty zapłaciłeś, by cię nie zabili do końca. Ty z kolei pozwałeś FBI i cudem uniknąłeś kolejnej kary. Niestety pozwu nie wygrałeś.",
    "Pomyślałeś więc, że wejdziesz do urzędu skarbowego i nałożysz podatek w wysokości 78 kwadryliardów złotych na swojego somsiada, który puszcał muzykę w nocy. Niestety, byłeś głupi i zapomniałeś wyłączyć kamer narzędziem od Natalii, więc policja obywatelska... znaczy milicja obywatelska... znaczy policja, czy jakoś tak, zamknęła Cię w więzieniu. Wyszłeś za kaucją wynoszącą {amount}zł.",
];

const COOLDOWN: i64 = 60 * 60;

#[poise::command(
    slash_command,
    prefix_command,
    description_localized(
        "pl",
        "Coś skrajnie nielegalnego. Bardzo mała szansa wygranej. Ale bardzo duży zysk."
    )
)]
pub async fn crime(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let user_data = db.ensure_member(user_id).await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let time_passed = now - user_data.timeouts.last_crime;

    if time_passed < COOLDOWN {
        let remaining = COOLDOWN - time_passed;
        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title("⏳ Może trochę rozwagi?")
                .description(format!("Zachciało Ci się coś porobić nielegalnego. Okej. Rozumiem. Nie będę Cię osądzać. Ale jeszcze jest za głośno o tamtej aferze. Ludzie cię szukają. Jesteś na listach policji, Interpolu, Europolu, wszędzie jesteś. Weź trochę zaczekaj jak nie chcesz zdradzić gdzie się ukrywasz. Musisz zaczekać {} sekund", remaining))
                .color(0xFF0000))
        ).await?;

        return Ok(());
    }

    let chance = {
        let mut rng = rand::rng();

        rng.random_range(0..100)
    };

    if chance < 20 {
        let (how_much, desc_template) = {
            let mut rng = rand::rng();

            let how_much = rng.random_range(900_00..=1500_00);

            let desc_template = RESPONSES
                .choose(&mut rng)
                .unwrap_or(&"message się zepsuł :wilted_rose: ale zarobiłeś {amount}");

            (how_much, desc_template)
        };

        let desc = desc_template.replace("{amount}", &format_number(how_much));

        user_data.user.change_wallet(&db.pool, how_much).await?;
        db.update_timeout(user_id, "last_work", now).await?;

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("⚒️ Przestępstwo się opłaciło")
                    .description(desc)
                    .color(0x00FF00),
            ),
        )
        .await?;

        Ok(())
    } else {
        let (how_much, desc_template) = {
            let mut rng = rand::rng();

            let how_much = rng.random_range(600_00..=3000_00);

            let desc_template = FAIL_RESPONSES
                .choose(&mut rand::rng())
                .unwrap_or(&"Coś poszło nie tak... straciłeś {amount}");

            (how_much, desc_template)
        };

        let loss = how_much / 4;

        let desc = desc_template.replace("{amount}", &format_number(loss));

        user_data.user.change_wallet(&db.pool, -loss).await?;
        db.update_timeout(user_id, "last_crime", now).await?;

        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ FBI czy tam kto inny Ci przeszkodził i nałożył grzywnę")
                    .description(desc)
                    .color(0xFF0000),
            ),
        )
        .await?;

        Ok(())
    }
}
