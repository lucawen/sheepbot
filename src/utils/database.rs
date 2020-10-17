use tracing::{error};

use crate::handler::{ConnectionPool};

use serenity::{model::prelude::Guild, prelude::Context};
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn obtain_pool(pg_url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&pg_url)
        .await?;

    Ok(pool)
}

pub async fn initialize_tables(ctx: &Context, guild: &Guild) {
    let data_read = ctx.data.read().await;
    let pool = data_read.get::<ConnectionPool>().unwrap();

    // config table initialization
    if let Err(why) = sqlx::query!(
        r#"
        INSERT INTO config (
            guild_id
        ) VALUES (
            $1
        ) ON CONFLICT (guild_id) DO NOTHING;
        "#,
        guild.id.0 as i64
    )
    .execute(pool)
    .await
    {
        error!(
            "Error initializing config table for guild with ID `{}`: {}",
            guild.id.0, why
        );
    }

    // prefixes table initialization
    if let Err(why) = sqlx::query!(
        "
        INSERT INTO prefixes (
            guild_id, prefix
        ) VALUES (
            $1, $2
        ) ON CONFLICT (guild_id) DO NOTHING;
        ",
        guild.id.0 as i64, "OK"
    )
    .execute(pool)
    .await
    {
        error!(
            "Error initializing prefixes table for guild with ID `{}`: {}",
            guild.id.0, why
        );
    }
}