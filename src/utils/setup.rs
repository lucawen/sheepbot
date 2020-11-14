use sqlx::{Pool, Postgres};
use anyhow::Result;

use crate::models::{OnlyLinkChannel};
use serenity::model::id::{GuildId, ChannelId};

pub async fn get_link_only_modes(pool: &Pool<Postgres>, guild_id: GuildId, channel_id: ChannelId) -> Result<Vec<OnlyLinkChannel>> {
    let rows = sqlx::query_as::<_, OnlyLinkChannel>(
        "SELECT * FROM only_link_channel
            WHERE guild_id = $1
            AND channel_id = $2
            "
        )
        .bind(i64::from(guild_id))
        .bind(i64::from(channel_id))
        .fetch_all(pool)
        .await?;

    Ok(rows)
}
