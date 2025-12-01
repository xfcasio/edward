use serenity::{
    model::channel::Message,
    prelude::*,
};
use poise::serenity_prelude as serenity;

/// A message processor takes a serenity Message and processes it,
/// allowing us to break down work into disjoint blocks.
pub trait StaticMessageProcessor { async fn process(&self, _: &Context, _: &Message) {} }
pub trait DynamicMessageProcessor { async fn process(&self, _: &mut Context, _: &Message) {} }
pub trait ModerationMessageProcessor {
    async fn process(&self, _: &mut Context, _: &Message) -> Propagation { Propagation::Propagate }
}

#[derive(PartialEq)]
pub enum Propagation { Propagate, Stop }

pub struct StaticMessageProcessorList<F, Ps: StaticMessageProcessor>(F, Ps);
impl<
    F: AsyncFn (&Context, &Message),
    Ps: StaticMessageProcessor
>
    StaticMessageProcessor for StaticMessageProcessorList<F, Ps>
{
    async fn process(&self, ctx: &Context, msg: &Message)
    {
        self.0(ctx, msg).await;
        self.1.process(ctx, msg).await;
    }
}

pub struct DynamicMessageProcessorList<F, Ps: DynamicMessageProcessor>(F, Ps);
impl<
    F: AsyncFn (&mut Context, &Message),
    Ps: DynamicMessageProcessor
>
    DynamicMessageProcessor for DynamicMessageProcessorList<F, Ps>
{
    async fn process(&self, ctx: &mut Context, msg: &Message)
    {
        self.0(ctx, msg).await;
        self.1.process(ctx, msg).await;
    }
}

pub struct ModerationMessageProcessorList<F, Ps: ModerationMessageProcessor>(F, Ps);
impl<
    F: AsyncFn (&mut Context, &Message) -> Propagation,
    Ps: ModerationMessageProcessor
>
    ModerationMessageProcessor for ModerationMessageProcessorList<F, Ps>
{
    async fn process(&self, ctx: &mut Context, msg: &Message) -> Propagation
    {
        if self.0(ctx, msg).await == Propagation::Propagate {
            return self.1.process(ctx, msg).await;
        }

        Propagation::Stop
    }
}

/// End marker for the heterogeneous-list
pub struct SentinelMessageProcessor;
impl StaticMessageProcessor for SentinelMessageProcessor {}
impl DynamicMessageProcessor for SentinelMessageProcessor {}
impl ModerationMessageProcessor for SentinelMessageProcessor {}

/// Type-safe api for scheduling message interaction systems.
/// Execution order policy: Moderation systems -> Dynamic systems -> Static systems
pub struct PriorityGroup<
    ModerationMessageProcessors: ModerationMessageProcessor,
    DynamicMessageProcessors: DynamicMessageProcessor,
    StaticMessageProcessors: StaticMessageProcessor,
> {
    /// Read/Reply/React/Delete perms on the input Message.
    pub moderation: ModerationMessageProcessors,

    /// Read/Reply/React perms on the input Message.
    pub dynamic: DynamicMessageProcessors,

    /// Read-only perms on the input Message.
    pub r#static: StaticMessageProcessors,
}

impl PriorityGroup<SentinelMessageProcessor, SentinelMessageProcessor, SentinelMessageProcessor>
{
    pub fn new() -> Self
    {
        PriorityGroup {
            moderation: const { SentinelMessageProcessor },
            dynamic: const { SentinelMessageProcessor },
            r#static: const { SentinelMessageProcessor }
        }
    }
}

impl<
    ModerationMessageProcessors: ModerationMessageProcessor,
    DynamicMessageProcessors: DynamicMessageProcessor,
    StaticMessageProcessors: StaticMessageProcessor,
>
    PriorityGroup<ModerationMessageProcessors, DynamicMessageProcessors, StaticMessageProcessors>
{
    pub fn with_moderation_system<F: AsyncFn (&mut Context, &Message) -> Propagation>(self, system: F)
        -> PriorityGroup<ModerationMessageProcessorList<F, ModerationMessageProcessors>, DynamicMessageProcessors, StaticMessageProcessors>
    {
        PriorityGroup {
            moderation: ModerationMessageProcessorList(system, self.moderation),
            dynamic: self.dynamic,
            r#static: self.r#static
        }
    }

    pub fn with_dynamic_system<F: AsyncFn (&mut Context, &Message)>(self, system: F)
        -> PriorityGroup<ModerationMessageProcessors, DynamicMessageProcessorList<F, DynamicMessageProcessors>, StaticMessageProcessors>
    {
        PriorityGroup {
            moderation: self.moderation,
            dynamic: DynamicMessageProcessorList(system, self.dynamic),
            r#static: self.r#static
        }
    }

    pub fn with_static_system<F: AsyncFn (&Context, &Message)>(self, system: F)
        -> PriorityGroup<ModerationMessageProcessors, DynamicMessageProcessors, StaticMessageProcessorList<F, StaticMessageProcessors>>
    {
        PriorityGroup {
            moderation: self.moderation,
            dynamic: self.dynamic,
            r#static: StaticMessageProcessorList(system, self.r#static),
        }
    }

    pub async fn start(self, mut ctx: Context, msg: Message)
    {
        if self.moderation.process(&mut ctx, &msg).await == Propagation::Stop { return; };
        self.dynamic.process(&mut ctx, &msg).await;
        self.r#static.process(&ctx, &msg).await;
    }
}
