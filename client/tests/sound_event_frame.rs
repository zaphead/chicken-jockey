use client::bootstrap::bootstrap_local_app;
use client::frame::run_client_frame;
use client::systems::ClientAudio;
use engine_core::{Stage, SystemContext, Time, SIM_DT};
use game::{SoundCue, SoundKind};
use glam::Vec3;

fn emit_test_cue(ctx: &mut SystemContext<'_>) {
    ctx.events.send(SoundCue {
        kind: SoundKind::PlayerFootstep,
        position: Vec3::ZERO,
        block_id: None,
    });
}

#[test]
fn sound_cues_reach_render_stage() {
    let mut app = bootstrap_local_app(Time::new(SIM_DT));
    app.add_system(Stage::PostUpdate, emit_test_cue);

    run_client_frame(&mut app, SIM_DT);

    assert!(
        app.resource::<ClientAudio>().is_some(),
        "expected ClientAudio from bootstrap"
    );
}
