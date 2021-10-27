use dashmap::DashMap;
use sqlx::postgres::{PgPool, PgPoolOptions};
use serenity::{model::id::GuildId};
use crate::models::{
    core::BotInfo
};
use sqlx::{Pool, Postgres};

pub async fn obtain_db_pool(db_connection: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let connection_string = &db_connection;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    Ok(pool)
}

pub async fn fetch_prefixes(pool: &Pool<Postgres>) -> Result<DashMap<GuildId, String>, Box<dyn std::error::Error>>{
    let prefixes: DashMap<GuildId, String> = DashMap::new();

    let cursor = sqlx::query_as::<_, BotInfo>(
        "SELECT guild_id, prefix FROM guild_info"
        )
        .fetch_all(pool)
        .await?;

    for i in cursor {
        if !!!i.prefix.is_empty() {
            prefixes.insert(GuildId::from(i.guild_id as u64), i.prefix);
        }
    }

    Ok(prefixes)
}