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
    collections::{HashSet},
    time::Duration,
    sync::{atomic::AtomicBool, Arc},
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
        macros::{help},
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
    dynamic_prefix
};
use crate::{
    helpers::{database_helper},
    structures::{cmd_data::*, errors::*},
    Lavalink
};
use aspotify::{Client as Spotify, ClientCredentials};

use settings::Settings;



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

    let client_credentials = ClientCredentials {
        id: settings.spotify.client_id.to_string(),
        secret: settings.spotify.client_token.to_string()
    };

    let spotify = Spotify::new(client_credentials);

    let pool = database_helper::obtain_db_pool(db_url).await?;
    let prefixes = database_helper::fetch_prefixes(&pool).await?;

    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners(owners)
                .dynamic_prefix(dynamic_prefix)
                .prefix("")
                .with_whitespace(false)
                .on_mention(Some(bot_id))
        })
        .help(&MY_HELP)
        .after(after)        
        .group(&FUN_GROUP)
        .group(&VOICE_GROUP)
        .group(&MUSIC_GROUP);

    let lava_client = LavalinkClient::builder(bot_id)
        .set_host(lavalink_url)
        .set_password(lavalink_password)
        .build(LavalinkHandler)
        .await?;

    let mut client = Client::builder(token)
        .event_handler(Handler {
            run_loop: AtomicBool::new(true),
        })
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        
        data.insert::<Lavalink>(lava_client);
        data.insert::<ConnectionPool>(pool);
        data.insert::<SettingsConf>(settings);
        data.insert::<VoiceTimerMap>(Arc::new(voice_timer_map));
        data.insert::<BotId>(bot_id);
        data.insert::<SpotifyClient>(Arc::new(spotify));
        data.insert::<PrefixMap>(Arc::new(prefixes));
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
