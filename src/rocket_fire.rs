use std::{cmp::Ordering, collections::VecDeque, f32::consts::PI, time::Duration};

use avian2d::{PhysicsPlugins, math::Vector, prelude::*};
use bevy::{
    color::palettes::css::{GREEN, PURPLE, RED},
    math::VectorSpace,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    reflect::{List, hash_error},
    render::render_resource::{AsBindGroup, Face, ShaderRef},
    scene::SceneInstance,
    time::common_conditions::on_timer,
};
use bevy_egui::{EguiContextPass, EguiContexts, egui};
use bevy_fly_camera::FlyCamera;
use bevy_tweening::{Animator, Delay, Tween, TweenCompleted};

const FIRE_SHADER_PATH: &str = "shaders/rocket_fire.wgsl";
const NOF_PARTICLES: usize = 20;
const PARTICLE_SPEED: f32 = 40.;

pub struct RocketFirePlugin;

impl Plugin for RocketFirePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, FireMaterialExtension>,
        >::default(),))
            .add_systems(EguiContextPass, build_sliders)
            .add_systems(Startup, setup_scene)
            .add_systems(Update, on_params_changed.run_if(resource_changed::<Params>))
            .add_systems(Update, customize_scene_materials)
            .add_systems(Update, set_shader_params)
            .init_resource::<Params>();
    }
}

#[derive(Component)]
struct Particle {
    id: usize,
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct FireMaterialExtension {
    #[uniform(100)]
    color: Vec4,
    #[uniform(101)]
    center: Vec4,
    #[uniform(102)]
    nof_particles: UVec4,

    #[uniform(103)]
    particles: [Vec4; 32],

    #[uniform(104)]
    dir: Vec4,

    #[uniform(105)]
    power: Vec4,
}

impl Default for FireMaterialExtension {
    fn default() -> Self {
        Self {
            color: Vec4::new(0., 0.0, 0.0, 0.0),
            center: Vec4::new(0.0, 0.0, 0.0, 0.0),
            nof_particles: UVec4::new(NOF_PARTICLES as u32, 0, 0, 0),
            particles: [Vec4::ZERO; 32],
            dir: Vec3::Y.extend(0.0),
            power: Vec4::new(1.0, 0., 0., 0.),
        }
    }
}

impl MaterialExtension for FireMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        FIRE_SHADER_PATH.into()
    }

    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Blend)
    }
}

#[derive(Component)]
struct Rocket {}

#[derive(Component)]
struct Fire {}

#[derive(Component)]
struct CustomizeMaterial {
    material: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct Params {
    rocket_rotation: f32,
    power: f32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            rocket_rotation: 0.0,
            power: 0.1,
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut explosion_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>,
    >,
    asset_server: Res<AssetServer>,
) {
    let rocket_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/Ship1.glb"));

    commands.spawn((
        Rocket {},
        SceneRoot(rocket_model),
        Transform::from_translation(Vec3::new(-5.0, 0.0, 0.0)).with_scale(Vec3::ONE * 1.0),
        // CustomizeMaterial {
        //     material: materials.add(StandardMaterial {
        //         base_color: GREEN.into(),
        //         ..Default::default()
        //     }),
        // },
        children![(
            Fire {},
            Transform::from_translation(-Vec3::Y * 0.),
            // Mesh3d(meshes.add(Sphere::new(10.0))),
            Mesh3d(meshes.add(Rectangle::from_length(200.))),
            MeshMaterial3d(explosion_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                },
                extension: FireMaterialExtension { ..default() },
            })),
        )],
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
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

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 40.0).looking_at(Vec3::ZERO, Dir3::Y),
        FlyCamera::default(),
    ));
}

fn on_params_changed(
    params: Res<Params>,
    mut rocket_transform: Single<&mut Transform, With<Rocket>>,
) {
    rocket_transform.rotation = Quat::from_rotation_z(PI * params.rocket_rotation);
}

fn build_sliders(mut contexts: EguiContexts, mut params: ResMut<Params>, mut commands: Commands) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut params.rocket_rotation, 0.0..=2.0).text("rotation"));
        ui.add(egui::Slider::new(&mut params.power, 0.0..=1.0).text("power"));
        // ui.add(egui::Slider::new(&mut params.progress, 0.0..=1.0).text("progress"));
        // ui.add(egui::Slider::new(&mut params.radius, 1.0..=50.0).text("radius"));
    });
}

fn customize_scene_materials(
    unloaded_instances: Query<(Entity, &SceneInstance, &CustomizeMaterial)>,
    handles: Query<Entity, (With<MeshMaterial3d<StandardMaterial>>, Without<Rocket>)>,
    scene_manager: Res<SceneSpawner>,
    mut cmds: Commands,
) {
    for (entity, instance, customize_material) in unloaded_instances.iter() {
        if scene_manager.instance_is_ready(**instance) {
            cmds.entity(entity).remove::<CustomizeMaterial>();
        }

        let handles = handles.iter_many(scene_manager.iter_instance_entities(**instance));
        for entity in handles {
            cmds.entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .insert(MeshMaterial3d(customize_material.material.clone()));
        }
    }
}

fn set_shader_params(
    rocket_transform: Single<&Transform, With<Rocket>>,
    mut fire_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>>,
    fire_material: Single<
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>,
        With<Fire>,
    >,
    params: Res<Params>,

    mut particles_queue: Local<VecDeque<(Vec2, Vec2)>>,
    mut last_particle_spawned: Local<u128>,
    time: Res<Time>,
) {
    let Some(fire_material) = fire_materials.get_mut(fire_material.id()) else {
        return;
    };

    let flame_dir = rocket_transform.rotation.mul_vec3(-Vec3::Y);
    let curr_time = time.elapsed().as_millis();
    if curr_time - *last_particle_spawned > 50 {
        particles_queue.push_front((rocket_transform.translation.xy(), flame_dir.xy()));

        if particles_queue.len() >= NOF_PARTICLES {
            particles_queue.pop_back();
        }

        *last_particle_spawned = curr_time;
    }

    for (pos, flame_dir) in &mut particles_queue {
        *pos += *flame_dir * 0.4 * params.power;
    }

    for (i, (pos, _)) in particles_queue.iter().enumerate() {
        fire_material.extension.particles[i] = pos.extend(0.0).extend(0.0);
    }

    fire_material.extension.nof_particles = UVec4::new(particles_queue.len() as u32, 0, 0, 0);

    fire_material.extension.dir = -flame_dir.extend(0.0);

    fire_material.extension.center = rocket_transform.translation.extend(0.0);

    let color = RED.lerp(PURPLE, params.power);
    fire_material.extension.color = color.to_vec4();

    fire_material.extension.power = Vec4::splat(params.power);
}
