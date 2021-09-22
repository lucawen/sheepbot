use tracing::{error, info};

use serenity::{client::Context};
use std::sync::atomic::{AtomicBool, Ordering};

use serenity::{
    async_trait,
    client::EventHandler,
    framework::standard::{macros::hook, CommandResult, CommandError},
    model::{
        guild::{Guild, GuildUnavailable},
        id::GuildId,
        channel::{Message, GuildChannel}, gateway::Ready
    },
};

use lavalink_rs::{gateway::*, model::*, LavalinkClient};

use crate::utils::setup::get_link_only_modes;
use crate::{
    ConnectionPool
};
use crate::{
    helpers::start_loops,
    structures::cmd_data::{SettingsConf, PrefixMap}
};

pub struct Handler {
    pub run_loop: AtomicBool,
}

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

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        if let Err(e) = thread.id.join_thread(ctx).await {
            println!("Error in thread join! (ID {}): {}", thread.id, e);
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        let pool = ctx
            .data
            .read()
            .await
            .get::<ConnectionPool>()
            .cloned()
            .unwrap();

        if is_new {
            sqlx::query!(
                "INSERT INTO guild_info VALUES($1, null) ON CONFLICT DO NOTHING",
                guild.id.0 as i64
            )
            .execute(&pool)
            .await
            .unwrap();
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable, _full: Option<Guild>) {
        let (pool, prefixes) = {
            let data = ctx.data.read().await;
            let pool = data.get::<ConnectionPool>().cloned().unwrap();
            let prefixes = data.get::<PrefixMap>().cloned().unwrap();

            (pool, prefixes)
        };

        if let Err(e) = sqlx::query!(
            "DELETE FROM guild_info WHERE guild_id = $1",
            incomplete.id.0 as i64
        )
        .execute(&pool)
        .await
        {
            eprintln!("Error in guild removal! (ID {}): {}", incomplete.id.0, e)
        }

        if prefixes.contains_key(&incomplete.id) {
            prefixes.remove(&incomplete.id);
        }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        if self.run_loop.load(Ordering::Relaxed) {
            self.run_loop.store(false, Ordering::Relaxed);

            println!("Running guild pruner!");
            if let Err(e) = start_loops::guild_pruner(&ctx).await {
                panic!("Error when pruning guilds! {}", e);
            }

            let pool = ctx
                .data
                .read()
                .await
                .get::<ConnectionPool>()
                .cloned()
                .unwrap();


            println!("Starting activity loop!");
            tokio::spawn(async move {
                start_loops::activity_loop(&ctx.shard).await;
            });
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
pub async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    let (prefixes, default_prefix) = {
        let data = ctx.data.read().await;
        let prefixes = data.get::<PrefixMap>().cloned().unwrap();
        let settings = data
            .get::<SettingsConf>()
            .unwrap();
        let default_prefix = &settings.discord.prefix;
        (prefixes, default_prefix.to_string())
    };

    let guild_id = msg.guild_id.unwrap();

    let wrapped_prefix = prefixes.get(&guild_id);

    match wrapped_prefix {
        Some(prefix_guard) => Some(prefix_guard.value().to_owned()),
        None => Some(default_prefix),
    }
}

// After a command is executed, goto here
#[hook]
pub async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        let part_1 = "Looks like the bot encountered an error! \n";
        let error_string = format!("{}", part_1);

        let _ = msg
            .channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.color(0xff69b4);
                    e.title("Aw Snap!");
                    e.description(error_string);
                    e.field("Command Name", cmd_name, false);
                    e.field("Error", format!("```{} \n```", why), false);
                    e
                })
            })
            .await;
    }
}