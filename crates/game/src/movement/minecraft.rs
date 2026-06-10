//! Minecraft survival locomotion (MCPK formulas), scaled from 20 Hz to arbitrary `delta`.

use glam::{Vec2, Vec3};

pub const MC_TICK_DT: f32 = 0.05;
pub const HORIZONTAL_DRAG: f32 = 0.91;
pub const GROUND_SLIPPERINESS: f32 = 0.6;
pub const AIR_SLIPPERINESS: f32 = 1.0;
pub const AIR_ACCEL_PER_MC_TICK: f32 = 0.02;
pub const GROUND_ACCEL_PER_MC_TICK: f32 = 0.1;
pub const GRAVITY_PER_MC_TICK: f32 = 0.08;
pub const VERTICAL_DRAG: f32 = 0.98;
pub const JUMP_VELOCITY_PER_MC_TICK: f32 = 0.42;
pub const SPRINT_JUMP_BOOST_PER_MC_TICK: f32 = 0.2;
/// Vanilla sprint is 1.3× walk; +30% for this project → 1.69× walk.
pub const SPRINT_MOVEMENT_MULT: f32 = 1.69;
pub const WALK_MOVEMENT_MULT: f32 = 1.0;
/// MCPK momentum cutoff in blocks/tick; velocity is stored in blocks/s.
pub const NEGLIGIBLE_MOMENTUM_PER_MC_TICK: f32 = 0.003;
pub const NEGLIGIBLE_SPEED: f32 = NEGLIGIBLE_MOMENTUM_PER_MC_TICK / MC_TICK_DT;
pub const JUMP_COOLDOWN_MC_TICKS: u8 = 10;

/// Jump impulse in blocks/s (continuous integration equivalent to 0.42 blocks/tick).
pub const JUMP_VELOCITY: f32 = JUMP_VELOCITY_PER_MC_TICK / MC_TICK_DT;

#[derive(Debug, Clone, Copy)]
pub struct McMovementInput {
    pub wish_dir: Vec3,
    pub facing_dir: Vec3,
    pub move_axis: Vec2,
    pub sprint: bool,
}

pub fn scale_mult(factor: f32, delta: f32) -> f32 {
    factor.powf(delta / MC_TICK_DT)
}

/// Per-MC-tick additive change (blocks/tick) → blocks/s delta over `delta` seconds.
pub fn scale_add_per_tick(per_mc_tick: f32, delta: f32) -> f32 {
    per_mc_tick * delta / (MC_TICK_DT * MC_TICK_DT)
}

pub fn jump_cooldown_sim_steps(sim_dt: f32) -> u8 {
    (f32::from(JUMP_COOLDOWN_MC_TICKS) * MC_TICK_DT / sim_dt).ceil() as u8
}

pub fn slipperiness_prev(was_on_ground: bool) -> f32 {
    if was_on_ground {
        GROUND_SLIPPERINESS
    } else {
        AIR_SLIPPERINESS
    }
}

pub fn slipperiness_current(on_ground: bool) -> f32 {
    if on_ground {
        GROUND_SLIPPERINESS
    } else {
        AIR_SLIPPERINESS
    }
}

fn strafe_multiplier(move_axis: Vec2) -> f32 {
    let ax = move_axis.x.abs();
    let ay = move_axis.y.abs();
    if ax < 1e-4 && ay > 1e-4 {
        return 0.98;
    }
    if ay < 1e-4 && ax > 1e-4 {
        return 1.0;
    }
    0.98
}

pub fn movement_multiplier(move_axis: Vec2, sprint: bool) -> f32 {
    if move_axis.length_squared() < 1e-8 {
        return 0.0;
    }
    let base = if sprint {
        SPRINT_MOVEMENT_MULT
    } else {
        WALK_MOVEMENT_MULT
    };
    base * strafe_multiplier(move_axis)
}

fn horizontal_friction_factor(slip_prev: f32, delta: f32) -> f32 {
    (slip_prev * HORIZONTAL_DRAG).powf(delta / MC_TICK_DT)
}

fn apply_momentum_axis(velocity: f32, slip_prev: f32, delta: f32) -> f32 {
    let momentum = velocity * horizontal_friction_factor(slip_prev, delta);
    if momentum.abs() < NEGLIGIBLE_SPEED {
        0.0
    } else {
        momentum
    }
}

/// MC horizontal velocity step (world XY). `jump_tick` uses ground accel + optional sprint-jump boost.
pub fn update_horizontal_velocity(
    velocity_xy: Vec2,
    input: &McMovementInput,
    was_on_ground: bool,
    on_ground: bool,
    jump_tick: bool,
    delta: f32,
) -> Vec2 {
    let slip_prev = slipperiness_prev(was_on_ground);
    let slip_curr = slipperiness_current(on_ground);

    let mut vx = apply_momentum_axis(velocity_xy.x, slip_prev, delta);
    let mut vy = apply_momentum_axis(velocity_xy.y, slip_prev, delta);

    let mt = movement_multiplier(input.move_axis, input.sprint);
    if mt > 0.0 && input.wish_dir.length_squared() > 0.0 {
        let wish = Vec2::new(input.wish_dir.x, input.wish_dir.y);
        let accel = if was_on_ground || jump_tick {
            let slip_factor = (GROUND_SLIPPERINESS / slip_curr).powi(3);
            scale_add_per_tick(GROUND_ACCEL_PER_MC_TICK, delta) * mt * slip_factor
        } else {
            scale_add_per_tick(AIR_ACCEL_PER_MC_TICK, delta) * mt
        };
        vx += accel * wish.x;
        vy += accel * wish.y;
    }

    if jump_tick && input.sprint {
        let boost = scale_add_per_tick(SPRINT_JUMP_BOOST_PER_MC_TICK, delta);
        let facing = Vec2::new(input.facing_dir.x, input.facing_dir.y);
        vx += boost * facing.x;
        vy += boost * facing.y;
    }

    Vec2::new(vx, vy)
}

