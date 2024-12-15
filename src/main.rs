use crate::time_helper::*;
use anyhow::anyhow;
use serenity::all::CreateMessage;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::{Arc, OnceLock};
use tracing::{debug, info};

mod time_helper;

async fn sleep_until_next_invitation_time() {
    let sleep_until = now().next_invitation_time();
    debug!("Next invitation is send at: {sleep_until}");
    let duration = sleep_until.signed_duration_since(now());
    debug!("Therefore we will wait for: {duration}");
    tokio::time::sleep(duration.to_std().unwrap()).await;
}

#[derive(Clone, Default)]
struct Bot {
    channel_id: ChannelId,
    self_user: OnceLock<CurrentUser>,
    owner: OnceLock<User>,
}

impl Bot {
    async fn send_planning_invitation(&self, ctx: Context) {
        let date = now().next_session_date();
        let date_localized = date.format_localized("%A %e %B", chrono::Locale::nl_NL);
        let text = format!("@everyone\nDe volgende datum voor een potentiele sessie is {}.\n\nReageer even met 👍 of 👎 om aan te geven of je kan.", date_localized);

        let message = CreateMessage::new().content(text);
        self.channel_id
            .send_message(&ctx.http, message)
            .await
            .unwrap();
    }

    async fn send_owner_message(&self, ctx: &Context, content: impl Into<String>) {
        let owner = self.owner.get().unwrap();
        let owner_dm_channel = owner.create_dm_channel(&ctx.http).await.unwrap();
        owner_dm_channel.say(&ctx.http, content).await.unwrap();
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        self.self_user.set(ready.user).unwrap();

        let application = ctx.http.get_current_application_info().await.unwrap();
        let owner = application.owner.unwrap();
        info!("My owner is {}", owner.name);
        self.owner.set(owner).unwrap();

        let ctx = Arc::new(ctx);
        let bot = self.clone();

        self.send_owner_message(&ctx, "Hello owner, I just started up")
            .await;
        tokio::spawn(async move {
            loop {
                sleep_until_next_invitation_time().await;
                info!("Sending planning invitation");
                bot.send_planning_invitation(Context::clone(&ctx)).await;
            }
        });
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        let self_user = self.self_user.get().unwrap();

        let message = add_reaction.message(&ctx.http).await.unwrap();
        if message.author.id == self_user.id {
            let reaction_user = add_reaction.user(&ctx.http).await.unwrap();
            let channel = message.channel(&ctx.http).await.unwrap();
            info!(
                "Reaction received: {} from {}",
                add_reaction.emoji, reaction_user.name
            );
            self.send_owner_message(
                &ctx,
                format!(
                    "{} heeft gereageerd in {}",
                    reaction_user.mention(),
                    channel.mention()
                ),
            )
            .await;
        }
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secret_store: shuttle_runtime::SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let channel_id = if let Some(token) = secret_store.get("DISCORD_CHANNEL_ID") {
        token.parse().unwrap()
    } else {
        return Err(anyhow!("'DISCORD_CHANNEL_ID' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let bot = Bot {
        channel_id: ChannelId::new(channel_id),
        ..Default::default()
    };

    let client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");
    Ok(client.into())
}
