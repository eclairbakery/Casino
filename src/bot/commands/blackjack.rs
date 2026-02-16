use crate::bot::{Context, Error};
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateActionRow, CreateButton, ButtonStyle, ComponentInteractionCollector, CreateInteractionResponse, CreateInteractionResponseMessage};
use rand::RngExt;
use rand::prelude::IndexedRandom;
use std::time::Duration;
use anyhow::anyhow;

#[derive(Clone, Copy)]
struct Card {
    name: &'static str,
    value: i32,
}

const DECK: [Card; 13] = [
    Card {
        name: "2",
        value: 2,
    },
    Card {
        name: "3",
        value: 3,
    },
    Card {
        name: "4",
        value: 4,
    },
    Card {
        name: "5",
        value: 5,
    },
    Card {
        name: "6",
        value: 6,
    },
    Card {
        name: "7",
        value: 7,
    },
    Card {
        name: "8",
        value: 8,
    },
    Card {
        name: "9",
        value: 9,
    },
    Card {
        name: "T",
        value: 10,
    },
    Card {
        name: "J",
        value: 10,
    },
    Card {
        name: "Q",
        value: 10,
    },
    Card {
        name: "K",
        value: 10,
    },
    Card {
        name: "A",
        value: 11,
    },
];

fn get_sum(hand: &[Card]) -> i32 {
    let mut sum = hand.iter().map(|c| c.value).sum();
    let mut ace_count = hand.iter().filter(|c| c.name == "A").count();
    while sum > 21 && ace_count > 0 {
        sum -= 10;
        ace_count -= 1;
    }
    sum
}

fn format_hand(hand: &[Card]) -> String {
    let names: Vec<&str> = hand.iter().map(|c| c.name).collect();
    format!("[{}]", names.join(", "))
}

#[poise::command(
    slash_command,
    prefix_command,
    aliases("bj"),
    description_localized(
        "pl",
        "Zagraj w Blackjacka przeciwko wykwalifikowanemu krupierowi z 20 latami doświadczenia w branży."
    )
)]
pub async fn blackjack(ctx: Context<'_>, bet: f64) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    let already_playing = {
        let mut active = ctx
            .data()
            .active_players
            .lock()
            .map_err(|_| anyhow!("Mutex error"))?;
        if active.contains(&user_id) {
            true
        } else {
            active.insert(user_id);
            false
        }
    };

    if already_playing {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Już grasz!")
                    .description("Dokończ swoją poprzednią partię, zanim zaczniesz nową.")
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    let game_result = start_blackjack(ctx, db, user_id, bet).await;

    if let Ok(mut active) = ctx.data().active_players.lock() {
        active.remove(&user_id);
    }

    game_result
}

