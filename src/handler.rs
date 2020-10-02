use std::{
    sync::Arc,
    collections::HashSet,
};

use serenity::client::bridge::voice::ClientVoiceManager;

use serenity::{
    client::Context,
    prelude::Mutex,
};

use serenity::{
    async_trait,
    client::{
        EventHandler
    },
    framework::{
        standard::{
            CommandResult,
            macros::{
                hook,
            },
        },
    },
    model::{
        channel::Message,
        gateway::Ready,
        id::GuildId,
        event::VoiceServerUpdateEvent,
    },
};

use serenity::prelude::*;
use lavalink_rs::{
    LavalinkClient,
    model::*,
    gateway::*,
};


pub(crate) struct VoiceManager;
pub(crate) struct Lavalink;
pub(crate) struct VoiceGuildUpdate;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

impl TypeMapKey for Lavalink {
    type Value = Arc<Mutex<LavalinkClient>>;
}

impl TypeMapKey for VoiceGuildUpdate {
    type Value = Arc<RwLock<HashSet<GuildId>>>;
}


pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name,
                shard[0],
                shard[1],
            );
        }
    }

    async fn voice_server_update(&self, ctx: Context, voice: VoiceServerUpdateEvent) {
        if let Some(guild_id) = voice.guild_id {
            let data = ctx.data.read().await;
            let voice_server_lock = data.get::<VoiceGuildUpdate>().unwrap();
            let mut voice_server = voice_server_lock.write().await;
            voice_server.insert(guild_id);
        }
    }

}

pub(crate) struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackStart) {
        println!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackFinish) {
        println!("Track finished!\nGuild: {}", event.guild_id);
    }
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Err(why) => println!("Command '{}' returned error {:?} => {}", command_name, why, why),
        _ => (),
    }
}

