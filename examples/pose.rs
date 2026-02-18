//! Load a character and apply a pose from a JSON file.
//!
//! The pose JSON has a hierarchical structure matching bone names.
//! You can edit the JSON to adjust bone rotations/translations and see the result.

use bevy::prelude::*;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (apply_pose_once_loaded, camera_controls))
        .run();
}

/// Resource holding the loaded pose data
#[derive(Resource)]
struct PoseData {
    bones: HashMap<String, BoneTransform>,
}

#[derive(Clone)]
struct BoneTransform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

/// Marker for entities that need pose applied
#[derive(Component)]
struct NeedsPose;

/// Marker for camera with controls
#[derive(Component)]
struct CameraController;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load the character model
    let character = asset_server.load(GltfAssetLabel::Scene(0).from_asset("james-fixed.glb"));

    commands.spawn((
        SceneRoot(character),
        Transform::from_xyz(0.0, 0.0, 0.0),
        NeedsPose,
    ));

    // Load and parse the pose JSON
    let pose_data = load_pose_from_file("assets/pose.json");
    commands.insert_resource(pose_data);

    // Camera - positioned to see a ~170cm tall model in centimeters
    // Use WASD to move, QE for up/down, arrow keys to look around
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 100.0, 400.0).looking_at(Vec3::new(0.0, 80.0, 0.0), Vec3::Y),
        CameraController,
    ));

    println!("Camera controls: WASD=move, QE=up/down, Arrows=look");

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    // Debug: Add a red cube at origin to verify rendering works
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(50.0, 50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 25.0, 0.0),
    ));

    // Debug: Add a ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(500.0)).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        })),
    ));
}

fn load_pose_from_file(path: &str) -> PoseData {
    // Try multiple locations for the pose file
    let paths_to_try = [
        path.to_string(),
        format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path),
        format!("../{}", path),
    ];

    let content = paths_to_try
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_else(|| panic!("Failed to read pose file. Tried: {:?}", paths_to_try));
    let json: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse pose JSON");

    let mut bones = HashMap::new();

    // Recursively extract bone transforms from the hierarchical JSON
    fn extract_bones(
        json: &serde_json::Value,
        bones: &mut HashMap<String, BoneTransform>,
    ) {
        if let Some(obj) = json.as_object() {
            for (name, data) in obj {
                if name.starts_with('_') {
                    continue; // Skip metadata fields
                }

                // Extract transform
                let translation = data.get("translation").and_then(|v| {
                    let arr = v.as_array()?;
                    Some(Vec3::new(
                        arr[0].as_f64()? as f32,
                        arr[1].as_f64()? as f32,
                        arr[2].as_f64()? as f32,
                    ))
                }).unwrap_or(Vec3::ZERO);

                let rotation = data.get("rotation").and_then(|v| {
                    let arr = v.as_array()?;
                    // glTF uses xyzw quaternion order
                    Some(Quat::from_xyzw(
                        arr[0].as_f64()? as f32,
                        arr[1].as_f64()? as f32,
                        arr[2].as_f64()? as f32,
                        arr[3].as_f64()? as f32,
                    ))
                }).unwrap_or(Quat::IDENTITY);

                let scale = data.get("scale").and_then(|v| {
                    let arr = v.as_array()?;
                    Some(Vec3::new(
                        arr[0].as_f64()? as f32,
                        arr[1].as_f64()? as f32,
                        arr[2].as_f64()? as f32,
                    ))
                }).unwrap_or(Vec3::ONE);

                bones.insert(name.clone(), BoneTransform {
                    translation,
                    rotation,
                    scale,
                });

                // Recurse into children
                if let Some(children) = data.get("children") {
                    extract_bones(children, bones);
                }
            }
        }
    }

    // Start from the "pose" key
    if let Some(pose) = json.get("pose") {
        extract_bones(pose, &mut bones);
    }

    println!("Loaded {} bone transforms from pose file", bones.len());
    PoseData { bones }
}

/// Apply the pose to bones once the scene is loaded
fn apply_pose_once_loaded(
    mut commands: Commands,
    pose_data: Res<PoseData>,
    scene_query: Query<Entity, With<NeedsPose>>,
    mut bone_query: Query<(&Name, &mut Transform)>,
    children_query: Query<&Children>,
) {
    for scene_entity in &scene_query {
        // Check if the scene has children (meaning it's loaded)
        let Ok(children) = children_query.get(scene_entity) else {
            continue;
        };

        if children.is_empty() {
            continue;
        }

        // Scene is loaded, apply pose to all bones
        let mut applied = 0;
        for (name, transform) in &bone_query {
            if pose_data.bones.contains_key(name.as_str()) {
                // DEBUG: Just count, don't apply pose yet
                println!("Bone: {} at {:?}", name, transform.translation);
                applied += 1;
            }
        }

        println!("Found {} bones (not applying pose for debug)", applied);

        // Remove the marker so we don't apply again
        commands.entity(scene_entity).remove::<NeedsPose>();
    }
}

/// Simple camera controls for finding the model
fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CameraController>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let speed = 200.0 * time.delta_secs();
    let rot_speed = 1.0 * time.delta_secs();

    // Movement
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

    // Rotation
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

    // Print position with P key
    if keyboard.just_pressed(KeyCode::KeyP) {
        println!("Camera position: {:?}", transform.translation);
    }
}
