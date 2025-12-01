use obfstr::obfstr;
use serenity::{
    model::{channel::{Message, Reaction}, gateway::Ready},
    gateway::ActivityData,
    async_trait,
    prelude::*,
};
use poise::serenity_prelude as serenity;
use anyhow::Result;

mod fetch;
mod group_system;
mod systems;

// add vote reactions to posts and remove non-posts
const SHOWCASE_CHANNELS: [u64; 5] = [
    0677869233803100171  /* #showcase */,
    0964023097843937280  /* #wallpapers */,
    1294352242719068292  /* #books */,
    0788975142684459058  /* #github-showcase */,
    1431695114807410809  /* #hall-of-fame */
];

// add vote reactions to posts only
const VOTE_CHANNELS: [u64; 1] = [
    0660353693283123231  /* #memes */,
];

const BLACKLISTED_REACTION_USERS: [u64; 1] = [
    735569837522157629
];

pub struct Handler;

#[tokio::main]
async fn main() -> Result<()>
{
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

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

    let client = serenity::ClientBuilder::new(obfstr!("TOKEN"), intents)
        .framework(framework)
        .event_handler(Handler).await;

    client?.start().await?;
    Ok(())
}

#[async_trait]
impl EventHandler for Handler
{
    async fn ready(&self, ctx: Context, _: Ready)
    {
        ctx.set_activity(Some(
            ActivityData::streaming("swatting flies in cisco's basement", "https://twitch.tv/zzz")
                .expect("MAKE_STREAMING_STATUS")
        ));
    }

    async fn message(&self, ctx: Context, msg: Message)
    {
        group_system::PriorityGroup::new()
            .with_moderation_system(systems::showcase_cleaner_and_voter)
            .with_dynamic_system(systems::rizz_ping)
            .start(ctx, msg)
            .await;
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction)
    {
        let user_id = reaction.user_id.expect("FAILED_RETRIEVING_REACTION_USER").get();

        if BLACKLISTED_REACTION_USERS.contains(&user_id) {
            reaction.delete(&ctx.http).await.expect("FAILED_REMOVING_BLACKLISTED_USER_REACTION");
        }
    }
}
