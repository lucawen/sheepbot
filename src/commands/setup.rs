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
use url::{Url, ParseError};
use crate::utils::setup::{get_link_only_modes};



#[command]
#[min_args(1)]
#[aliases(set_col)]
pub async fn set_channel_only_link(ctx: &Context, msg: &Message, args: Args) -> CommandResult {

    if let Some(guild) = msg.guild(&ctx.cache).await {
        let guild_id: i64 = i64::from(guild.id);
        let channel_id = msg.channel_id.0 as i64;
        let user_id = i64::from(msg.author.id);
        
        let msg_received = args.message();
        let url_obj = Url::parse(msg_received);
        let domain_str = match url_obj {
            Ok(ref url_obj) => url_obj.host_str().clone(),
            Err(erro) => {
                if erro == ParseError::RelativeUrlWithoutBase {
                    Some(msg_received)
                } else {
                    check_msg(msg.reply(ctx, "Cant parse this url").await);
                    None
                }
            },
        };

        let domain = match domain_str {
            Some(x) => x.replace("www.", ""),
            None    => {
                check_msg(msg.reply(ctx, "Cant parse this url").await);
                return Ok(())
            },
        };

        let data_read = ctx.data.read().await;
        let pool = data_read.get::<ConnectionPool>().unwrap();
        

        if let Ok(_) = sqlx::query!(
            "SELECT channel_id FROM only_link_channel
            WHERE guild_id = $1
            AND channel_id = $2
            AND url = $3
            LIMIT 1",
            guild_id, channel_id, domain
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
                guild_id as i64, channel_id, user_id, domain
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
pub async fn unset_channel_only_link(ctx: &Context, msg: &Message) -> CommandResult {

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


#[command]
#[aliases(ls_col)]
pub async fn list_channel_only_link(ctx: &Context, msg: &Message) -> CommandResult {

    let data_read = ctx.data.read().await;
    let pool = data_read.get::<ConnectionPool>().unwrap();

    if let Some(guild_id) = msg.guild_id {
        let rules = get_link_only_modes(
            pool, guild_id, msg.channel_id).await;
        match rules {
            Ok(res) => {
                let string_rules_arr = res.iter().map(|x| x.url.clone()).collect::<Vec<_>>();
                if string_rules_arr.is_empty() {
                    check_msg(msg.reply(ctx, "No urls set yet.").await);
                } else {
                    let rules_str = string_rules_arr.join(", ");
                    check_msg(msg.reply(ctx, format!("Urls enabled: {}", rules_str)).await);
                }
            },
            Err(err) => {
                println!(
                    "Error when trying to get links only: {:?}",
                    err,
                );
            },
        }
    }

    Ok(())
}