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
pub async fn set_channel_only_link(ctx: &Context, msg: &Message, args: Args) -> CommandResult {

    if let Some(guild) = msg.guild(&ctx.cache).await {
        let guild_id: i64 = i64::from(guild.id);
        let channel_id = msg.channel_id.0 as i64;
        let user_id = i64::from(msg.author.id);
        
        let msg_received = args.message().to_string();

        let data_read = ctx.data.read().await;
        let pool = data_read.get::<ConnectionPool>().unwrap();

        if let Err(why) = sqlx::query!(
            r#"
            INSERT INTO only_link_channel (
                guild_id, channel_id, user_id, url
            ) VALUES (
                $1, $2, $3, $4
            ) ON CONFLICT (channel_id) DO NOTHING;
            "#,
            guild_id as i64, channel_id, user_id, msg_received
        )
        .execute(pool)
        .await
        {
            check_msg(msg.reply(ctx, "Cant add this channel as link only").await);
            println!("error: {}", why)
        } else {
            check_msg(msg.reply(ctx, "Channel set as link only").await);
        }
    } else {
        check_msg(msg.reply(ctx, "Error ocurred when try to run this command").await);
    };
    Ok(())
}