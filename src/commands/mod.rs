
mod music;
mod fun;
mod setup;
mod voice;
mod tracker;

use serenity::{
    framework::{
        standard::{
            macros::{group},
        },
    }
};

use music::*;
use fun::*;
use setup::*;
use voice::*;
use tracker::*;

#[group]
#[description = "Music commands"]
#[commands(play, pause, stop, resume, queue, clear, remove, skip, seek)]
pub(crate) struct Music;

#[group]
#[description = "Fun commands"]
#[commands(rojao, huehue, qrcode, tts)]
pub(crate) struct Fun;

#[group]
#[description = "Config commands"]
#[commands(set_channel_only_link, unset_channel_only_link, list_channel_only_link)]
pub(crate) struct Config;

#[group("Voice")]
#[description = "Commands used for voice chat"]
#[commands(summon, disconnect)]
pub struct Voice;

#[group("Tracker")]
#[description = "Package tracker on 'Correios'"]
#[commands(pkg_add, pkg_list, pkg_rm)]
#[only_in(dm)]
pub struct Tracker;