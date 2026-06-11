use engine_core::{SystemContext, Time};

use crate::day_night::{DayNightCycle, DAY_TICKS};

pub fn day_night_system(ctx: &mut SystemContext<'_>) {
    let Some(time) = ctx.resources.get::<Time>() else {
        return;
    };
    let dt = time.fixed_delta;
    let Some(cycle) = ctx.resources.get_mut::<DayNightCycle>() else {
        return;
    };
    let ticks_per_sec = DAY_TICKS / cycle.day_length_secs;
    cycle.world_time = (cycle.world_time + ticks_per_sec * dt) % DAY_TICKS;
}
