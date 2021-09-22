use futures::future::{AbortHandle, Abortable};
use serenity::{
    client::Context,
    framework::standard::{CommandResult},
    model::{
        channel::Message,
        guild::Guild,
        id::{ChannelId, GuildId, UserId},
    },
};
use std::time::Duration;
use tokio::time::sleep;

use crate::{BotId, Lavalink, VoiceTimerMap};

pub async fn get_voice_state(
    ctx: &Context,
    guild: &Guild,
    author_id: UserId,
) -> CommandResult<bool> {
    let bot_id = ctx.data.read().await.get::<BotId>().cloned().unwrap();

    if !(guild.voice_states.contains_key(&author_id) || guild.voice_states.contains_key(&bot_id)) {
        return Ok(false);
    }

    let user_voice_id = guild
        .voice_states
        .get(&author_id)
        .and_then(|state| state.channel_id);
    let bot_voice_id = guild
        .voice_states
        .get(&bot_id)
        .and_then(|state| state.channel_id);

    if user_voice_id == bot_voice_id {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn join_voice_internal(
    ctx: &Context,
    msg: &Message,
    voice_channel: ChannelId,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let (_, handler) = manager.join_gateway(guild_id, voice_channel).await;

    match handler {
        Ok(conn_info) => {
            let lava_client = ctx.data.read().await.get::<Lavalink>().cloned().unwrap();
            lava_client.create_session_with_songbird(&conn_info).await?;
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

pub async fn leavevc_internal(ctx: &Context, guild_id: GuildId) -> CommandResult {
    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await?;

        let lava_client = ctx.data.read().await.get::<Lavalink>().cloned().unwrap();
        lava_client.destroy(guild_id).await?;

        {
            let nodes = lava_client.nodes().await;
            nodes.remove(&guild_id.0);

            let loops = lava_client.loops().await;
            loops.remove(&guild_id.0);
        }
    } else {
        return Err("The bot isn't in a voice channel!".into());
    }

    Ok(())
}

pub async fn create_new_timer(ctx: Context, guild_id: GuildId) {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(leavevc_internal(&ctx, guild_id), abort_registration);

    let voice_timer_map = ctx
        .data
        .read()
        .await
        .get::<VoiceTimerMap>()
        .cloned()
        .unwrap();

    voice_timer_map.insert(guild_id, abort_handle);

    sleep(Duration::from_secs(300)).await;
    let _ = future.await;

    voice_timer_map.remove(&guild_id);
}
