use std::{cmp::Ordering, f32::consts::PI, time::Duration};

use avian2d::{PhysicsPlugins, math::Vector, prelude::*};
use bevy::{
    color::palettes::css::{GREEN, PURPLE, RED},
    math::VectorSpace,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    reflect::hash_error,
    render::render_resource::{AsBindGroup, Face, ShaderRef},
    scene::SceneInstance,
    time::common_conditions::on_timer,
};
use bevy_egui::{EguiContextPass, EguiContexts, egui};
use bevy_fly_camera::FlyCamera;
use bevy_tweening::{Animator, Delay, Tween, TweenCompleted};

const FIRE_SHADER_PATH: &str = "shaders/rocket_fire.wgsl";
const NOF_PARTICLES: usize = 20;
const PARTICLE_SPEED: f32 = 50.;

pub struct RocketFirePlugin;

impl Plugin for RocketFirePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>::default(),
            PhysicsPlugins::default(),
        ))
        .add_systems(EguiContextPass, build_sliders)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, on_params_changed.run_if(resource_changed::<Params>))
        .add_systems(Update, customize_scene_materials)
        .add_systems(Update, set_shader_params)
        .add_systems(
            Update,
            spawn_particle.run_if(on_timer(Duration::from_millis(20))),
        )
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
}

impl Default for FireMaterialExtension {
    fn default() -> Self {
        Self {
            color: Vec4::new(0., 0.0, 0.0, 0.0),
            center: Vec4::new(0.0, 0.0, 0.0, 0.0),
            nof_particles: UVec4::new(NOF_PARTICLES as u32, 0, 0, 0),
            particles: [Vec4::ZERO; 32],
            dir: Vec3::Y.extend(0.0),
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
    color: Vec3,
    center: Vec3,
    size: f32,

    particle_speed: f32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            rocket_rotation: 0.0,
            color: Vec3::new(1., 0., 0.),
            center: Vec3::new(0., 0., 0.),
            size: 0.1,
            particle_speed: 50.0,
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
    mut fire_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>>,
    fire_material: Query<
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>,
        With<Fire>,
    >,
    mut rocket_transform: Single<&mut Transform, With<Rocket>>,
) {
    let Ok(material) = fire_material.single() else {
        return;
    };
    let Some(material) = fire_materials.get_mut(material.id()) else {
        return;
    };

    let color = RED.lerp(PURPLE, params.particle_speed / 100.);
    material.extension.color = color.to_vec4();

    rocket_transform.rotation = Quat::from_rotation_z(PI * params.rocket_rotation);
}

fn build_sliders(mut contexts: EguiContexts, mut params: ResMut<Params>, mut commands: Commands) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut params.rocket_rotation, 0.0..=2.0).text("rotation"));
        ui.add(egui::Slider::new(&mut params.particle_speed, 10.0..=100.0).text("power"));
        // ui.add(egui::Slider::new(&mut params.progress, 0.0..=1.0).text("progress"));
        // ui.add(egui::Slider::new(&mut params.radius, 1.0..=50.0).text("radius"));
    });
}

fn customize_scene_materials(
    unloaded_instances: Query<(Entity, &SceneInstance, &CustomizeMaterial)>,
    handles: Query<Entity, (With<MeshMaterial3d<StandardMaterial>>, Without<Rocket>)>,
    // pbr_materials: Res<Assets<StandardMaterial>>,
    scene_manager: Res<SceneSpawner>,
    // mut custom_materials: ResMut<Assets<StandardMaterial>>,
    // mut custom_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>>,
    mut cmds: Commands,
) {
    for (entity, instance, customize_material) in unloaded_instances.iter() {
        if scene_manager.instance_is_ready(**instance) {
            cmds.entity(entity).remove::<CustomizeMaterial>();
        }

        // Iterate over all entities in scene (once it's loaded)
        let handles = handles.iter_many(scene_manager.iter_instance_entities(**instance));
        for entity in handles {
            // let Some(material) = pbr_materials.get(material_handle) else {
            //     continue;
            // };

            cmds.entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .insert(MeshMaterial3d(customize_material.material.clone()));

            // let custom = custom_materials.add(material.into());
            // cmds.entity(entity)
            //     .insert(custom)
            //     .remove::<Handle<StandardMaterial>>();
        }
    }
}

fn spawn_particle(
    mut commands: Commands,
    rocket_transform: Single<&Transform, With<Rocket>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_id: Local<usize>,
    params: Res<Params>,
    particles: Query<(Entity, &Particle)>,
) {
    let particles = particles.iter().collect::<Vec<(Entity, &Particle)>>();
    if particles.len() > NOF_PARTICLES {
        let mut oldest_partile_entity: Option<Entity> = None;
        let mut oldest_id = usize::MAX;
        for (entity, particle) in particles {
            if particle.id < oldest_id {
                oldest_partile_entity = Some(entity);
                oldest_id = particle.id;
            }
        }

        if let Some(oldest_partile_entity) = oldest_partile_entity {
            commands.entity(oldest_partile_entity).despawn();
        }
    }

    commands.spawn((
        RigidBody::Kinematic,
        LinearVelocity(-PARTICLE_SPEED * rocket_transform.rotation.mul_vec3(Vec3::Y).xy()),
        Transform::from_translation(
            rocket_transform.translation
                - rocket_transform.rotation.mul_vec3(Vec3::Y) * params.particle_speed * 0.02,
        ),
        // Mesh3d(meshes.add(Sphere::new(0.2))),
        // MeshMaterial3d(materials.add(StandardMaterial {
        //     base_color: RED.into(),
        //     ..Default::default()
        // })),
        Particle { id: *last_id },
    ));

    *last_id += 1;
}

fn set_shader_params(
    particles: Query<(&Transform, &Particle)>,
    rocket_transform: Single<&Transform, With<Rocket>>,
    mut fire_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>>,
    fire_material: Single<
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, FireMaterialExtension>>,
        With<Fire>,
    >,
) {
    let Some(fire_material) = fire_materials.get_mut(fire_material.id()) else {
        return;
    };

    let mut particles = particles.iter().collect::<Vec<(&Transform, &Particle)>>();
    particles.sort_by(|(_, a), (_, b)| b.id.cmp(&a.id));
    for (i, (transform, _)) in particles.iter().enumerate() {
        fire_material.extension.particles[i] = transform.translation.extend(0.0);
    }

    fire_material.extension.nof_particles = UVec4::new(particles.iter().len() as u32, 0, 0, 0);

    fire_material.extension.dir = rocket_transform.rotation.mul_vec3(Vec3::Y).extend(0.0);

    fire_material.extension.center = rocket_transform.translation.extend(0.0);
}
