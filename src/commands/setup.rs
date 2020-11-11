use serenity::{client::Context};

use serenity::{
    framework::{
        standard::{
            Args, CommandResult,
            macros::{command},
        },
    },
    model::{channel::Message}
};
use crate::utils::message::{check_msg};

use crate::handler::{ConnectionPool};


#[command]
#[min_args(1)]
#[aliases(set_col)]
pub async fn set_channel_only_link(ctx: &Context, msg: &Message, args: Args) -> CommandResult {

    if let Some(guild) = msg.guild(&ctx.cache).await {
        let guild_id: i64 = i64::from(guild.id);
        let channel_id = msg.channel_id.0 as i64;
        let user_id = i64::from(msg.author.id);
        
        let msg_received = args.message().to_string();

        let data_read = ctx.data.read().await;
        let pool = data_read.get::<ConnectionPool>().unwrap();
        

        if let Ok(_) = sqlx::query!(
            "SELECT channel_id FROM only_link_channel
            WHERE guild_id = $1
            AND channel_id = $2
            AND url = $3
            LIMIT 1",
            guild_id, channel_id, msg_received
        )
        .fetch_one(pool)
        .await
        {
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.content("Channel already have Only Link Mode enabled with this url");
                m
            }).await;
        } else {
            if let Err(why) = sqlx::query!(
                r#"
                INSERT INTO only_link_channel (
                    guild_id, channel_id, user_id, url
                ) VALUES (
                    $1, $2, $3, $4
                );
                "#,
                guild_id as i64, channel_id, user_id, msg_received
            )
            .execute(pool)
            .await
            {
                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.content("Cant add Only Link Mode to this channel");
                    m
                }).await;
                println!("error: {}", why)
            } else {
                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.content("Only Link Mode enabled to this channel");
                    m
                }).await;
            }
        }
    } else {
        check_msg(msg.reply(ctx, "Error ocurred when try to run this command").await);
    };
    Ok(())
}

#[command]
#[aliases(unset_col)]
pub async fn unset_channel_only_link(ctx: &Context, msg: &Message, args: Args) -> CommandResult {

    if let Some(guild) = msg.guild(&ctx.cache).await {
        let guild_id: i64 = i64::from(guild.id);
        let channel_id = msg.channel_id.0 as i64;
        
        let data_read = ctx.data.read().await;
        let pool = data_read.get::<ConnectionPool>().unwrap();

        if let Ok(_) = sqlx::query!(
            "SELECT channel_id FROM only_link_channel WHERE guild_id = $1 and channel_id = $2 LIMIT 1",
            guild_id, channel_id
        )
        .fetch_one(pool)
        .await
        {
            if let Err(why) = sqlx::query!(
                r#"
                DELETE FROM only_link_channel
                WHERE guild_id = $1
                AND channel_id = $2
                "#,
                guild_id as i64, channel_id
            )
            .execute(pool)
            .await
            {
                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.content("Cant remove Only Link Mode from this channel");
                    m
                }).await;
                println!("error: {}", why)
            } else {
                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.content("Only Link Mode removed from this channel");
                    m
                }).await;
            }
        } else {
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.content("Only Link Mode is not set to this channel");
                m
            }).await;
        }
    } else {
        check_msg(msg.reply(ctx, "Error ocurred when try to run this command").await);
    };
    Ok(())
}