use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::EguiPlugin;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

mod explosion;
mod rocket_fire;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        FlyCameraPlugin,
        FrameTimeDiagnosticsPlugin::default(),
        LogDiagnosticsPlugin::default(),
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        // explosion::ExplosionPlugin {},
        rocket_fire::RocketFirePlugin {},
    ))
    .add_systems(Update, handle_input);

    app.run();
}

fn handle_input(input: Res<ButtonInput<KeyCode>>, mut flycam: Single<&mut FlyCamera>) {
    // if input.just_pressed(KeyCode::KeyB) {
    //     if let Ok(transform) = meteorite.single() {
    //         println!("triggering the explosion");
    //         commands.trigger(MeteoriteExplosion {
    //             position: transform.translation.xy(),
    //         });
    //     }
    // }

    if input.just_pressed(KeyCode::KeyC) {
        flycam.enabled = !flycam.enabled;
        println!("camera enabled: {}", flycam.enabled);
    }

    // for event in keyboard_events.read() {
    //     if event.key_code == KeyCode::KeyB {
    //         if let Ok(transform) = meteorite.single() {
    //             println!("triggering the explosion");
    //             commands.trigger(MeteoriteExplosion {
    //                 position: transform.translation.xy(),
    //             });
    //         }
    //     }
    // }
}
