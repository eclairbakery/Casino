use crate::bot::{ Error, Data};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { ctx, .. } => {
            let _ = ctx.send(CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("🥀 Komenda się zwaliła...")
                    .description("Weź spinguj gorciu'a, hashcata czy kogokolwiek, ale spinguj...")
                )
            ).await;
        }
        poise::FrameworkError::CommandStructureMismatch { ctx, .. } => {
            let _ = ctx.send(CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("🤨 Coś za mało tych argumentów")
                    .description("Weź. Nie baw się ze mną. Dawaj te argumenty. Albo wezwę istotę wyższą.")
                )
            ).await;
        }
        poise::FrameworkError::ArgumentParse { ctx, .. } => {
            let _ = ctx.send(CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("🤦🏻 Nie umiem czytać")
                    .description("Coś ty za argument dał? Czy ty naprawdę nie wiesz jak działa ta komenda? Potrzebujesz specjalnego traktowania?")
                )
            ).await;
        }
        _ => poise::builtins::on_error(error).await.unwrap(), 
    }
}
