use hecs::World;

use crate::commands::Commands;
use crate::events::Events;
use crate::resources::Resources;
use crate::schedule::{RunCondition, Schedule, Stage, SystemFn, SystemId};
use crate::time::Time;

pub struct SystemContext<'a> {
    pub world: &'a mut World,
    pub resources: &'a mut Resources,
    pub commands: &'a mut Commands,
    pub events: &'a mut Events,
}

pub struct App {
    pub world: World,
    resources: Resources,
    commands: Commands,
    events: Events,
    schedule: Schedule,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            resources: Resources::default(),
            commands: Commands::default(),
            events: Events::default(),
            schedule: Schedule::default(),
        }
    }

    pub fn insert_resource<T: Send + Sync + 'static>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    pub fn resource<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.resources.get()
    }

    pub fn resource_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.resources.get_mut()
    }

    pub fn add_system(&mut self, stage: Stage, system: SystemFn) -> SystemId {
        self.schedule.add_system(stage, system)
    }

    pub fn add_system_with_condition(
        &mut self,
        stage: Stage,
        system: SystemFn,
        run_if: RunCondition,
    ) -> SystemId {
        self.schedule
            .add_system_with_condition(stage, system, Some(run_if))
    }

    pub fn add_system_after(
        &mut self,
        stage: Stage,
        system: SystemFn,
        after: SystemId,
    ) -> SystemId {
        self.schedule.add_system_after(stage, system, after)
    }

    pub fn run_stage(&mut self, stage: Stage) {
        let systems = self.schedule.sort_stage(stage);
        for entry in systems {
            if let Some(run_if) = entry.run_if {
                if !run_if(&self.resources) {
                    continue;
                }
            }
            let mut context = SystemContext {
                world: &mut self.world,
                resources: &mut self.resources,
                commands: &mut self.commands,
                events: &mut self.events,
            };
            (entry.function)(&mut context);
        }
    }

    pub fn tick(&mut self) {
        for &stage in Stage::ORDER {
            if matches!(stage, Stage::Extract | Stage::Render) {
                continue;
            }
            self.run_stage(stage);
            if stage == Stage::PostUpdate {
                self.commands.flush(&mut self.world);
            }
        }
    }

    pub fn tick_with_render(&mut self) {
        for &stage in Stage::ORDER {
            self.run_stage(stage);
            if stage == Stage::PostUpdate {
                self.commands.flush(&mut self.world);
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.events.clear();
    }

    pub fn drain_events<T: Send + 'static>(&mut self) -> Vec<T> {
        self.events.drain()
    }

    pub fn system_context<R>(&mut self, f: impl FnOnce(&mut SystemContext<'_>) -> R) -> R {
        let mut context = SystemContext {
            world: &mut self.world,
            resources: &mut self.resources,
            commands: &mut self.commands,
            events: &mut self.events,
        };
        f(&mut context)
    }

    pub fn run_headless(&mut self, fixed_delta: f32, max_ticks: u64) {
        if self.resource::<Time>().is_none() {
            self.insert_resource(Time::new(fixed_delta));
        }
        for _ in 0..max_ticks {
            if let Some(time) = self.resources.get_mut::<Time>() {
                time.advance_fixed();
            }
            self.tick();
            self.end_frame();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    thread_local! {
        static COUNTER: Cell<u32> = const { Cell::new(0) };
    }

    fn increment_system(_ctx: &mut SystemContext<'_>) {
        COUNTER.with(|counter| counter.set(counter.get() + 1));
    }

    fn emit_and_spawn_system(ctx: &mut SystemContext<'_>) {
        ctx.events.send(42u32);
        ctx.commands.push(|world| {
            let _ = world.spawn(());
        });
    }

    fn consume_event_system(ctx: &mut SystemContext<'_>) {
        let events: Vec<u32> = ctx.events.drain();
        assert_eq!(events, vec![42]);
    }

    #[test]
    fn schedule_runs_stages_in_order() {
        COUNTER.with(|counter| counter.set(0));

        let mut app = App::new();
        let emit = app.add_system(Stage::Update, emit_and_spawn_system);
        app.add_system_after(Stage::Update, consume_event_system, emit);
        app.add_system(Stage::PostUpdate, increment_system);
        app.tick();

        COUNTER.with(|counter| assert_eq!(counter.get(), 1));
        assert_eq!(app.world.len(), 1);
    }
}
