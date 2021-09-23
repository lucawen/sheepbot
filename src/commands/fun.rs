use tokio::time::{sleep, Duration};
use std::{
    io::Cursor,
    path::Path,
    fs::{remove_file, File},
    sync::{Arc, Weak}
};

use serenity::{client::Context};

use serenity::{
    async_trait,
    framework::{
        standard::{
            Args, CommandResult,
            macros::{command},
        },
    },
    model::{channel::Message},
    http::AttachmentType,
    prelude::Mutex,
};
use tts_urls::google_translate;

use rand::Rng; 
use rand::distributions::Alphanumeric;
use qrcode::QrCode;
use image::Luma;
use crate::{
    utils::message::{check_msg},
    structures::cmd_data::{ReqwestClient, Lavalink},
};
use songbird::{
    Event,
    TrackEvent,
    EventHandler as VoiceEventHandler,
    EventContext,
    input::{
        self
    },
    Call,
};


#[command]
#[description = "fireworks like effect as text messages"]
pub(self) async fn rojao(ctx: &Context, msg: &Message) -> CommandResult {
    let num_lines: u8 = rand::thread_rng().gen_range(1..6);
    let ms_wait: u32 = rand::thread_rng().gen_range(1000..5000);
    for _ in 0..=num_lines {
        let num_cols: u16 = rand::thread_rng().gen_range(1..10);
        let mut arr_its: Vec<String> = vec![Default::default(); num_cols.into()];
        for _ in 0..=num_cols {
            arr_its.push("pra".to_owned());
        }
        let text = arr_its.join(" ");
        check_msg(msg.channel_id.say(&ctx.http, text).await);
        let ms_wait_next: u32 = rand::thread_rng().gen_range(100..1200);
        sleep(Duration::from_millis(ms_wait_next.into())).await;
    }
    sleep(Duration::from_millis(ms_wait.into())).await;
    check_msg(msg.channel_id.say(&ctx.http, "POOOOOWW").await);
    Ok(())
}

#[command]
#[description = "send random brazilliam 'huehue' meme"]
pub(self) async fn huehue(ctx: &Context, msg: &Message) -> CommandResult {
    let avaiable_gifs = [
        "https://tenor.com/bn4pN.gif",
        "https://tenor.com/bmfGc.gif",
        "https://tenor.com/QITm.gif",
        "https://tenor.com/05He.gif",
        "https://tenor.com/8VkU.gif",
    ];
    let choose = avaiable_gifs[rand::thread_rng().gen_range(0..=avaiable_gifs.len())];
    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.content(choose);
        m
    }).await;
    Ok(())
}

#[command]
#[min_args(1)]
#[description = "qrcode <URL or any text> : convert anything into qrcode image"]
pub(self) async fn qrcode(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let msg_received = args.message().to_string();

    let code = QrCode::new(msg_received).unwrap();
    let image = code.render::<Luma<u8>>().build();

    // Save the image.
    let random_fname: String  = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

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

#[command]
#[min_args(1)]
#[description = "play <text> : Create a tts message and play in voice"]
pub(self) async fn tts(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let (reqwest_client, lavalink_client) = {
        let data = ctx.data.read().await;
        let reqwest_client = data.get::<ReqwestClient>().cloned().unwrap();
        let lavalink_client = data.get::<Lavalink>().cloned().unwrap();
        (reqwest_client, lavalink_client)
    };

    if !!!lavalink_client.nodes().await.get(&msg.guild_id.unwrap().0).is_none()  {
        check_msg(msg.reply(ctx, "Already connected to a channel.").await);
        return Ok(());
    }

    let msg_received = args.message().to_string();
    let tts_url = google_translate::url(&msg_received, "pt-br");

    let response = reqwest_client
        .get(tts_url)
        .send()
        .await?;

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.color(0x98fb98);
                e.title("Sheep are saying");
                e.field("Message: ", msg_received, true);
                e.footer(|f| {
                    f.text(format!("Requested by {}", msg.author.name));
                    f
                })
            })
        })
        .await?;

    // Save the image.
    let random_fname: String  = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    let filepath = format!("./{}.mp3", random_fname);
    let path = Path::new(&filepath);

    let mut file = File::create(&path).unwrap();
    let content_byte = response.bytes().await?;
    let mut content =  Cursor::new(content_byte);
    std::io::copy(&mut content, &mut file)?;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let (handler_lock, success_reader) = manager.join(guild_id, connect_to).await;
    let call_lock_for_evt = Arc::downgrade(&handler_lock);

    if let Ok(_reader) = success_reader {
        let mut handler = handler_lock.lock().await;
        let ting_src = input::ffmpeg(&filepath).await.expect("Cant get tts file");
        let song = handler.play_source(ting_src);
        let _ = song.add_event(
            Event::Track(TrackEvent::End),
            EndPlaySound {
                call_lock: call_lock_for_evt,
                filename: filepath,
            },
        );
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel").await);
    }

    Ok(())
}

struct EndPlaySound {
    call_lock: Weak<Mutex<Call>>,
    filename: String,
}

#[async_trait]
impl VoiceEventHandler for EndPlaySound {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        if let Some(call_lock) = self.call_lock.upgrade() {
            let mut handler = call_lock.lock().await;
            handler.leave().await.unwrap();
            remove_file(&self.filename)
                .expect("Cant remove tts file file");
        }
        None
    }
}