use std::marker::PhantomData;

use serenity::{
    model::channel::{Message, Reaction},
    prelude::*,
};
use poise::serenity_prelude as serenity;

/// A processor takes a Data item (Message, Reaction) and processes it,
/// allowing us to break down work into disjoint blocks.
pub trait StaticProcessor {
    type D: ProcessorData;

    async fn process(&self, _: &Context, _: &<Self as StaticProcessor>::D) {}
}

pub trait DynamicProcessor {
    type D: ProcessorData;

    async fn process(&self, _: &mut Context, _: &<Self as DynamicProcessor>::D) {}
}

pub trait ModerationProcessor {
    type D: ProcessorData;

    async fn process(&self, _: &mut Context, _: &<Self as ModerationProcessor>::D) -> Propagation { Propagation::Propagate }
}

pub trait ProcessorData {}
impl ProcessorData for Message {}
impl ProcessorData for Reaction {}

#[derive(PartialEq)]
pub enum Propagation { Propagate, Stop }

pub struct StaticProcessorList<F, Data: ProcessorData, Ps>(F, Ps) where Ps: StaticProcessor<D = Data>;
impl<
    F: AsyncFn (&Context, &Data),
    Data: ProcessorData,
    Ps: StaticProcessor<D = Data>
>
    StaticProcessor for StaticProcessorList<F, Data, Ps>
{
    type D = Data;

    async fn process(&self, ctx: &Context, data: &Data)
    {
        self.0(ctx, data).await;
        self.1.process(ctx, data).await;
    }
}

pub struct DynamicProcessorList<F, Data: ProcessorData, Ps>(F, Ps) where Ps: DynamicProcessor<D = Data>;
impl<
    F: AsyncFn (&mut Context, &Data),
    Data: ProcessorData,
    Ps: DynamicProcessor<D = Data>
>
    DynamicProcessor for DynamicProcessorList<F, Data, Ps>
{
    type D = Data;

    async fn process(&self, ctx: &mut Context, data: &Data)
    {
        self.0(ctx, data).await;
        self.1.process(ctx, data).await;
    }
}

pub struct ModerationProcessorList<F, Data: ProcessorData, Ps>(F, Ps) where Ps: ModerationProcessor<D = Data>;
impl<
    F: AsyncFn (&mut Context, &Data) -> Propagation,
    Data: ProcessorData,
    Ps: ModerationProcessor<D = Data>
>
    ModerationProcessor for ModerationProcessorList<F, Data, Ps>
{
    type D = Data;

    async fn process(&self, ctx: &mut Context, data: &Data) -> Propagation
    {
        if self.0(ctx, data).await == Propagation::Propagate {
            return self.1.process(ctx, data).await;
        }

        Propagation::Stop
    }
}

/// End marker for the heterogeneous-list
pub struct SentinelMessageProcessor<Data: ProcessorData>(PhantomData<Data>);
impl<Data: ProcessorData> StaticProcessor for SentinelMessageProcessor<Data> { type D = Data; }
impl<Data: ProcessorData> DynamicProcessor for SentinelMessageProcessor<Data> { type D = Data; }
impl<Data: ProcessorData> ModerationProcessor for SentinelMessageProcessor<Data> { type D = Data; }

/// Type-safe api for scheduling interaction systems.
/// Execution order policy: Moderation systems -> Dynamic systems -> Static systems
pub struct PriorityGroup<
    Data: ProcessorData,
    ModerationProcessors: ModerationProcessor<D = Data>,
    DynamicProcessors: DynamicProcessor<D = Data>,
    StaticProcessors: StaticProcessor<D = Data>,
> {
    /// Read/Reply/React/Delete perms on the input Data.
    pub moderation: ModerationProcessors,

    /// Read/Reply/React perms on the input Data.
    pub dynamic: DynamicProcessors,

    /// Read-only perms on the input Data.
    pub r#static: StaticProcessors,
}

impl<Data: ProcessorData> PriorityGroup<Data, SentinelMessageProcessor<Data>, SentinelMessageProcessor<Data>, SentinelMessageProcessor<Data>>
{
    pub fn new() -> Self
    {
        PriorityGroup {
            moderation: const { SentinelMessageProcessor(PhantomData) },
            dynamic: const { SentinelMessageProcessor(PhantomData) },
            r#static: const { SentinelMessageProcessor(PhantomData) }
        }
    }
}

impl<
    Data: ProcessorData,
    ModerationProcessors: ModerationProcessor<D = Data>,
    DynamicProcessors: DynamicProcessor<D = Data>,
    StaticProcessors: StaticProcessor<D = Data>,
>
    PriorityGroup<Data, ModerationProcessors, DynamicProcessors, StaticProcessors>
{
    pub fn with_moderation_system<F: AsyncFn (&mut Context, &Data) -> Propagation>(self, system: F)
        -> PriorityGroup<Data, ModerationProcessorList<F, Data, ModerationProcessors>, DynamicProcessors, StaticProcessors>
    {
        PriorityGroup {
            moderation: ModerationProcessorList(system, self.moderation),
            dynamic: self.dynamic,
            r#static: self.r#static
        }
    }

    pub fn with_dynamic_system<F: AsyncFn (&mut Context, &Data)>(self, system: F)
        -> PriorityGroup<Data, ModerationProcessors, DynamicProcessorList<F, Data, DynamicProcessors>, StaticProcessors>
    {
        PriorityGroup {
            moderation: self.moderation,
            dynamic: DynamicProcessorList(system, self.dynamic),
            r#static: self.r#static
        }
    }

    pub fn with_static_system<F: AsyncFn (&Context, &Data)>(self, system: F)
        -> PriorityGroup<Data, ModerationProcessors, DynamicProcessors, StaticProcessorList<F, Data, StaticProcessors>>
    {
        PriorityGroup {
            moderation: self.moderation,
            dynamic: self.dynamic,
            r#static: StaticProcessorList(system, self.r#static),
        }
    }

    pub async fn start(self, mut ctx: Context, data: Data)
    {
        if self.moderation.process(&mut ctx, &data).await == Propagation::Stop { return; };
        self.dynamic.process(&mut ctx, &data).await;
        self.r#static.process(&ctx, &data).await;
    }
}
