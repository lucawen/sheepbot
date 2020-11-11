use sqlx::{Pool, Postgres};

use crate::models::{OnlyLinkChannel};
use std::collections::HashMap;


pub async fn get_link_only_modes(pool: &Pool<Postgres>, guild_id: i64, channel_id: i64) -> Result<()> {
    let mut rows = sqlx::query(
        "SELECT * FROM only_link_channel
            WHERE guild_id = ?
            AND channel_id = ?
            "
        )
        .bind(guild_id)
        .bind(channel_id)
        .fetch(pool)

    let mut book_reviews: HashMap<i64, OnlyLinkChannel> = HashMap::new();

    while let Some(row) = rows.try_next().await? {
        // map the row into a user-defined domain type
        let email: &str = row.try_get("email")?;
    }

    Ok(())
}