async fn start_blackjack(
    ctx: Context<'_>,
    db: &crate::services::database::abstraction::DbManager,
    user_id: i64,
    bet: f64,
) -> Result<(), Error> {
    if bet <= 50.0 {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Za mała stawka")
                    .description("Minimum to 50 dolarów.")
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    let user_data = db.ensure_member(user_id).await?;
    let member = user_data.user;
    let timeouts = user_data.timeouts;
    if member.cash < bet as f64 {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("❌ Jesteś biedny")
                    .description(format!("Masz zaledwie `{}` dolarów...", member.cash))
                    .color(0xFF0000),
            ),
        )
        .await?;
        return Ok(());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let cooldown = 15;
    let time_passed = now - timeouts.last_hazarded;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;
        ctx.send(CreateReply::default()
            .embed(CreateEmbed::new()
                .title(":hourglass_flowing_sand: Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    let mut player_hand = vec![
        *DECK.choose(&mut rand::rng()).unwrap(),
        *DECK.choose(&mut rand::rng()).unwrap(),
    ];
    let mut dealer_hand = vec![
        *DECK.choose(&mut rand::rng()).unwrap(),
        *DECK.choose(&mut rand::rng()).unwrap(),
    ];

    let ctx_id = ctx.id();
    let hit_id = format!("{}hit", ctx_id);
    let stand_id = format!("{}stand", ctx_id);
    let mut status_message = String::from("Twoja tura: Dobierasz czy pasujesz?");
    let mut game_over = false;

    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("🃏 Blackjack")
                    .description(&status_message)
                    .field(
                        "Twoje karty",
                        format!(
                            "{} (Suma: {})",
                            format_hand(&player_hand),
                            get_sum(&player_hand)
                        ),
                        true,
                    )
                    .field(
                        "Karty krupiera",
                        format!("[{}, ?]", dealer_hand[0].name),
                        true,
                    )
                    .color(0x00AEFF),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new(&hit_id)
                    .label("Dobierz")
                    .style(ButtonStyle::Primary),
                CreateButton::new(&stand_id)
                    .label("Pasuj")
                    .style(ButtonStyle::Secondary),
            ])]),
    )
    .await?;

    while let Some(press) = ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(Duration::from_secs(45))
        .await
    {
        if press.user.id != ctx.author().id {
            press
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await?;
            continue;
        }

        if press.data.custom_id == hit_id {
            player_hand.push(*DECK.choose(&mut rand::rng()).unwrap());
            if get_sum(&player_hand) > 21 {
                status_message = format!("Fura! Przekroczyłeś 21. Przegrałeś **{}** 💰.", bet);
                game_over = true;
                db.change_cash(user_id, -bet).await?;
            }
        } else if press.data.custom_id == stand_id {
            game_over = true;

            while get_sum(&dealer_hand) < 17 {
                dealer_hand.push(*DECK.choose(&mut rand::rng()).unwrap());
            }

            let p_sum = get_sum(&player_hand);
            let d_sum = get_sum(&dealer_hand);

            if d_sum > 21 {
                let win = bet;
                status_message = format!("Krupier fura ({})! Wygrałeś **{}** dolarów!", d_sum, win);
                db.change_cash(user_id, win).await?;
            } else if p_sum > d_sum {
                if rand::rng().random_range(1..=100) <= 5 {
                    status_message =
                        format!("Remis techniczny! Krupier cudem wyrównał do `{}`.", p_sum);
                } else {
                    let win = bet * 0.95;
                    status_message = format!(
                        "Wygrałeś! `{}` vs `{}`. Zyskałeś **{}** dolarów",
                        p_sum, d_sum, win
                    );
                    db.change_cash(user_id, win).await?;
                }
            } else if p_sum == d_sum {
                status_message = format!("Remis! Tracisz połowę, czyli **{}** dolarów.", bet / 2.0);
                db.change_cash(user_id, -bet / 2.0).await?;
            } else {
                status_message = format!(
                    "Przegrałeś! Krupier ma `{}`. Tracisz **{}** dolarów.",
                    d_sum, bet
                );
                db.change_cash(user_id, -bet).await?;
            }
        }

        let mut embed = CreateEmbed::new()
            .title("🃏 Blackjack")
            .description(&status_message)
            .field(
                "Twoje karty",
                format!(
                    "{} (Suma: {})",
                    format_hand(&player_hand),
                    get_sum(&player_hand)
                ),
                true,
            )
            .color(if game_over {
                if status_message.contains("Wygrałeś") || status_message.contains("Remis") {
                    0x00FF00
                } else {
                    0xFF0000
                }
            } else {
                0x00AEFF
            });

        if game_over {
            embed = embed.field(
                "Karty krupiera",
                format!(
                    "{} (Suma: {})",
                    format_hand(&dealer_hand),
                    get_sum(&dealer_hand)
                ),
                true,
            );
            press
                .create_response(
                    ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(vec![]),
                    ),
                )
                .await?;
            break;
        } else {
            embed = embed.field(
                "Karty krupiera",
                format!("[{}, ?]", dealer_hand[0].name),
                true,
            );
            press
                .create_response(
                    ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new().embed(embed),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}
