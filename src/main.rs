use bevy::prelude::*;

mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(camera::Plugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let red: Color = bevy::color::palettes::css::RED.into();

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
}
