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
) -> Result<(), anyhow::Error> {
    ctx.say("downloading..").await?;

    match download_video(&url)
    {
        Ok(filename) => {
            match CreateAttachment::path(filename.clone()).await
            {
                Ok(attachment) => {
                    let reply = CreateReply::default()
                        .attachment(attachment);

                    ctx.send(reply).await?;
                },
                Err(e) => { ctx.say(format!("Error getting local download: {e}")).await?; }
            }

            std::fs::remove_file(filename).expect("RM_ERROR")
        },
        Err(err) => { ctx.say(err).await?; }

    }

    Ok(())
}

fn download_video(url: &str) -> Result<String, String>
{
    let mut random_filename: String = (0..10)
        .map(|_| rng().sample(rand::distr::Alphanumeric) as char)
        .collect();

    random_filename += ".mp4";

    if let Err(status) = Command::new("yt-dlp")
        .args(["-o", &random_filename, url])
        .status()
    {
        return Err(format!("yt-dlp error: {status}"));
    }

    Ok(random_filename)
}
