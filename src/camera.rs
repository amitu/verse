use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DirectionalLightShadowMap { size: 4096 });
        app.insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            ..default()
        });
        app.add_systems(Startup, setup);
        app.add_systems(Update, camera_controls);
    }
}

#[derive(bevy::prelude::Component)]
struct CameraController;

fn setup(mut commands: Commands) {
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
