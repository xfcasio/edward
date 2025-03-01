use poise::serenity_prelude as serenity;
use poise::futures_util::StreamExt;
use serenity::all::{CreateEmbed, Channel};
use serenity::{
    model::{channel::{Message, ReactionType::{Custom, Unicode}, MessageReaction}, id::ChannelId},
    prelude::*,
};
use poise::CreateReply;
use rayon::prelude::*;

use crate::Handler;

pub type Context<'a> = poise::Context<'a, Handler, anyhow::Error>;

#[derive(Debug, poise::ChoiceParameter)]
enum ChannelOption {
    #[name = "#🍙-showcase"] Showcase     = 0677869233803100171,
    #[name = "#📷-wallpapers"] Wallpapers = 0964023097843937280,
    #[name = "#📜-books"] Books           = 1294352242719068292,
    #[name = "#memes"] Memes              = 660353693283123231,
    #[name = "#github-showcase"] GithubShowcase = 0788975142684459058,
}

#[poise::command(slash_command)]
pub async fn fetch(
    ctx: Context<'_>,
    #[description = "Fetch top N posts"]
    #[min = 1]
    top: Option<u32>,

    #[description = "Fetch lowest N posts"]
    #[min = 1]
    lowest: Option<u32>,

    #[description = "Showcase channel to fetch posts from"]
    channel: ChannelOption,
) -> Result<(), anyhow::Error> {
    if top.is_some() && lowest.is_some() {
        ctx.say("You can only specify either `top` or `lowest`, not both!").await?;
        return Ok(());
    } else if top.is_none() && lowest.is_none() {
        ctx.say("You must specify either one of `top` or `lowest` (`<= 9` due to embed limit)").await?;
        return Ok(());
    }
    ctx.defer().await?;

    let channel_id = ChannelId::new(channel as u64);

    // maybe just messages[(len - N)..]
    let (sorting_coefficient, num) = if let Some(n) = top {
        (-1, n)
    } else if let Some(n) = lowest {
        (1, n)
    } else {
        return Ok(());
    };

    let target_channel_name = match channel_id.to_channel(&ctx.http()).await {
        Ok(channel) => {
            match channel {
                Channel::Guild(guild_channel) => guild_channel.name,
                _ => unreachable!()
            }
        },

        Err(_) => unreachable!(/* this doesn't make sense - hence unreachable */)
    };

    let messages = capture_channel_posts(&ctx, channel_id, sorting_coefficient).await;

    let mut embeds = vec![CreateEmbed::new()
        .title(format!("{} posts in #{}",
            if sorting_coefficient == -1 { "Top rated" }
            else { "Lowest rated" },
            target_channel_name
        ))
    ];

    for m in (0..num).map(|i| messages.get(i as usize)).flatten() {
        if let Some(attachment) = m.attachments.first() {
            embeds.push(
                CreateEmbed::new()
                    .title(format!("Author: {}", m.author.name))
                    .description(format!("💙 likes - {}", get_post_votes(&m)))
                    .timestamp(m.timestamp)
                    .image(attachment.url.clone())
            );
        }
    }

    let mut reply = CreateReply::default();
    reply.embeds = embeds;

    ctx.send(reply).await?;

    Ok(())
}


async fn capture_channel_posts(ctx: &Context<'_>, channel_id: ChannelId, sorting_coefficient: isize) -> Vec<Message> {
    let mut posts: Vec<Message> = vec![];
    
    let is_post = |r: &MessageReaction| -> bool {
        match &r.reaction_type {
            Custom { id: reaction_id, .. } => {
                reaction_id.get() == 1343553189508681728
            },
            Unicode(emoji) => { (emoji == "💙") || (emoji == "😂") },
            _ => unreachable!()
        }
    };

    let mut message_iterator = channel_id.messages_iter(ctx.http()).boxed(); // boxed?
    while let Some(Ok(m)) = message_iterator.next().await {
        if m.reactions.iter().any(is_post) {
            posts.push(m);
        }
    }

    posts.par_sort_by_key(|m| sorting_coefficient * get_post_votes(&m));
    posts
}

fn get_post_votes(m: &Message) -> isize {
    let mut votes = 0isize;

    for r in &m.reactions {
        match &r.reaction_type {
            Custom { name: Some(name), .. } => {
                match name.as_str() {
                    "upvote" => { votes += r.count as isize },
                    "downvote" => { votes -= r.count as isize },
                    _ => {}
                }
            },
            Unicode(emoji) => {
                if (emoji == "💙") || (emoji == "😂") { votes += r.count as isize }
            }
            _ => unreachable!()
        }
    }

    votes
}

