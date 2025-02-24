use obfstr::obfstr;
use serenity::{
    model::{channel::Message, gateway::Ready, id::EmojiId},
    gateway::ActivityData,
    all::ReactionType,
    async_trait,
    prelude::*,
};

const SHOWCASE_CHANNELS: [u64; 4] = [
    0677869233803100171  /* #showcase */,
    0964023097843937280  /* #wallpapers */,
    1294352242719068292  /* #books */,
    0788975142684459058  /* #github-showcase */
];

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        ctx.set_activity(
            Some(
                ActivityData::streaming("swatting flies in cisco's basement", "https://twitch.tv/zzz").expect("MAKE_STREAMING_STATUS")
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
            } else {
                if let Err(why) = msg.delete(&ctx.http).await {
                    eprintln!("Error deleting message by {}: {why:?}", msg.author.name);
                }
            }
        } else if msg.channel_id.get() == 660353693283123231 /* memes */ {
            if msg.attachments.len() > 0 || msg.embeds.len() > 0 {
                add_vote_reactions(&ctx, &msg).await;
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
        id: EmojiId::new(1343553212481015818),
        name: Some("downvote".to_string()),
    }];

    for reaction in reactions {
        if let Err(why) = msg.react(&ctx.http, reaction).await {
            eprintln!("Error reacting to message by {}: {why:?}", msg.author.name);
        }
    }
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(
            obfstr!("TOKEN"),
            intents
        ).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
