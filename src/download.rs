#![allow(unused)]

use rand::prelude::*;
use serenity::all::Attachment;
use serenity::all::CreateAttachment;
use rand::rng;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use anyhow::anyhow;
use tokio::process::Command;
use tokio::fs;

use crate::Handler;

pub type Context<'a> = poise::Context<'a, Handler, anyhow::Error>;

#[poise::command(slash_command)]
pub async fn download(
    ctx: Context<'_>,

    #[description = "url of the youtube/instagram/etc.. video to download (with yt-dlp)"]
    url: String,

    #[description = "whether or not to remove audio from the video. (default = False)"]
    should_remove_audio: Option<bool>
) -> anyhow::Result<()> {
    let reply_handle = ctx.say("downloading..").await?;

    match download_video(&url).await
    {
        Ok(filename) => {
            if let Some(should_remove) = should_remove_audio {
                if should_remove { remove_audio(&filename).await?; }
            }

            match CreateAttachment::path(&filename).await
            {
                Ok(attachment) => {
                    let reply = CreateReply::default()
                        .content(format!("`{}`:", ctx.author().display_name()))
                        .attachment(attachment);

                    if let Err(why) = ctx.send(reply).await
                    {
                        ctx.say(format!("{why}")).await?;
                    }
                },
                Err(e) => {
                    ctx.say("Rate limit reached, consider touching grass or taking a shower.").await?;
                    eprintln!("Error getting local download: {e}");
                }
            }

            fs::remove_file(filename).await;
        },
        Err(err) => { ctx.say(format!("{err}")).await?; }

    }


    reply_handle.delete(ctx).await?;

    Ok(())
}

async fn download_video(url: &str) -> anyhow::Result<String>
{
    let mut random_filename: String = (0..10).map(|_| rng().sample(rand::distr::Alphanumeric) as char).collect();
    random_filename += ".mp4";

    if let Err(status) = Command::new("yt-dlp")
        .args(["-o", &random_filename, url])
        .status()
        .await
    {
        return Err(anyhow!(format!("yt-dlp error: {status}")));
    }

    match Command::new("ffmpeg")
        .args(["-i", &random_filename, "-c:v", "libvpx",
            "-deadline", "good", "-cpu-used", "4", "-crf", "32",
            "-threads", "2", "-an", "extracted.webm"])
        .status()
        .await
    {
        Ok(_) => {
            fs::remove_file(&random_filename).await;
            fs::copy("extracted.webm", &random_filename).await;
            fs::remove_file("extracted.webm").await;
        },

        Err(status) => {
            fs::remove_file("extracted.webm").await;
            return Err(anyhow!(format!("ffmpeg error: {status}")));
        }
    }

    Ok(random_filename)
}

async fn remove_audio(filename: &str) -> anyhow::Result<()>
{
    match Command::new("ffmpeg")
        .args(["-i", filename, "-c", "copy", "-an", "extracted.webm"])
        .status()
        .await
    {
        Ok(_) => {
            fs::remove_file(filename).await;
            fs::copy("extracted.webm", filename).await;
            fs::remove_file("extracted.webm").await;
        },

        Err(status) => {
            fs::remove_file("extracted.webm").await;
            return Err(anyhow!(format!("ffmpeg error: {status}")));
        }
    }

    Ok(())
}
