mod commands;
mod utils;
mod handler;

use dotenv::dotenv;

use std::{
    env,
    sync::Arc,
    collections::HashSet,
    time::Duration,
};

use serenity::{
    http::Http,
    client::{
        Client,
    },
    framework::{
        StandardFramework,
    },
};

use serenity::prelude::*;
use lavalink_rs::{
    LavalinkClient
};


use crate::commands::*;

use crate::handler::{
    Handler,
    Lavalink,
    VoiceManager,
    VoiceGuildUpdate,
    LavalinkHandler
};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let lavalink_url = env::var("LAVALINK_URL")
        .expect("Expected the lavalink url in the environment");
    let lavalink_password = env::var("LAVALINK_PASSWORD")
        .expect("Expected the lavalink password in the environment");
    
    let prefix = env::var("BOT_PREFIX").ok();
    let prefix = prefix
        .as_ref()
        .map(String::as_str)        
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .unwrap_or("~");

    let http = Http::new_with_token(&token);

    let bot_id = match http.get_current_application_info().await {
        Ok(info) => info.id,
        Err(why) => panic!("Could not access application info: {:?}", why),
    };
    

    let framework = StandardFramework::new()
        .configure(|c| c
                   .prefix(prefix))
        .group(&MUSIC_GROUP)
        .group(&FUN_GROUP);

    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
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
