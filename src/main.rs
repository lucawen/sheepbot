extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;


mod commands;
mod utils;
mod handler;
mod settings;
mod models;
mod helpers;
mod structures;

use std::{
    collections::HashSet,
    time::Duration,
    sync::{Arc},
};
use dashmap::DashMap;
use futures::future::AbortHandle;

use serenity::{
    http::Http,
    client::{
        Client,
    },
    framework::standard::{
        Args, StandardFramework, CommandGroup,
        HelpOptions, help_commands, CommandResult,
        macros::{help, hook},
    },
    model::prelude::*,
    model::id::GuildId,
    prelude::*,
};

use lavalink_rs::{
    LavalinkClient
};
use songbird::SerenityInit;

use tracing_subscriber::{
    FmtSubscriber,
    EnvFilter,
};

use crate::commands::*;

use crate::handler::{
    Handler,
    LavalinkHandler,
    after,
    SettingsConf,
    normal_message
};
use crate::{
    helpers::{command_utils, database_helper},
    structures::{cmd_data::*, commands::*, errors::*},
    Lavalink
};

use settings::Settings;

use crate::utils::database::{obtain_pool};


#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, &help_options, groups, owners).await;
    Ok(())
}

#[hook]
// Sets a custom prefix for a guild.
pub async fn dynamic_prefix(ctx: &Context, _msg: &Message) -> Option<String> {
    let data = ctx.data.read().await;
    let settings = data.get::<SettingsConf>().unwrap();
    let default_prefix = settings.discord.prefix.as_str();

    let prefix: String;
    // TODO: Fix prefix from database
    // if let Some(id) = guild_id {
    //     let pool = data.get::<ConnectionPool>().unwrap();

    //     let res = sqlx::query(
    //         "SELECT prefix FROM prefixes WHERE guild_id = $1",
    //     )
    //     .bind(id.0 as i64)
    //     .fetch_one(pool)
    //     .await;

    //     prefix = if let Ok(data) = res {
    //         data.try_get("prefix").unwrap_or(default_prefix.to_string())
    //     } else {
    //         error!("I couldn't query the database for getting guild prefix.");
    //         default_prefix.to_string()
    //     }
    // } else {
    //     prefix = default_prefix.to_string();
    // };

    prefix = default_prefix.to_string();

    Some(prefix)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings =  match Settings::new() {
        Ok(conf) => conf,
        Err(why) => panic!("Could not read config: {:?}", why),
    };

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    let token = &settings.discord.token;
    let lavalink_url = &settings.lavalink.url;
    let lavalink_password = &settings.lavalink.password;
    let db_url = &settings.database_url;

    let http = Http::new_with_token(token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let voice_timer_map: DashMap<GuildId, AbortHandle> = DashMap::new();
    

    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners(owners)
                .dynamic_prefix(dynamic_prefix)
                .with_whitespace(false)
                .on_mention(Some(bot_id))
        })
        .help(&MY_HELP)
        .normal_message(normal_message)
        .after(after)        
        .group(&FUN_GROUP)
        // .group(&CONFIG_GROUP)
        .group(&MUSIC_GROUP);

    let lava_client = LavalinkClient::builder(bot_id)
        .set_host(lavalink_url)
        .set_password(lavalink_password)
        .build(LavalinkHandler)
        .await?;

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        let pool = obtain_pool(db_url).await?;
        
        data.insert::<Lavalink>(lava_client);
        data.insert::<ConnectionPool>(pool);
        data.insert::<SettingsConf>(settings);
        data.insert::<VoiceTimerMap>(Arc::new(voice_timer_map));
    }

    // Here we clone a lock to the Shard Manager, and then move it into a new
    // thread. The thread will unlock the manager and print shards' status on a
    // loop.
    let manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let lock = manager.lock().await;
            let shard_runners = lock.runners.lock().await;

            for (id, runner) in shard_runners.iter() {
                println!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id,
                    runner.stage,
                    runner.latency,
                );
            }
        }
    });
    

    // Start two shards. Note that there is an ~5 second ratelimit period
    // between when one shard can start after another.
    if let Err(why) = client.start_shards(2).await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}
