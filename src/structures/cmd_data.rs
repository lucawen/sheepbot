use aspotify::Client as Spotify;
use lavalink_rs::LavalinkClient;
use serenity::{
    prelude::{TypeMapKey},
    model::id::{GuildId, UserId},
};
use std::{
    convert::TryInto,
    sync::Arc
};
use dashmap::DashMap;
use futures::future::AbortHandle;
use sqlx::PgPool;
use crate::settings::Settings;
use reqwest::Client as Reqwest;
use songbird::{
    input::{
        cached::{Compressed, Memory},
        Input,
    },
};
use sea_orm::{
    DatabaseConnection
};

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

pub struct SeaDBConnection;

impl TypeMapKey for SeaDBConnection {
    type Value = DatabaseConnection;
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

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
    type Value = Reqwest;
}

pub enum CachedSound {
    Compressed(Compressed),
    Uncompressed(Memory),
}

impl From<&CachedSound> for Input {
    fn from(obj: &CachedSound) -> Self {
        use CachedSound::*;
        match obj {
            Compressed(c) => c.new_handle()
                .into(),
            Uncompressed(u) => u.new_handle()
                .try_into()
                .expect("Failed to create decoder for Memory source."),
        }
    }
}