use aspotify::Client as Spotify;
use lavalink_rs::LavalinkClient;
use serenity::{
    prelude::{TypeMapKey},
    model::id::{GuildId, UserId},
};
use std::{
    sync::Arc
};
use dashmap::DashMap;
use futures::future::AbortHandle;
use sqlx::PgPool;
use crate::settings::Settings;

pub struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

pub struct SpotifyClient;

impl TypeMapKey for SpotifyClient {
    type Value = Arc<Spotify>;
}

pub struct VoiceTimerMap;

impl TypeMapKey for VoiceTimerMap {
    type Value = Arc<DashMap<GuildId, AbortHandle>>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}

pub struct BotId;

impl TypeMapKey for BotId {
    type Value = UserId;
}

pub struct SettingsConf;

impl TypeMapKey for SettingsConf {
    type Value = Settings;
}

pub struct PrefixMap;

impl TypeMapKey for PrefixMap {
    type Value = Arc<DashMap<GuildId, String>>;
}