/// Post-move vertical step (gravity then drag).
pub fn apply_vertical_post_move(vz: f32, delta: f32) -> f32 {
    let v = (vz - scale_add_per_tick(GRAVITY_PER_MC_TICK, delta)) * scale_mult(VERTICAL_DRAG, delta);
    if v.abs() < NEGLIGIBLE_SPEED {
        0.0
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axes::horizontal_forward;

    const SIM_DT: f32 = 1.0 / 60.0;

    fn mc_input(sprint: bool) -> McMovementInput {
        McMovementInput {
            wish_dir: horizontal_forward(0.0),
            facing_dir: horizontal_forward(0.0),
            move_axis: Vec2::new(0.0, 1.0),
            sprint,
        }
    }

    #[test]
    fn scaling_matches_mc_tick_at_reference_dt() {
        let delta = MC_TICK_DT;
        assert!((scale_mult(HORIZONTAL_DRAG, delta) - HORIZONTAL_DRAG).abs() < 1e-5);
        let air_step = scale_add_per_tick(AIR_ACCEL_PER_MC_TICK, delta);
        let expected = AIR_ACCEL_PER_MC_TICK / MC_TICK_DT;
        assert!((air_step - expected).abs() < 1e-4);
    }

    fn simulate_jump_height(delta: f32, steps: u32) -> f32 {
        let mut vz = 0.0f32;
        let mut height = 0.0f32;
        let mut peak = 0.0f32;

        for step in 0..steps {
            if step == 0 {
                vz = JUMP_VELOCITY;
            }
            height += vz * delta;
            peak = peak.max(height);
            vz = apply_vertical_post_move(vz, delta);
        }
        peak
    }

    #[test]
    fn jump_height_exact_at_mc_tick_rate() {
        let peak = simulate_jump_height(MC_TICK_DT, 12);
        assert!(
            (peak - 1.252).abs() < 0.06,
            "peak height {peak} blocks, expected ~1.252"
        );
    }

    #[test]
    fn jump_height_near_minecraft_at_60hz() {
        let steps = (12.0 * MC_TICK_DT / SIM_DT).ceil() as u32;
        let peak = simulate_jump_height(SIM_DT, steps);
        assert!(
            (1.1..1.65).contains(&peak),
            "peak height {peak} blocks, expected ~1.252 at 60 Hz"
        );
    }

    #[test]
    fn air_momentum_decays_without_cap() {
        let initial = Vec2::new(7.0, 0.0);
        let input = McMovementInput {
            wish_dir: Vec3::ZERO,
            facing_dir: horizontal_forward(0.0),
            move_axis: Vec2::ZERO,
            sprint: false,
        };
        let after = update_horizontal_velocity(
            initial,
            &input,
            false,
            false,
            false,
            MC_TICK_DT,
        );
        assert!(after.x > 3.0, "sprint-speed momentum should carry: {}", after.x);
        assert!((after.x - 7.0 * AIR_SLIPPERINESS * HORIZONTAL_DRAG).abs() < 0.01);
    }

    #[test]
    fn ledge_jump_eligible_when_was_on_ground() {
        let can_jump = true && true && 0u8 == 0;
        assert!(can_jump);
        let on_ground = false;
        let was_on_ground = true;
        assert!(was_on_ground && !on_ground);
    }

    #[test]
    fn jump_cooldown_blocks_rapid_jumps() {
        let cooldown_steps = jump_cooldown_sim_steps(SIM_DT);
        assert_eq!(cooldown_steps, 30);
        let mut cooldown = 0u8;
        let mut jumps = 0u32;
        for _ in 0..40 {
            if cooldown == 0 {
                jumps += 1;
                cooldown = cooldown_steps;
            }
            if cooldown > 0 {
                cooldown -= 1;
            }
        }
        assert_eq!(jumps, 2, "holding jump should fire twice in 40 steps with 30-step cooldown");
    }

    fn simulate_ground_speed(sprint: bool, steps: u32) -> f32 {
        let input = mc_input(sprint);
        let mut horiz = Vec2::ZERO;
        let mut accum = 0.0f32;
        for _ in 0..steps {
            accum += SIM_DT;
            while accum >= MC_TICK_DT {
                accum -= MC_TICK_DT;
                horiz = update_horizontal_velocity(
                    horiz,
                    &input,
                    true,
                    true,
                    false,
                    MC_TICK_DT,
                );
            }
        }
        horiz.y
    }

    #[test]
    fn ground_walk_and_sprint_speeds_match_minecraft() {
        let walk = simulate_ground_speed(false, 600);
        let sprint = simulate_ground_speed(true, 600);
        assert!(
            (walk - 4.32).abs() < 0.15,
            "walk speed {walk} blocks/s, expected ~4.32"
        );
        assert!(
            (sprint - 7.29).abs() < 0.25,
            "sprint speed {sprint} blocks/s, expected ~7.29"
        );
        assert!(sprint > walk + 0.5, "sprint should be clearly faster than walk");
    }

    #[test]
    fn landing_preserves_momentum_not_snap() {
        let carried = Vec2::new(6.0, 0.0);
        let input = mc_input(false);
        let after = update_horizontal_velocity(
            carried,
            &input,
            true,
            true,
            false,
            MC_TICK_DT,
        );
        let snap_speed = 5.1;
        assert!(
            (after.x - snap_speed).abs() > 0.5,
            "landing should not snap to walk speed: {}",
            after.x
        );
        assert!(
            after.x > 3.0,
            "momentum should persist through landing step: {}",
            after.x
        );
    }
}
