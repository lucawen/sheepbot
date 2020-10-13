
mod music;
mod fun;

use serenity::{
    framework::{
        standard::{
            macros::{group},
        },
    }
};

use music::*;
use fun::*;

#[group]
#[description = "Music commands"]
#[commands(join, leave, ping, play, now, skip)]
pub(crate) struct Music;

#[group]
#[description = "Fun commands"]
#[commands(rojao, huehue, qrcode)]
pub(crate) struct Fun;
