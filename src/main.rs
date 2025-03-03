use obfstr::obfstr;
use serenity::{
    model::{channel::Message, gateway::Ready, id::EmojiId},
    gateway::ActivityData,
    all::ReactionType,
    async_trait,
    prelude::*,
};
use poise::serenity_prelude as serenity;
use anyhow::Result;

mod fetch;

const SHOWCASE_CHANNELS: [u64; 5] = [
    0677869233803100171  /* #showcase */,
    0964023097843937280  /* #wallpapers */,
    1294352242719068292  /* #books */,
    0788975142684459058  /* #github-showcase */,
    0660353693283123231  /* #memes */
];

pub struct Handler;

#[tokio::main]
async fn main() -> Result<()> {
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![crate::fetch::fetch()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Handler)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(
        obfstr!("TOKEN"),
        intents
    ).framework(framework)
    .event_handler(Handler).await;

    client?.start().await?;
    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        ctx.set_activity(
            Some(
                ActivityData::streaming("swatting flies in cisco's basement", "https://twitch.tv/zzz")
                    .expect("MAKE_STREAMING_STATUS")
            )
        );
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.contains("!rizz") || msg.content.contains("!Rizz") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "\\*looksmaxxes\\*").await {
                println!("Error sending message: {why:?}");
            }
        }

        if SHOWCASE_CHANNELS.contains(&msg.channel_id.get()) {
            if msg.attachments.len() > 0 || msg.embeds.len() > 0 || msg.content.contains("https://") {
                add_vote_reactions(&ctx, &msg).await;
            } else if msg.channel_id.get() != 660353693283123231 /* memes */ {
                if let Err(why) = msg.delete(&ctx.http).await {
                    eprintln!("Error deleting message by {}: {why:?}", msg.author.name);
                }
            }
        }
    }
}

async fn add_vote_reactions(ctx: &Context, msg: &Message) {
    let reactions = [ReactionType::Custom {
        animated: false,
        id: EmojiId::new(1343553189508681728),
        name: Some("upvote".to_string()),
    }, ReactionType::Custom {
        animated: false,
        id: EmojiId::new(1343558658872709141),
        name: Some("downvote".to_string()),
    }];

    for reaction in reactions {
        if let Err(why) = msg.react(&ctx.http, reaction).await {
            eprintln!("Error reacting to message by {}: {why:?}", msg.author.name);
        }
    }
}
