use crate::ConnectionPool;
use rand::{prelude::StdRng, Rng, SeedableRng};
use serenity::{
    client::bridge::gateway::ShardMessenger,
    framework::standard::CommandResult,
    model::{id::GuildId, prelude::Activity},
    prelude::*,
};
use std::time::{Duration};
use tokio::time::sleep;
use crate::models::{
    BotInfo
};

pub async fn guild_pruner(ctx: &Context) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let guilds = ctx.cache.guilds().await;

    let guild_data = sqlx::query_as::<_, BotInfo>(
        "SELECT guild_id, prefix FROM guild_info"
        )
        .fetch_all(&pool)
        .await?;


    println!(" ");

    for guild in guild_data {
        if !guilds.contains(&GuildId::from(guild.guild_id as u64)) {
            println!("Removing guild: {}", guild.guild_id);

            sqlx::query("DELETE FROM guild_info WHERE guild_id = ?")
                .bind(guild.guild_id)
                .execute(&pool)
                .await?;
        }
    }

    println!(" ");

    Ok(())
}

pub async fn activity_loop(messenger: &ShardMessenger) {
    let activity_vec = vec![
        Activity::playing("as the fool"),
        Activity::listening("a tune!"),
        Activity::listening("straight vibes"),
        Activity::playing("vibe checks"),
        Activity::playing("hacking the mainframe"),
        Activity::playing("boof simulator 2021"),
        Activity::playing("ZA WARUDO!"),
        Activity::listening("lofi music"),
        Activity::playing("Minecraft"),
        Activity::listening("Bhai tunes"),
        Activity::playing("Purging scalpers"),
        Activity::listening("the rustdoc audiobook"),
    ];

    let mut rng = StdRng::from_entropy();

    loop {
        let val = rng.gen_range(0..=activity_vec.len() - 1);

        messenger.set_activity(Some(activity_vec[val].to_owned()));

        sleep(Duration::from_secs(7200)).await;
    }
}