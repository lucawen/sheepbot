use dashmap::DashMap;
use sqlx::postgres::{PgPool, PgPoolOptions};
use serenity::{model::id::GuildId};

pub async fn obtain_db_pool(db_connection: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let connection_string = &db_connection;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    Ok(pool)
}

pub async fn fetch_prefixes(pool: &PgPool) -> Result<DashMap<GuildId, String>, Box<dyn std::error::Error>>{
    let prefixes: DashMap<GuildId, String> = DashMap::new();

    let cursor = sqlx::query!("SELECT guild_id, prefix FROM guild_info")
        .fetch_all(pool)
        .await?;

    for i in cursor {
        if let Some(prefix) = i.prefix {
            prefixes.insert(GuildId::from(i.guild_id as u64), prefix);
        }
    }

    Ok(prefixes)
}