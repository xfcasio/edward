#![allow(unused)]

use rand::prelude::*;
use serenity::all::Attachment;
use serenity::all::CreateAttachment;
use std::process::Command;
use rand::rng;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use anyhow::anyhow;

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

    match download_video(&url)
    {
        Ok(filename) => {
            if let Some(should_remove) = should_remove_audio {
                if should_remove { remove_audio(&filename)?; }
            }

            match CreateAttachment::path(&filename).await
            {
                Ok(attachment) => {
                    let reply = CreateReply::default()
                        .content(format!("`{}`:", ctx.author().display_name()))
                        .attachment(attachment);

                    ctx.send(reply).await?;
                },
                Err(e) => {
                    ctx.say("Rate limit reached, consider touching grass or taking a shower.").await?;
                    eprintln!("Error getting local download: {e}");
                }
            }

            std::fs::remove_file(filename);
        },
        Err(err) => { ctx.say(format!("{err}")).await?; }

    }


    reply_handle.delete(ctx).await?;

    Ok(())
}

fn download_video(url: &str) -> anyhow::Result<String>
{
    let mut random_filename: String = (0..10).map(|_| rng().sample(rand::distr::Alphanumeric) as char).collect();
    random_filename += ".mp4";

    if let Err(status) = Command::new("yt-dlp")
        .args(["-o", &random_filename, url])
        .status()
    {
        return Err(anyhow!(format!("yt-dlp error: {status}")));
    }

    match Command::new("ffmpeg")
        .args(["-i", &random_filename, "extracted.webm"])
        .status()
    {
        Ok(_) => {
            std::fs::remove_file(&random_filename);
            std::fs::copy("extracted.webm", &random_filename);
            std::fs::remove_file("extracted.webm");
        },

        Err(status) => {
            std::fs::remove_file("extracted.webm");
            return Err(anyhow!(format!("ffmpeg error: {status}")));
        }
    }

    Ok(random_filename)
}

fn remove_audio(filename: &str) -> anyhow::Result<()>
{
    match Command::new("ffmpeg")
        .args(["-i", filename, "-c", "copy", "-an", "extracted.webm"])
        .status()
    {
        Ok(_) => {
            std::fs::remove_file(filename);
            std::fs::copy("extracted.webm", filename);
            std::fs::remove_file("extracted.webm");
        },

        Err(status) => {
            std::fs::remove_file("extracted.webm");
            return Err(anyhow!(format!("ffmpeg error: {status}")));
        }
    }

    Ok(())
}
