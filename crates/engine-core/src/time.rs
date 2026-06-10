/// Authoritative simulation rate (Hz).
pub const SIM_HZ: f32 = 60.0;
/// Fixed simulation step duration (seconds).
pub const SIM_DT: f32 = 1.0 / SIM_HZ;
/// Maximum fixed sim steps per render frame (spiral-of-death guard).
pub const MAX_SIM_STEPS_PER_FRAME: u32 = 8;
/// Maximum wall-clock frame delta accepted into the accumulator (seconds).
pub const MAX_FRAME_DELTA: f32 = 0.25;

/// Frame and fixed-timestep timing resource.
#[derive(Debug, Clone, Copy)]
pub struct Time {
    pub elapsed: f64,
    pub fixed_delta: f32,
    pub sim_tick: u64,
    pub frame_delta: f32,
    pub accumulator_secs: f32,
    pub interpolation_alpha: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            fixed_delta: SIM_DT,
            sim_tick: 0,
            frame_delta: 0.0,
            accumulator_secs: 0.0,
            interpolation_alpha: 0.0,
        }
    }
}

impl Time {
    pub fn new(fixed_delta: f32) -> Self {
        Self {
            fixed_delta,
            ..Default::default()
        }
    }

    pub fn advance_fixed(&mut self) {
        self.elapsed += f64::from(self.fixed_delta);
        self.sim_tick += 1;
    }

    pub fn add_frame_delta(&mut self, delta: f32) {
        let clamped = delta.min(MAX_FRAME_DELTA);
        self.frame_delta = clamped;
        self.accumulator_secs += clamped;
    }

    pub fn drain_sim_steps(&mut self, max_steps: u32) -> u32 {
        let mut steps = 0;
        while steps < max_steps && self.accumulator_secs >= self.fixed_delta {
            self.accumulator_secs -= self.fixed_delta;
            steps += 1;
        }
        steps
    }

    pub fn set_interpolation_alpha(&mut self) {
        if self.fixed_delta > 0.0 {
            self.interpolation_alpha = (self.accumulator_secs / self.fixed_delta).clamp(0.0, 1.0);
        } else {
            self.interpolation_alpha = 0.0;
        }
    }

    pub fn runs_on_divisor(self, n: u64) -> bool {
        n > 0 && self.sim_tick % n == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_fixed_increments_sim_tick() {
        let mut time = Time::new(SIM_DT);
        time.advance_fixed();
        assert_eq!(time.fixed_delta, SIM_DT);
        assert_eq!(time.sim_tick, 1);
    }

    #[test]
    fn drain_sim_steps_one_dt_yields_one_step() {
        let mut time = Time::new(SIM_DT);
        time.add_frame_delta(SIM_DT);
        assert_eq!(time.drain_sim_steps(MAX_SIM_STEPS_PER_FRAME), 1);
        assert!(time.accumulator_secs < SIM_DT);
    }

    #[test]
    fn drain_sim_steps_two_dt_yields_two_steps() {
        let mut time = Time::new(SIM_DT);
        time.add_frame_delta(SIM_DT * 2.0);
        assert_eq!(time.drain_sim_steps(MAX_SIM_STEPS_PER_FRAME), 2);
    }

    #[test]
    fn drain_sim_steps_respects_max_steps() {
        let mut time = Time::new(SIM_DT);
        time.add_frame_delta(1.0);
        assert_eq!(
            time.drain_sim_steps(MAX_SIM_STEPS_PER_FRAME),
            MAX_SIM_STEPS_PER_FRAME
        );
        assert!(time.accumulator_secs >= SIM_DT);
    }

    #[test]
    fn interpolation_alpha_is_remainder_over_dt() {
        let mut time = Time::new(SIM_DT);
        time.add_frame_delta(0.025);
        let steps = time.drain_sim_steps(MAX_SIM_STEPS_PER_FRAME);
        assert_eq!(steps, 1);
        time.set_interpolation_alpha();
        let expected = time.accumulator_secs / SIM_DT;
        assert!((time.interpolation_alpha - expected).abs() < 1e-6);
    }

    #[test]
    fn runs_on_divisor() {
        let mut time = Time::new(SIM_DT);
        time.advance_fixed();
        assert!(!time.runs_on_divisor(3));
        time.advance_fixed();
        time.advance_fixed();
        assert!(time.runs_on_divisor(3));
    }
}
