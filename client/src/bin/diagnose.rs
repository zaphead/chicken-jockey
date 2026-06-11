//! Headless client pipeline diagnostic — no window required.
use client::bootstrap::bootstrap_local_app;
use client::diagnostics::ClientDiagnostics;
use client::frame::run_client_frame;
use engine_core::{Time, SIM_DT};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut app = bootstrap_local_app(Time::new(SIM_DT));
    let mut last_mesh_count = 0usize;

    for frame in 1..=300 {
        run_client_frame(&mut app, SIM_DT);

        last_mesh_count = app
            .resource::<engine_render::RenderWorld>()
            .map(|world| world.meshes().len())
            .unwrap_or(0);

        if frame == 1 || frame % 60 == 0 || frame == 300 {
            let mut diag = ClientDiagnostics::sample(&app, false, last_mesh_count);
            diag.frame = frame as u64;
            log::info!("oc diag: {}", diag.log_line());
        }
    }

    let diag = ClientDiagnostics::sample(&app, false, last_mesh_count);
    println!("{}", diag.log_line());
    if diag.is_healthy() {
        println!("DIAGNOSE_OK");
        std::process::exit(0);
    }
    eprintln!("DIAGNOSE_FAIL");
    std::process::exit(1);
}
