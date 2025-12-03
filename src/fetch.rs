use poise::serenity_prelude as serenity;
use poise::futures_util::StreamExt;
use serenity::all::{CreateEmbed, Channel};
use serenity::{
    model::{channel::{Message, Channel::Guild, ReactionType::{Custom, Unicode}, MessageReaction}, id::ChannelId},
    prelude::*,
};
use poise::CreateReply;
use rayon::prelude::*;
use anyhow::anyhow;

use crate::Handler;

pub type Context<'a> = poise::Context<'a, Handler, anyhow::Error>;

#[derive(Debug, poise::ChoiceParameter)]
enum ChannelOption
{
    #[name = "#🍙-showcase"] Showcase     = 0677869233803100171,
    #[name = "#📷-wallpapers"] Wallpapers = 0964023097843937280,
    #[name = "#📜-books"] Books           = 1294352242719068292,
    #[name = "#memes"] Memes              = 660353693283123231,
    #[name = "#github-showcase"] GithubShowcase = 0788975142684459058,
    #[name = "#⚜️-hall-of-fame"] HallOfFame = 1431695114807410809, 
}

#[poise::command(slash_command)]
pub async fn fetch(
    ctx: Context<'_>,
    #[description = "Fetch top N posts"]
    #[min = 1]
    top: Option<usize>,

    #[description = "Fetch lowest N posts"]
    #[min = 1]
    lowest: Option<usize>,

    #[description = "Showcase channel to fetch posts from"]
    channel: ChannelOption,
) -> Result<(), anyhow::Error> {
    if top.is_some() && lowest.is_some() {
        ctx.say("You can only specify either `top` or `lowest`, not both!").await?;
        return Ok(());
    } else if top.is_none() && lowest.is_none() {
        ctx.say("You must specify either one of `top` or `lowest`").await?;
        return Ok(());
    }
    ctx.defer().await?;

    let channel_id = ChannelId::new(channel as u64);

    // maybe just messages[(len - N)..]
    let (sorting_coefficient, num, reply_partitions) = if let Some(n) = top {
        (-1, n, (n as f64 / 10.).ceil() as usize)
    } else if let Some(n) = lowest {
        (1, n, (n as f64 / 10.).ceil() as usize)
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

    if messages.len() < num {
        let guild_id = match channel_id.to_channel(&ctx.http()).await {
            Ok(Guild(channel)) => channel.guild_id,
            _ => return Err(anyhow!("how did we get here?")),
        };

        let message_link = format!("https://discord.com/channels/{}/{}",
            guild_id,
            channel_id.get()
        );

        ctx.say(format!("{} only has {} posts.", message_link, messages.len())).await?;
        return Ok(());
    }

    let mut embeds: Vec<CreateEmbed> = Vec::with_capacity(num);
    embeds.push(CreateEmbed::new()
        .title(format!("{} {} posts in #{}",
            if sorting_coefficient == -1 { "Top" }
            else { "Lowest" },
            num,
            target_channel_name
        ))
        .thumbnail("https://cdn.discordapp.com/icons/647981638348832790/0449935cebf16998c890e0b16af0e6a0.webp")
        .image("https://media.discordapp.net/attachments/647997874940018710/1370271088151367741/image.png?ex=681ee3e5&is=681d9265&hm=2c89755338a02761d570bc19fa8a7362bbad7db100646bed8ab9b02f92d6f7e9&=&format=webp")
        .color(0x111A1F));

    for m in (0..num).map(|i| messages.get(i as usize)).flatten() {
        let message_link = format!("https://discord.com/channels/{}/{}/{}",
            match m.guild_id { Some(id) => format!("{id}"), None => "@me".to_owned() },
            m.channel_id, m.id
        );

        let message_content_trimmed = if m.content.len() > 256 { &format!("{}...", &m.content[0..253]) }
            else { &m.content };

        let mut item = CreateEmbed::new()
            .title(message_content_trimmed)
            .timestamp(m.timestamp)
            .thumbnail(
                format!("https://cdn.discordapp.com/avatars/{}/{}.webp",
                    m.author.id.get(), m.author.avatar.expect("FAILED_GETTING_USER_AVATAR_HASH").to_string()
                )
            )
            .color(0xA175EB)
            .description(format!("🪶 author •• {}\n💙 likes ••• {}\n🔗 link •••• {message_link}",
                match &m.author.global_name { Some(name) => name, None => &m.author.name },
                get_post_votes(&m)
            ));

        for embed in &m.embeds {
            if let Some(embed_img) = &embed.image {
                item = item.image(&embed_img.url);
            }
        }
        
        for attachment in &m.attachments {
            item = item.image(&attachment.url);
        }

        embeds.push(item);
    }

    let replies = vec![CreateReply::default(); reply_partitions];
    for (mut reply, i) in replies.into_iter().zip(0..) {
        reply.embeds = embeds.fallback_slice(10 * i, 10 * i + 10).to_vec();
        ctx.send(reply).await?;
    }

    Ok(())
}


async fn capture_channel_posts(ctx: &Context<'_>, channel_id: ChannelId, sorting_coefficient: isize) -> Vec<Message>
{
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

fn get_post_votes(m: &Message) -> isize
{
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

trait FallbackSlice<T>
{
    fn fallback_slice(&self, start: usize, end: usize) -> &[T];
}

impl<T> FallbackSlice<T> for [T]
{
    fn fallback_slice(&self, start: usize, end: usize) -> &[T]
    {
        if start >= self.len() { return &[]; }
        if end >= self.len() { return &self[start..self.len()] }

        &self[start..end]
    }
}
