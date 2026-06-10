/// Frame and fixed-timestep timing resource.
#[derive(Debug, Clone, Copy, Default)]
pub struct Time {
    pub delta: f32,
    pub elapsed: f64,
    pub fixed_delta: f32,
    pub tick: u64,
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
        self.delta = self.fixed_delta;
        self.tick += 1;
    }

    pub fn advance_variable(&mut self, delta: f32) {
        self.elapsed += f64::from(delta);
        self.delta = delta;
        self.tick += 1;
    }
}
