use anyhow::anyhow;
use chrono::{DateTime, Timelike};
use chrono::{Datelike, Duration};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use std::ops::Add;
use std::sync::Arc;
use tracing::{debug, info};

fn now() -> DateTime<chrono_tz::Tz> {
    chrono::Utc::now().with_timezone(&chrono_tz::Europe::Amsterdam)
}

fn next_weekday(weekday: chrono::Weekday) -> DateTime<chrono_tz::Tz> {
    let mut time = now();
    while time.weekday() != weekday {
        time = time.add(Duration::days(1))
    }
    time
}

fn next_invitation_time() -> DateTime<chrono_tz::Tz> {
    let time = next_weekday(chrono::Weekday::Tue);
    let time = time.with_hour(10).unwrap();
    let time = time.with_minute(0).unwrap();
    let time = time.with_second(0).unwrap();
    let time = time.with_nanosecond(0).unwrap();
    if time > now() {
        time
    } else {
        time.add(Duration::days(1))
    }
}

async fn sleep_until_next_invitation_time() {
    let sleep_until = next_invitation_time();
    debug!("Next invitation is send at: {sleep_until}");
    let duration = sleep_until.signed_duration_since(now());
    debug!("Therefore we will wait for: {duration}");
    tokio::time::sleep(duration.to_std().unwrap()).await;
}

fn next_session_date() -> DateTime<chrono_tz::Tz> {
    let time = next_weekday(chrono::Weekday::Thu);
    time.add(Duration::weeks(2))
}

#[derive(Clone)]
struct Bot {
    channel_id: ChannelId,
}

impl Bot {
    async fn send_planning_invitation(&self, ctx: Context) {
        let date = next_session_date();
        let date_localized = date.format_localized("%A %e %B", chrono::Locale::nl_NL);
        let text = format!("De volgende datum voor een potentiele sessie is {}.\n\nReageer even met 👍 of 👎 om aan te geven of je kan.", date_localized);

        self.channel_id
            .send_message(&ctx.http, |message| message.content(text))
            .await
            .unwrap();
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);
        let bot = self.clone();

        tokio::spawn(async move {
            loop {
                sleep_until_next_invitation_time().await;
                info!("Sending planning invitation");
                bot.send_planning_invitation(Context::clone(&ctx)).await;
            }
        });
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
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
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let bot = Bot {
        channel_id: ChannelId(channel_id),
    };

    let client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");
    Ok(client.into())
}
