use bevy::prelude::*;
use std::time::Duration;

// Offset to apply when transitioning to idle (adjust to match animation endpoints)
// Negative X moves left, positive X moves right
// Negative Z moves forward, positive Z moves backward
const IDLE_TRANSITION_OFFSET: Vec3 = Vec3::new(-0.025, 0.0, 0.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (setup_scene_once_loaded, handle_animation_transitions),
        )
        .run();
}

#[derive(Resource)]
struct Animations {
    graph: Handle<AnimationGraph>,
    // Store indices for each animation
    action_index: AnimationNodeIndex,
    idle_index: AnimationNodeIndex,
    // Store clip handle to query duration
    action_clip: Handle<AnimationClip>,
}

// Track which animation is currently playing
#[derive(Component, Default, PartialEq)]
enum AnimationState {
    #[default]
    Action,
    Idle,
}

// Marker for the character root entity
#[derive(Component)]
struct Character;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Load the character model
    let character = asset_server.load(GltfAssetLabel::Scene(0).from_asset("character.glb"));

    // Load animations from different GLB files
    let action_clip: Handle<AnimationClip> =
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("character.glb"));
    let idle_clip: Handle<AnimationClip> =
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("standing-idle.glb"));

    // Create animation graph with multiple clips
    let mut graph = AnimationGraph::new();
    let action_index = graph.add_clip(action_clip.clone(), 1.0, graph.root);
    let idle_index = graph.add_clip(idle_clip, 1.0, graph.root);

    let graph_handle = graphs.add(graph);

    commands.insert_resource(Animations {
        graph: graph_handle,
        action_index,
        idle_index,
        action_clip,
    });

    // Spawn the character
    commands.spawn((
        SceneRoot(character),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Character,
    ));

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

// Once the scene is loaded, start with the action animation
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        // Add the animation graph, transitions, and state tracking
        commands.entity(entity).insert((
            AnimationGraphHandle(animations.graph.clone()),
            AnimationTransitions::new(),
            AnimationState::Action,
        ));

        // Start with the action animation (plays once, no repeat)
        player.play(animations.action_index);
    }
}

// Check if action animation finished and transition to idle
fn handle_animation_transitions(
    animations: Res<Animations>,
    animation_clips: Res<Assets<AnimationClip>>,
    mut anim_query: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &mut AnimationState,
    )>,
    mut char_query: Query<&mut Transform, With<Character>>,
) {
    for (mut player, mut transitions, mut state) in &mut anim_query {
        // Only check for transition if we're still in Action state
        if *state != AnimationState::Action {
            continue;
        }

        // Check if the action animation is near the end
        if let Some(active) = player.animation(animations.action_index) {
            let elapsed = active.seek_time();

            // Get the clip duration
            let Some(clip) = animation_clips.get(&animations.action_clip) else {
                continue;
            };

            let duration = clip.duration();
            let blend_time = 0.5;

            // Start blending when we're within blend_time of the end
            if elapsed >= duration - blend_time || active.is_finished() {
                transitions
                    .play(
                        &mut player,
                        animations.idle_index,
                        Duration::from_secs_f32(blend_time),
                    )
                    .repeat();

                // Apply position offset to compensate for animation mismatch
                for mut transform in &mut char_query {
                    transform.translation += IDLE_TRANSITION_OFFSET;
                }

                *state = AnimationState::Idle;
            }
        }
    }
}
