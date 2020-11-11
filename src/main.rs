extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;


mod commands;
mod utils;
mod handler;
mod settings;
mod models;

use tracing::{error, info};

use std::{
    sync::Arc,
    collections::HashSet,
    time::Duration,
};

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
    prelude::*,
};

use lavalink_rs::{
    LavalinkClient
};


use crate::commands::*;

use crate::handler::{
    Handler,
    Lavalink,
    VoiceManager,
    VoiceGuildUpdate,
    LavalinkHandler,
    ConnectionPool,
    after,
    SettingsConf
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
pub async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    let guild_id = &msg.guild_id;

    let data = ctx.data.read().await;
    let settings = data.get::<SettingsConf>().unwrap();
    let default_prefix = settings.discord.prefix.as_str();

    let prefix: String;
    if let Some(id) = guild_id {
        let pool = data.get::<ConnectionPool>().unwrap();

        let res = sqlx::query!(
            "SELECT prefix FROM prefixes WHERE guild_id = $1",
            id.0 as i64
        )
        .fetch_one(pool)
        .await;

        prefix = if let Ok(data) = res {
            if let Some(p) = data.prefix {
                p
            } else {
                default_prefix.to_string()
            }
        } else {
            error!("I couldn't query the database for getting guild prefix.");
            default_prefix.to_string()
        }
    } else {
        prefix = default_prefix.to_string();
    };

    Some(prefix)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings =  match Settings::new() {
        Ok(conf) => conf,
        Err(why) => panic!("Could not read config: {:?}", why),
    };

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
    

    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners(owners)
                .dynamic_prefix(dynamic_prefix)
                .with_whitespace(false)
                .on_mention(Some(bot_id))
        })
        .help(&MY_HELP)
        .after(after)
        .group(&MUSIC_GROUP)
        .group(&FUN_GROUP)
        .group(&CONFIG_GROUP);

    let mut client = Client::new(token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;

        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<VoiceGuildUpdate>(Arc::new(RwLock::new(HashSet::new())));

        let mut lava_client = LavalinkClient::new(bot_id);

        lava_client.set_host(lavalink_url);
        lava_client.set_password(lavalink_password);

        let lava = lava_client.initialize(LavalinkHandler).await?;
        data.insert::<Lavalink>(lava);

        let pool = obtain_pool(db_url).await?;
        data.insert::<ConnectionPool>(pool);
        data.insert::<SettingsConf>(settings);
    }

    // Here we clone a lock to the Shard Manager, and then move it into a new
    // thread. The thread will unlock the manager and print shards' status on a
    // loop.
    let manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::delay_for(Duration::from_secs(30)).await;

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
