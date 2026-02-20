use bevy::{
    color::palettes::css::RED,
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, camera_controls)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let red: Color = RED.into();

    // 3D Sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: red,
            ..default()
        })),
        Transform::from_xyz(0., 50., 0.),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(200.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        })),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 120.0, 250.0).looking_at(Vec3::new(0.0, 100.0, 0.0), Vec3::Y),
        CameraController,
    ));

    // Light with shadow cascade configuration
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
        CascadeShadowConfigBuilder {
            maximum_distance: 500.0,
            first_cascade_far_bound: 50.0,
            num_cascades: 4,
            ..default()
        }
        .build(),
    ));
}

#[derive(Component)]
struct CameraController;

fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CameraController>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let speed = 200.0 * time.delta_secs();
    let rot_speed = 2.0 * time.delta_secs();

    let forward = transform.forward();
    let right = transform.right();

    if keyboard.pressed(KeyCode::KeyW) {
        transform.translation += forward * speed;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        transform.translation -= forward * speed;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        transform.translation -= right * speed;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        transform.translation += right * speed;
    }
    if keyboard.pressed(KeyCode::KeyQ) {
        transform.translation.y -= speed;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        transform.translation.y += speed;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        transform.rotate_y(rot_speed);
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        transform.rotate_y(-rot_speed);
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        transform.rotate_local_x(rot_speed);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        transform.rotate_local_x(-rot_speed);
    }
}
