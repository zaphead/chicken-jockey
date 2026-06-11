use std::collections::HashMap;

use engine_assets::{AttenuationToml, BlockRegistry, SoundRegistry};
use engine_audio::{Attenuation, AudioEngine, Listener, PlayRequest, SoundCategory, SoundClip};
use engine_core::SystemContext;
use engine_render::RenderWorld;
use game::SoundCue;

pub struct SoundBank {
    cache: HashMap<String, SoundClip>,
}

impl Default for SoundBank {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl SoundBank {
    pub fn clip<'a>(&'a mut self, registry: &SoundRegistry, path: &str) -> Option<&'a SoundClip> {
        if !self.cache.contains_key(path) {
            let bytes = registry.clip_bytes_for(path)?;
            let clip = SoundClip::from_ogg_bytes(bytes).ok()?;
            self.cache.insert(path.to_string(), clip);
        }
        self.cache.get(path)
    }
}

pub struct ClientAudio {
    pub engine: AudioEngine,
    pub bank: SoundBank,
}

pub fn audio_feedback_system(ctx: &mut SystemContext<'_>) {
    let cues: Vec<SoundCue> = ctx.events.drain::<SoundCue>();
    let listener = ctx
        .resources
        .get::<RenderWorld>()
        .map(|render_world| Listener {
            position: render_world.camera.position,
            forward: render_world.camera.forward(),
            up: glam::Vec3::Z,
        });

    if cues.is_empty() {
        if let Some(audio) = ctx.resources.get_mut::<ClientAudio>() {
            if let Some(listener) = listener {
                audio.engine.set_listener(listener);
            }
            audio.engine.tick();
        }
        return;
    }

    let Some(registry) = ctx.resources.get::<BlockRegistry>().cloned() else {
        return;
    };
    let Some(sound_registry) = ctx.resources.get::<SoundRegistry>().cloned() else {
        return;
    };
    let Some(audio) = ctx.resources.get_mut::<ClientAudio>() else {
        return;
    };

    if let Some(listener) = listener {
        audio.engine.set_listener(listener);
    }

    for cue in cues {
        play_cue(
            &mut audio.engine,
            &mut audio.bank,
            &sound_registry,
            &registry,
            cue,
        );
    }
    audio.engine.tick();
}

fn play_cue(
    engine: &mut AudioEngine,
    bank: &mut SoundBank,
    sound_registry: &SoundRegistry,
    block_registry: &BlockRegistry,
    cue: SoundCue,
) {
    let sound_group = cue
        .block_id
        .map(|id| block_registry.sound_group(id).to_string())
        .or_else(|| cue.kind.manifest_sound_group().map(str::to_string));

    let event = sound_registry
        .resolve(cue.kind.manifest_key(), sound_group.as_deref())
        .or_else(|| {
            sound_registry.resolve(
                cue.kind.manifest_key(),
                cue.kind.manifest_sound_group(),
            )
        });
    let Some(event) = event else {
        return;
    };
    let Some((clip_path, pitch)) = sound_registry.pick_clip_path(event) else {
        return;
    };
    let Some(clip) = bank.clip(sound_registry, clip_path) else {
        return;
    };

    let category = match event.category.as_str() {
        "player" => SoundCategory::Player,
        _ => SoundCategory::Blocks,
    };
    let attenuation = match event.attenuation {
        AttenuationToml::None => Attenuation::None,
        AttenuationToml::Linear => Attenuation::Linear {
            max_distance: event.max_distance,
        },
    };

    engine.play(PlayRequest {
        clip: clip.clone(),
        position: cue.position,
        volume: event.volume,
        pitch,
        category,
        attenuation,
    });
}
