use tracing::{error, info};

use serenity::{client::Context};

use serenity::{
    async_trait,
    client::EventHandler,
    framework::standard::{macros::hook, CommandResult},
    model::{
        channel::Message, gateway::Ready,
        prelude::Guild,
    },
};

use lavalink_rs::{gateway::*, model::*, LavalinkClient};
use serenity::prelude::*;

use crate::settings::Settings;
use sqlx::PgPool;

use crate::dynamic_prefix;
use crate::utils::database::initialize_tables;

use crate::utils::setup::get_link_only_modes;

pub(crate) struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

pub(crate) struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}

pub(crate) struct SettingsConf;

impl TypeMapKey for SettingsConf {
    type Value = Settings;
}

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            info!(
                "{} is connected on shard {}/{}!",
                ready.user.name, shard[0], shard[1],
            );
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        // We'll initialize the database tables for a guild if it's new.
        if !is_new {
            return;
        }

        initialize_tables(&ctx, &guild).await;
    }
}

#[hook]
pub async fn normal_message(ctx: &Context, msg: &Message) {
    let data_read = ctx.data.read().await;
    let settings = data_read.get::<SettingsConf>().unwrap();

    // get prefix for the guild.
    let prefix = match dynamic_prefix(&ctx, &msg).await {
        Some(p) => p,
        None => String::from(&settings.discord.prefix),
    };
    if !!!msg.content.starts_with(&prefix) && !!!msg.author.bot {
        let pool = data_read.get::<ConnectionPool>().unwrap();
        if let Some(guild_id) = msg.guild_id {
            let rules = get_link_only_modes(pool, guild_id, msg.channel_id).await;
            match rules {
                Ok(res) => {
                    let string_rules = res.iter().map(|x| x.url.clone()).collect::<Vec<_>>();

                    if !!!string_rules.iter().any(|i| msg.content.contains(i)) && !!!res.is_empty()
                    {
                        msg.delete(&ctx.http).await.unwrap();
                    }
                }
                Err(err) => {
                    println!("Error when trying to get links only: {:?}", err,);
                }
            }
        }
    }
}

pub(crate) struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
        info!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: LavalinkClient, event: TrackFinish) {
        info!("Track finished!\nGuild: {}", event.guild_id);
    }
}

#[hook]
pub(crate) async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: CommandResult) {
    if let Err(why) = &error {
        error!("Error while running command {}", &cmd_name);
        error!("{:?}", &error);

        let err = why.to_string();
        if msg.channel_id.say(ctx, &err).await.is_err() {
            error!(
                "Unable to send messages on channel id {}",
                &msg.channel_id.0
            );
        };
    }
}
