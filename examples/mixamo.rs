use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, setup_scene_once_loaded)
        .run();
}

#[derive(Resource)]
struct Animations {
    graph: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Load the character model - change this path to your downloaded model
    let character = asset_server.load(GltfAssetLabel::Scene(0).from_asset("character.glb"));

    // Load the animation from the same file (Mixamo embeds animations in the model)
    let animation = asset_server.load(GltfAssetLabel::Animation(0).from_asset("standing-idle.glb"));

    // Create animation graph
    let (graph, index) = AnimationGraph::from_clip(animation);
    let graph_handle = graphs.add(graph);

    commands.insert_resource(Animations {
        graph: graph_handle,
        index,
    });

    // Spawn the character
    commands.spawn((SceneRoot(character), Transform::from_xyz(0.0, 0.0, 0.0)));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.0, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(asset_server.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)).into())),
        MeshMaterial3d(asset_server.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        })),
    ));
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        // Add the animation graph to the entity
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()));

        // Play the animation on loop
        player.play(animations.index).repeat();
    }
}
