use engine_core::{SystemContext, Time};
use glam::Vec2;

use crate::axes::wrap_angle;
use crate::components::{
    BlockMiningState, LocomotionState, Player, PlayerAnimation, PLAYER_MAX_HEAD_YAW, Transform,
    Velocity,
};

/// Approximate survival walk speed (blocks/s) for normalizing limb swing amount.
const WALK_SPEED: f32 = 4.32;
const SWING_AMOUNT_LERP: f32 = 4.0;
const MIN_SWING_SPEED: f32 = 0.05;
/// Walk-cycle phase advances this many times faster than vanilla MC.
const LIMB_SWING_SPEED_MULT: f32 = 4.0;
/// Torso catch-up speed when look exceeds [`PLAYER_MAX_HEAD_YAW`] (rad/s).
const BODY_CATCH_UP_SPEED: f32 = 12.0;
const ARM_SWING_AMOUNT_LERP: f32 = 10.0;
const DIG_PHASE_SPEED: f32 = 27.0;
const PLACE_PHASE_SPEED: f32 = DIG_PHASE_SPEED * 1.5;

pub fn player_animation_system(ctx: &mut SystemContext<'_>) {
    let delta = ctx
        .resources
        .get::<Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(0.0);
    if delta <= 0.0 {
        return;
    }

    let entities: Vec<_> = ctx
        .world
        .query::<(&Player, &Transform, &Velocity, &LocomotionState)>()
        .iter()
        .map(|(entity, (_, transform, velocity, locomotion))| {
            (entity, transform.yaw, velocity.0, locomotion.on_ground)
        })
        .collect();

    let blend = (delta * SWING_AMOUNT_LERP).clamp(0.0, 1.0);
    let arm_swing_blend = (delta * ARM_SWING_AMOUNT_LERP).clamp(0.0, 1.0);
    let body_step = (BODY_CATCH_UP_SPEED * delta).min(1.0);
    for (entity, look_yaw, velocity, on_ground) in entities {
        let speed = Vec2::new(velocity.x, velocity.y).length();
        let target_amount = if on_ground {
            (speed / WALK_SPEED).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let Ok(mut anim) = ctx.world.get::<&mut PlayerAnimation>(entity) else {
            continue;
        };
        anim.limb_swing_amount += (target_amount - anim.limb_swing_amount) * blend;
        if on_ground && speed > MIN_SWING_SPEED {
            anim.limb_swing += speed * delta * LIMB_SWING_SPEED_MULT;
        }

        let is_digging = ctx
            .world
            .get::<&BlockMiningState>(entity)
            .ok()
            .is_some_and(|mining| mining.target.is_some());
        let target_dig = if is_digging { 1.0 } else { 0.0 };
        anim.dig_amount += (target_dig - anim.dig_amount) * arm_swing_blend;
        if is_digging {
            anim.dig_phase += delta * DIG_PHASE_SPEED;
        }

        if anim.place_loop_remaining > 0.0 {
            let step = (delta * PLACE_PHASE_SPEED).min(anim.place_loop_remaining);
            anim.place_phase += step;
            anim.place_loop_remaining -= step;
        }
        let target_place = if anim.place_loop_remaining > 0.0 {
            1.0
        } else {
            0.0
        };
        anim.place_amount += (target_place - anim.place_amount) * arm_swing_blend;

        let relative_yaw = wrap_angle(look_yaw - anim.body_yaw);
        if relative_yaw.abs() > PLAYER_MAX_HEAD_YAW {
            let head_offset = relative_yaw.clamp(-PLAYER_MAX_HEAD_YAW, PLAYER_MAX_HEAD_YAW);
            let target_body_yaw = wrap_angle(look_yaw - head_offset);
            let delta_yaw = wrap_angle(target_body_yaw - anim.body_yaw);
            anim.body_yaw = wrap_angle(anim.body_yaw + delta_yaw * body_step);
        }
    }
}
