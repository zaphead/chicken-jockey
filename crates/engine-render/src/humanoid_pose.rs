use std::f32::consts::PI;

use glam::{Mat4, Vec3};

use crate::player_model::{
    HUMANOID_PART_BODY, HUMANOID_PART_COUNT, HUMANOID_PART_HEAD, HUMANOID_PART_LEFT_ARM,
    HUMANOID_PART_LEFT_LEG, HUMANOID_PART_RIGHT_ARM, HUMANOID_PART_RIGHT_LEG,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct HumanoidPose {
    pub head_rot: Vec3,
    pub body_rot: Vec3,
    pub right_arm_rot: Vec3,
    pub left_arm_rot: Vec3,
    pub right_leg_rot: Vec3,
    pub left_leg_rot: Vec3,
}

impl HumanoidPose {
    pub fn part_rotations(self) -> [Vec3; HUMANOID_PART_COUNT] {
        let mut parts = [Vec3::ZERO; HUMANOID_PART_COUNT];
        parts[HUMANOID_PART_HEAD] = self.head_rot;
        parts[HUMANOID_PART_BODY] = self.body_rot;
        parts[HUMANOID_PART_RIGHT_ARM] = self.right_arm_rot;
        parts[HUMANOID_PART_LEFT_ARM] = self.left_arm_rot;
        parts[HUMANOID_PART_RIGHT_LEG] = self.right_leg_rot;
        parts[HUMANOID_PART_LEFT_LEG] = self.left_leg_rot;
        parts
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerAnimationParams {
    pub limb_swing: f32,
    pub limb_swing_amount: f32,
    pub head_pitch: f32,
    /// Head yaw relative to the torso (Z-up → rotation around Z).
    pub head_yaw: f32,
    /// Blend weight for the right-arm digging pose (0 = walk, 1 = dig).
    pub dig_amount: f32,
    /// Phase of the digging oval swing (radians).
    pub dig_phase: f32,
    /// Blend weight for the right-arm place pose (0 = walk, 1 = place swing).
    pub place_amount: f32,
    /// Phase of the place oval swing (radians).
    pub place_phase: f32,
}

fn right_arm_walk_rotation(swing: f32, amount: f32) -> f32 {
    (swing + PI).cos() * 2.0 * amount * 0.5
}

/// Right-arm digging swing: oval motion (up/down on X, side-to-side on Y).
fn right_arm_dig_rotation(phase: f32) -> Vec3 {
    const BASE_FORWARD: f32 = 1.15;
    const UP_DOWN_AMP: f32 = 0.55;
    const SIDE_AMP: f32 = 0.1;
    let t = -phase;
    Vec3::new(
        BASE_FORWARD - t.sin() * UP_DOWN_AMP,
        -t.cos() * SIDE_AMP,
        0.0,
    )
}

/// Minecraft `HumanoidModel` walk cycle (limb swing + head look).
pub fn humanoid_pose_from_animation(params: PlayerAnimationParams) -> HumanoidPose {
    let swing = params.limb_swing * 0.6662;
    let amount = params.limb_swing_amount;
    let arm_scale = 2.0 * amount * 0.5;
    let leg_scale = 1.4 * amount;

    let walk_right_arm = Vec3::new(right_arm_walk_rotation(swing, amount), 0.0, 0.0);
    let dig_t = params.dig_amount.clamp(0.0, 1.0);
    let place_t = params.place_amount.clamp(0.0, 1.0);
    let right_arm_rot = if dig_t > 0.0 {
        walk_right_arm.lerp(right_arm_dig_rotation(params.dig_phase), dig_t)
    } else {
        walk_right_arm.lerp(right_arm_dig_rotation(params.place_phase), place_t)
    };

    HumanoidPose {
        head_rot: Vec3::new(params.head_pitch, 0.0, params.head_yaw),
        body_rot: Vec3::ZERO,
        right_arm_rot,
        left_arm_rot: Vec3::new(swing.cos() * arm_scale, 0.0, 0.0),
        right_leg_rot: Vec3::new((swing + PI).cos() * leg_scale, 0.0, 0.0),
        left_leg_rot: Vec3::new(swing.cos() * leg_scale, 0.0, 0.0),
    }
}

/// Part transform in model space (feet at origin). Mesh vertices are pivot-local.
pub fn part_local_matrix(pivot: Vec3, rotation: Vec3) -> Mat4 {
    Mat4::from_translation(pivot)
        * Mat4::from_rotation_x(rotation.x)
        * Mat4::from_rotation_y(rotation.y)
        * Mat4::from_rotation_z(rotation.z)
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerRender {
    pub base: Mat4,
    pub pose: HumanoidPose,
    /// Bit `i` set → draw humanoid part `i`.
    pub part_mask: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_pose_swings_arms_opposite() {
        let pose = humanoid_pose_from_animation(PlayerAnimationParams {
            limb_swing: 1.0,
            limb_swing_amount: 1.0,
            head_pitch: 0.0,
            head_yaw: 0.0,
            ..Default::default()
        });
        assert!(
            (pose.right_arm_rot.x + pose.left_arm_rot.x).abs() < 0.01,
            "arms should swing in opposition"
        );
    }

    #[test]
    fn dig_pose_only_moves_right_arm() {
        let walk = humanoid_pose_from_animation(PlayerAnimationParams {
            limb_swing: 0.5,
            limb_swing_amount: 1.0,
            ..Default::default()
        });
        let dig = humanoid_pose_from_animation(PlayerAnimationParams {
            limb_swing: 0.5,
            limb_swing_amount: 1.0,
            dig_amount: 1.0,
            dig_phase: PI / 2.0,
            ..Default::default()
        });
        assert_ne!(dig.right_arm_rot, walk.right_arm_rot);
        assert_eq!(dig.left_arm_rot, walk.left_arm_rot);
    }
}
