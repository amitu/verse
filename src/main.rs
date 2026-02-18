use bevy::{color::palettes::css::RED, math::prelude::*, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn our viewport so we can see things
    commands.spawn(Camera2d);

    let red: Color = RED.into();
    let circle = Circle::new(50.);

    // Circle mesh
    commands.spawn((
        Mesh2d(meshes.add(circle)),
        MeshMaterial2d(materials.add(ColorMaterial::from(red))),
        Transform::from_xyz(-150., 0., 0.),
    ));

    // Sprite
    commands.spawn((
        Sprite {
            image: asset_server.load("enemy.png"),
            ..default()
        },
        Transform::from_translation(Vec3::new(50., 50., 0.)),
    ));
}
