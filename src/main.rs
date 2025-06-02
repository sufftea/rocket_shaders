use bevy::{
    color::palettes::css::{GREEN, RED},
    core_pipeline::prepass::DepthPrepass,
    math::VectorSpace,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

const EXPLOSION_SHADER_PATH: &str = "shaders/explosion.wgsl";

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        FlyCameraPlugin,
        MaterialPlugin::<ExtendedMaterial<StandardMaterial, ExplosionMaterialExtension>>::default(),
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
    ))
    .add_systems(EguiContextPass, build_sliders)
    .add_systems(Startup, setup_scene)
    .add_systems(Update, handle_input)
    .add_systems(Update, on_params_changed.run_if(resource_changed::<Params>))
    .add_observer(on_spawn_explosion)
    .init_resource::<Params>();

    app.run();
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct ExplosionMaterialExtension {
    #[uniform(100)]
    progress: Vec4,
    #[uniform(101)]
    center: Vec4,
    #[uniform(102)]
    radius: Vec4,
}

impl Default for ExplosionMaterialExtension {
    fn default() -> Self {
        Self {
            progress: Vec4::new(0., 0.0, 0.0, 0.0),
            radius: Vec4::new(20., 0.0, 0.0, 0.0),
            center: Vec4::ZERO,
        }
    }
}

impl MaterialExtension for ExplosionMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        EXPLOSION_SHADER_PATH.into()
    }

    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Blend)
    }
}

#[derive(Event)]
struct MeteoriteExplosion {
    position: Vec2,
}

#[derive(Component)]
struct Meteorite {}

#[derive(Component)]
struct Explosion {}

#[derive(Resource)]
struct Params {
    progress: f32,
    radius: f32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            progress: 0.0,
            radius: 20.,
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Meteorite {},
        Mesh3d(meshes.add(Sphere::new(2.0))),
        Transform::from_xyz(0., 0., 0.),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.into(),
            ..Default::default()
        })),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(6.0))),
        Transform::from_xyz(5., 30., 10.),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GREEN.into(),
            ..Default::default()
        })),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(10.0))),
        Transform::from_xyz(20., 0., 0.),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GREEN.into(),
            ..Default::default()
        })),
    ));

    commands.spawn((
        DirectionalLight { ..default() },
        Transform::default().looking_at(Vec3::new(1.0, 0.0, 0.0), Dir3::Y),
    ));

    // commands.trigger(MeteoriteExplosion {
    //     position: Vec2::ZERO,
    // });

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 40.0).looking_at(Vec3::ZERO, Dir3::Y),
        FlyCamera::default(),
    ));
}

fn on_params_changed(
    params: Res<Params>,
    mut explosion_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, ExplosionMaterialExtension>>,
    >,
    materials: Query<
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, ExplosionMaterialExtension>>,
        With<Explosion>,
    >,
) {
    let Ok(material) = materials.single() else {
        return;
    };
    let Some(material) = explosion_materials.get_mut(material.id()) else {
        return;
    };

    material.extension.progress = Vec4::new(params.progress, 0.0, 0.0, 0.0);
    material.extension.radius =  Vec4::new(params.radius, 0.0, 0.0, 0.0);
}

fn build_sliders(mut contexts: EguiContexts, mut params: ResMut<Params>, mut commands: Commands) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut params.progress, 0.0..=1.0).text("progress"));
        ui.add(egui::Slider::new(&mut params.radius, 1.0..=50.0).text("radius"));

        if ui.button("Spawn explosion").clicked() {
            println!("spawn explosion clicked");
            commands.trigger(MeteoriteExplosion {
                position: Vec2::ZERO,
            });
        }
        // ui.add(egui::Button::new("spawn explosion"));

        // if ui.button("Increment").clicked() {
        //     ui_state.value += 1.0;
        // }
    });
}

fn on_spawn_explosion(
    trigger: Trigger<MeteoriteExplosion>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, ExplosionMaterialExtension>>,
    >,
) {
    commands.spawn((
        Explosion {},
        Transform::from_translation(trigger.position.extend(0.)),
        Mesh3d(meshes.add(Sphere::new(20.0))),
        MeshMaterial3d(explosion_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
            extension: ExplosionMaterialExtension {
                center: trigger.position.extend(0.0).extend(0.0),
                ..default()
            },
        })),
    ));
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
