
mod music;
mod fun;
mod setup;

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

#[group]
#[description = "Music commands"]
#[commands(join, leave, ping, play, now, skip)]
pub(crate) struct Music;

#[group]
#[description = "Fun commands"]
#[commands(rojao, huehue, qrcode)]
pub(crate) struct Fun;

#[group]
#[description = "Config commands"]
#[commands(set_channel_only_link)]
pub(crate) struct Config;
