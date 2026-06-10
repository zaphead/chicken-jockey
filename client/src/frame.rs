use engine_core::{App, Stage, Time, MAX_SIM_STEPS_PER_FRAME};

/// One client render frame: PreUpdate once, fixed sim drain, then Extract + Render.
pub fn run_client_frame(app: &mut App, frame_delta: f32) {
    if let Some(time) = app.resource_mut::<Time>() {
        time.add_frame_delta(frame_delta);
    }

    app.run_stage(Stage::PreUpdate);

    let steps = app
        .resource_mut::<Time>()
        .map(|time| time.drain_sim_steps(MAX_SIM_STEPS_PER_FRAME))
        .unwrap_or(0);

    for _ in 0..steps {
        if let Some(time) = app.resource_mut::<Time>() {
            time.advance_fixed();
        }
        app.tick_fixed_step();
        app.end_frame();
    }

    if let Some(time) = app.resource_mut::<Time>() {
        time.set_interpolation_alpha();
    }

    app.tick_render();
}
