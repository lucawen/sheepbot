
use std::{
    sync::Arc,
    time::Duration,
};

use serenity::{client::Context};

use serenity::{
    framework::{
        standard::{
            Args, CommandResult,
            macros::{command},
        },
    },
    model::{channel::Message, misc::Mentionable},
};

use lavalink_rs::{
    LavalinkClient
};


use crate::utils::message::{check_msg};
use crate::handler::{Lavalink, VoiceManager, VoiceGuildUpdate};


#[command]
pub(self) async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();

    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;
    let has_joined = manager.join(guild_id, connect_to).is_some();

    if has_joined {
        drop(manager);

        loop {
            let data = ctx.data.read().await;
            let vgu_lock = data.get::<VoiceGuildUpdate>().unwrap();
            let mut vgu = vgu_lock.write().await;
            if !vgu.contains(&guild_id) {
                tokio::time::delay_for(Duration::from_millis(500)).await;
            } else {
                vgu.remove(&guild_id);
                break;
            }
        }

        let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
        let manager = manager_lock.lock().await;

        let mut data = ctx.data.write().await;
        let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let handler = manager.get(guild_id).unwrap();
        lava_client_lock.lock().await.create_session(guild_id, &handler).await?;

        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel").await);
    }

    Ok(())
}

#[command]
pub(self) async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = ctx.cache.guild_channel_field(msg.channel_id, |channel| channel.guild_id).await.unwrap();

    let manager_lock = ctx.data.read().await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);

        let mut data = ctx.data.write().await;
        let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        lava_client_lock.lock().await.destroy(guild_id).await?;

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
pub(self) async fn ping(context: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&context.http, "Pong!").await);

    Ok(())
}

#[command]
#[min_args(1)]
pub(self) async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info").await);

            return Ok(());
        },
    };

    let manager_lock = ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in TypeMap.");
    let mut manager = manager_lock.lock().await;

    if let Some(_handler) = manager.get_mut(guild_id) {
        let mut data = ctx.data.write().await;
        let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
        let lava_client = lava_client_lock.lock().await;

        let query_information = lava_client.auto_search_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            check_msg(msg.channel_id.say(&ctx, "Could not find any video of the search query.").await);
            return Ok(());
        }

        drop(lava_client);

        if let Err(why) = LavalinkClient::play(guild_id, query_information.tracks[0].clone()).queue(Arc::clone(lava_client_lock)).await {
            eprintln!("{}", why);
            return Ok(());
        };
        check_msg(msg.channel_id.say(&ctx.http, format!("Added to queue: {}", query_information.tracks[0].info.as_ref().unwrap().title)).await);

    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Use `~join` first, to connect the bot to your current voice channel.").await);
    }

    Ok(())
}

#[command]
#[aliases(np)]
pub(self) async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");
    let lava_client = lava_client_lock.lock().await;

    if let Some(node) = lava_client.nodes.get(&msg.guild_id.unwrap().0) {
        if let Some(track) = &node.now_playing {
            check_msg(msg.channel_id.say(&ctx.http, format!("Now Playing: {}", track.track.info.as_ref().unwrap().title)).await);
        } else {
            check_msg(msg.channel_id.say(&ctx.http, "Nothing is playing at the moment.").await);
        }
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Nothing is playing at the moment.").await);
    }

    Ok(())
}

#[command]
pub(self) async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let lava_client_lock = data.get_mut::<Lavalink>().expect("Expected a lavalink client in TypeMap");

    if let Some(track) = lava_client_lock.lock().await.skip(msg.guild_id.unwrap()).await {
        check_msg(msg.channel_id.say(ctx, format!("Skipped: {}", track.track.info.as_ref().unwrap().title)).await);
    } else {
        check_msg(msg.channel_id.say(ctx, "Nothing to skip.").await);
    }

    Ok(())
}

