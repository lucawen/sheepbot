use serenity::{client::Context};

use serenity::{
    framework::{
        standard::{
            Args, CommandResult,
            macros::{command},
        },
    },
    model::{channel::Message}
};

use crate::{
    SeaDBConnection
};
use chrono::Utc;
use sea_orm::{entity::*, query::*};
use crate::models::{
    tracker, TrackerEntity
};

#[command]
#[min_args(1)]
#[description = "pkg_add <tracking code> : Add a package tracking code to be tracked"]
pub(self) async fn pkg_add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let msg_received = args.message().to_string();
    let data = ctx.data.read().await;
    let db = data.get::<SeaDBConnection>().unwrap();
    let user_id = msg.author.id.0 as i64;
    let dt = Utc::now();
    let timestamp = dt.naive_utc();
    let timestamp2 = dt.naive_utc();
    let status = String::from("new");

    let one_entity = TrackerEntity::find()
        .filter(tracker::Column::Code.eq(msg_received.clone()))
        .filter(tracker::Column::UserId.eq(user_id.clone()))
        .one(db)
        .await?;

    match one_entity {
        Some(_) => {
            let _ = msg
                .channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.color(0xff69b4);
                        e.title("Package Tracker");
                        e.description("You already have this tracking code");
                        e.field("Code", &msg_received, false);
                        e
                    })
                })
                .await;
        },
        _ => {
            let pear = tracker::ActiveModel {
                user_id: Set(user_id.to_owned()),
                created_at: Set(timestamp.to_owned()),
                updated_at: Set(timestamp2.to_owned()),
                code: Set(msg_received.to_owned()),
                status: Set(status.to_owned()),
                ..Default::default()
            };
            TrackerEntity::insert(pear).exec(db).await?;
            let _ = msg
                .channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.color(0x2FF354);
                        e.title("Package Tracker");
                        e.description("Tracking code registered");
                        e.field("Code", &msg_received, false);
                        e.field("Details", "You will receive notifications about moviment with your package", false);
                        e
                    })
                })
                .await;
        }
    };

    Ok(())
}

#[command]
#[description = "pkg_list : List all your packages"]
pub(self) async fn pkg_list(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<SeaDBConnection>().unwrap();
    let user_id = msg.author.id.0 as i64;

    let all_packages: Vec<tracker::Model> = TrackerEntity::find()
        .filter(tracker::Column::UserId.eq(user_id.clone()))
        .all(db)
        .await?;
    
    let vec_formated = all_packages.iter().map(|x| {
        (&x.code, format!("Status {}\n Last Update {} UTC", x.status, x.updated_at.format("%d-%m-%Y %H:%M:%S")), true)
    }).collect::<Vec<_>>();

    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.color(0x0A2AF4);
                e.title("Package Tracker");
                e.description("List all your tracking codes");
                e.fields(vec_formated);
                e
            })
        })
        .await;

    Ok(())
}

#[command]
#[description = "pkg_rm <tracking code> : Remove the monitoring from a tracking code"]
pub(self) async fn pkg_rm(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let msg_received = args.message().to_string();
    let data = ctx.data.read().await;
    let db = data.get::<SeaDBConnection>().unwrap();
    let user_id = msg.author.id.0 as i64;

    TrackerEntity::delete_many()
        .filter(tracker::Column::UserId.eq(user_id.clone()))
        .filter(tracker::Column::Code.eq(msg_received.clone()))
        .exec(db)
        .await?;

    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.color(0xF40A2A);
                e.title("Package Tracker");
                e.description(format!("Tracking code {} removed", &msg_received));
                e
            })
        })
        .await;

    Ok(())
}
