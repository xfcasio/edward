use std::future::Future;
use serenity::{
    model::{channel::Message, id::EmojiId},
    all::ReactionType,
    prelude::*,
};
use poise::serenity_prelude as serenity;
use anyhow::{anyhow, Result};

use crate::{SHOWCASE_CHANNELS, VOTE_CHANNELS};

/// DynamicMessageProcessor
pub async fn rizz_ping(ctx: &mut Context, msg: &Message)
{
    if msg.content.to_lowercase().contains("!rizz") {
        if let Err(why) = msg.channel_id.say(&ctx.http, "\\*looksmaxxes\\*").await {
            println!("Error sending message: {why:?}");
        }
    }
}

/// DynamicMessageProcessor
pub async fn showcase_cleaner_and_voter(ctx: &mut Context, msg: &Message)
{
    if SHOWCASE_CHANNELS.contains(&msg.channel_id.get()) || VOTE_CHANNELS.contains(&msg.channel_id.get()) {
        let is_post = msg.attachments.len() > 0
            || msg.embeds.len() > 0
            || msg.content.starts_with("https://");

        let is_post = is_post && !(
            msg.embeds.len() == 1 &&
            msg.embeds[0].url
                .as_deref()
                .unwrap_or_default()
                .starts_with("https://cdn.discordapp.com/emojis")
        );

        if is_post { add_vote_reactions(&ctx, &msg).await; }
        else if !VOTE_CHANNELS.contains(&msg.channel_id.get()) {
            if let Err(why) = msg.delete(&ctx.http).await {
                eprintln!("Error deleting message by {}: {why:?}", msg.author.name);
            }
        }
    }
}


async fn add_vote_reactions(ctx: &Context, msg: &Message)
{
    let reactions = [
        ReactionType::Custom { animated: false, id: EmojiId::new(1343553189508681728), name: Some("upvote".to_string()), },
        ReactionType::Custom { animated: false, id: EmojiId::new(1343558658872709141), name: Some("downvote".to_string()) }
    ];

    for reaction in reactions {
        retry(3, reaction, async |reaction| msg.react(&ctx.http, reaction).await).await.unwrap();
    }
}

async fn retry<T, U, E, Fut>(mut retry_number: usize, argument: T, f: impl Fn(T) -> Fut) -> Result<U>
    where
        E: std::fmt::Debug,
        T: Clone,
        Fut: Future<Output = Result<U, E>>
{
    loop {
        match f(argument.clone()).await {
            Ok(value) => return Ok(value),
            Err(why) if retry_number > 0 => {
                eprintln!("[retry #{retry_number}]: {why:?}");
                retry_number -= 1;
            }
            Err(_) => return Err(anyhow!("retry limit reached!"))
        }
    }
}
