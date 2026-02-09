use crate::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use rand::Rng;
use std::time::Duration;

#[poise::command(
    slash_command,
    prefix_command,
    aliases("bj"),
    description_localized("pl", "Zagraj w Blackjacka przeciwko wykwalifikowanemu krupierowi z 20 latami doświadczenia w branży.")
)]
pub async fn blackjack(
    ctx: Context<'_>,
    bet: i64,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get() as i64;
    let db = &ctx.data().db;

    if bet <= 50 {
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ Ale weź coś postaw...")
                .description("Ja ci tak polecam od 50 w górę.")
                .color(0xFF0000)
        )).await?;
        return Ok(());
    }

    let (member, timeouts) = db.ensure_member(user_id).await?;
    if member.cash < bet {
        ctx.send(CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("❌ Jesteś biedny")
                .description(format!("Masz za mało gotówki! Posiadasz zaledwie `{}` dolarów...", member.cash))
                .color(0xFF0000)
        )).await?;
        return Ok(());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;

    let cooldown = 15;
    let time_passed = now - timeouts.last_hazarded;

    if time_passed < cooldown {
        let remaining = cooldown - time_passed;
        ctx.send(CreateReply::default()
            .embed(poise::serenity_prelude::CreateEmbed::new()
                .title("⏳ Czekaj chwilę")
                .description(format!("No ten... kasyno zawsze wygrywa. A przynajmniej tak ma być. Więc nie możesz spamić hazardem. Pozdrawiam. Wróć za **{} sekund**.", remaining))
                .color(0xFF0000))
        ).await?;
        return Ok(());
    }

    db.update_timeout(user_id, "last_hazarded", now).await?;

    let mut player_hand = vec![rand::rng().random_range(2..=11), rand::rng().random_range(2..=11)];
    let mut dealer_hand = vec![rand::rng().random_range(2..=11), rand::rng().random_range(2..=11)];

    let ctx_id = ctx.id();
    let hit_id = format!("{}hit", ctx_id);
    let stand_id = format!("{}stand", ctx_id);

    let mut game_over = false;
    let mut status_message = String::from("Twoja tura i te odwieczne pytanie każdego hazardzisty: Dobierasz czy pasujesz?");

    ctx.send(CreateReply::default()
        .embed(serenity::CreateEmbed::new()
            .title("🃏 Blackjack")
            .description(&status_message)
            .field("Twoje karty", format!("{:?} (Suma: {})", player_hand, player_hand.iter().sum::<i32>()), true)
            .field("Karty krupiera", format!("[{}, ?]", dealer_hand[0]), true)
            .color(0x00AEFF)
        )
        .components(vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&hit_id).label("Dobierz").style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new(&stand_id).label("Pasuj").style(serenity::ButtonStyle::Secondary),
        ])])
    ).await?;

    while let Some(press) = serenity::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(Duration::from_secs(30))
        .await
    {
        if press.user.id != ctx.author().id {
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
            continue;
        }

        if press.data.custom_id == hit_id {
            player_hand.push(rand::rng().random_range(2..=11));
            let player_sum: i32 = player_hand.iter().sum();

            if player_sum > 21 {
                status_message = format!("F*ck! Przekroczyłeś 21. Przegrałeś **{}** dolarów.", bet);
                game_over = true;
                db.add_cash(user_id, -bet).await?;
            }
        } else if press.data.custom_id == stand_id {
            game_over = true;
            let mut dealer_sum: i32 = dealer_hand.iter().sum();
            
            while dealer_sum < 17 {
                dealer_hand.push(rand::rng().random_range(2..=11));
                dealer_sum = dealer_hand.iter().sum();
            }

            let player_sum: i32 = player_hand.iter().sum();
            
            if dealer_sum > 21 {
                let win = bet;
                status_message = format!("Krupier fura! Pękł z wynikiem {}. Wygrałeś **{}** dolarów!", dealer_sum, win);
                db.add_cash(user_id, win).await?;
            } else if player_sum > dealer_sum {
                let win = bet;
                status_message = format!("Wygrałeś! Masz lepszy układ. Zyskałeś **{}** dolarów!", win);
                db.add_cash(user_id, win).await?;
            } else if player_sum == dealer_sum {
                status_message = String::from("Remis! Pieniądze wracają do Ciebie.");
            } else {
                status_message = format!("Przegrałeś! Krupier ma `{}`. Straciłeś **{}** dolarów.", dealer_sum, bet);
                db.add_cash(user_id, -bet).await?;
            }
        }
    
        let current_player_sum: i32 = player_hand.iter().sum();
        let current_dealer_sum: i32 = dealer_hand.iter().sum();
        
        let mut embed = serenity::CreateEmbed::new()
            .title("🃏 Blackjack")
            .description(&status_message)
            .field("Twoje karty", format!("{:?} (Suma: {})", player_hand, current_player_sum), true)
            .color(if game_over { 
                if status_message.contains("Wygrałeś") || status_message.contains("Remis") { 0x00FF00 } else { 0xFF0000 } 
            } else { 0x00AEFF });

        if game_over {
            embed = embed.field("Karty krupiera", format!("{:?} (Suma: {})", dealer_hand, current_dealer_sum), true);
            press.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![])
            )).await?;
            break;
        } else {
            embed = embed.field("Karty krupiera", format!("[{}, ?]", dealer_hand[0]), true);
            press.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new().embed(embed)
            )).await?;
        }
    } 

    Ok(())
}
