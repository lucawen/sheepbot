#[derive(sqlx::FromRow)]
pub struct OnlyLinkChannel {
    pub guild_id: i64,
    pub id: i32,
    pub channel_id: i64,
    pub user_id: i64,
    pub url: String
}