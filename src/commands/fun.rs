use tokio::time::{sleep, Duration};
use std::{path::Path, fs::remove_file};

use serenity::{client::Context};

use serenity::{
    framework::{
        standard::{
            Args, CommandResult,
            macros::{command},
        },
    },
    model::{channel::Message},
    http::AttachmentType
};
use crate::utils::message::{check_msg};

use rand::Rng; 
use rand::distributions::Alphanumeric;
use qrcode::QrCode;
use image::Luma;


#[command]
pub(self) async fn rojao(ctx: &Context, msg: &Message) -> CommandResult {

    let num_lines: u8 = rand::thread_rng().gen_range(1, 6);
    let ms_wait: u32 = rand::thread_rng().gen_range(1000, 5000);
    for _ in 0..=num_lines {
        let num_cols: u16 = rand::thread_rng().gen_range(1, 10);
        let mut arr_its: Vec<String> = vec![Default::default(); num_cols.into()];
        for _ in 0..=num_cols {
            arr_its.push("pra".to_owned());
        }
        let text = arr_its.join(" ");
        check_msg(msg.channel_id.say(&ctx.http, text).await);
        let ms_wait_next: u32 = rand::thread_rng().gen_range(100, 1200);
        sleep(Duration::from_millis(ms_wait_next.into())).await;
    }
    sleep(Duration::from_millis(ms_wait.into())).await;
    check_msg(msg.channel_id.say(&ctx.http, "POOOOOWW").await);
    Ok(())
}

#[command]
pub(self) async fn huehue(ctx: &Context, msg: &Message) -> CommandResult {
    let avaiable_gifs = [
        "https://tenor.com/bn4pN.gif",
        "https://tenor.com/bmfGc.gif",
        "https://tenor.com/QITm.gif",
        "https://tenor.com/05He.gif",
        "https://tenor.com/8VkU.gif",
    ];
    let choose = avaiable_gifs[rand::thread_rng().gen_range(0, avaiable_gifs.len())];
    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.content(choose);
        m
    }).await;
    Ok(())
}

#[command]
#[min_args(1)]
pub(self) async fn qrcode(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let msg_received = args.message().to_string();

    let code = QrCode::new(msg_received).unwrap();
    let image = code.render::<Luma<u8>>().build();

    // Save the image.
    let random_fname = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect::<String>();
    let filepath = format!("/tmp/{}.png", random_fname);
    image.save(&filepath).unwrap();

    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.add_file(AttachmentType::Path(Path::new(&filepath)));
        m
    }).await;

    remove_file(&filepath)
        .expect("Cant remove qrcode file");

    Ok(())
}