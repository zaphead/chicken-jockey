use hecs::World;

/// Deferred ECS mutations flushed at the end of `PostUpdate`.
pub struct Commands {
    queue: Vec<Box<dyn FnOnce(&mut World) + Send>>,
}

impl Default for Commands {
    fn default() -> Self {
        Self { queue: Vec::new() }
    }
}

impl Commands {
    pub fn push(&mut self, command: impl FnOnce(&mut World) + Send + 'static) {
        self.queue.push(Box::new(command));
    }

    pub fn flush(&mut self, world: &mut World) {
        let queue = std::mem::take(&mut self.queue);
        for command in queue {
            command(world);
        }
    }
}
