use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::{
        channel::Message,
    },
};

use crate::{
    helpers::voice_utils::{
        create_new_timer, join_voice_internal, leavevc_internal, get_voice_state
    },
    BotId,
    VoiceTimerMap
};

#[command]
#[aliases("dc")]
#[description = "disconnect: Leaves the voice chat and clears everything"]
async fn disconnect(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = ctx
        .cache
        .guild_channel_field(msg.channel_id, |channel| channel.guild_id)
        .await
        .unwrap();
    let guild = msg.guild(ctx).await.unwrap();

    if !get_voice_state(ctx, &guild, msg.author.id).await? {
        msg.channel_id
            .say(
                ctx,
                "Please be in a voice channel or in the same voice channel as me!",
            )
            .await?;
        return Ok(());
    }

    match leavevc_internal(ctx, guild_id).await {
        Ok(_) => {
            let voice_timer_map = ctx
                .data
                .read()
                .await
                .get::<VoiceTimerMap>()
                .cloned()
                .unwrap();

            if voice_timer_map.contains_key(&guild_id) {
                if let Some(future_guard) = voice_timer_map.get(&guild_id) {
                    future_guard.value().abort();
                }
                voice_timer_map.remove(&guild_id);
            }

            msg.channel_id.say(ctx, "Left the voice channel!").await?;
        }
        Err(_e) => {
            msg.channel_id
                .say(ctx, "The bot isn't in a voice channel!")
                .await?;
        }
    }

    Ok(())
}


#[command]
#[aliases("connect")]
#[description = "summon: Forces the bot to join the voice chat"]
pub async fn summon(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).await.unwrap();
    let bot_id = ctx.data.read().await.get::<BotId>().cloned().unwrap();

    if guild.voice_states.contains_key(&bot_id) {
        msg.channel_id
            .say(ctx, "Looks like I'm already in a voice channel! Please disconnect me before summoning me again!")
            .await?;

        return Ok(());
    }

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let voice_channel = match channel_id {
        Some(channel) => channel,
        None => {
            msg.channel_id
                .say(ctx, "Please join a voice channel!")
                .await?;

            return Ok(());
        }
    };

    match join_voice_internal(ctx, msg, voice_channel).await {
        Ok(_) => {
            msg.channel_id
                .say(
                    ctx,
                    format!("Joined {}", voice_channel.name(ctx).await.unwrap()),
                )
                .await?;

            let ctx_clone = ctx.clone();
            tokio::spawn(async move {
                create_new_timer(ctx_clone, guild.id).await;
            });
        }
        Err(_e) => {
            msg.channel_id
                .say(ctx, "I couldn't join the voice channel. Please check if I have permission to access it!")
                .await?;
        }
    }

    Ok(())
}