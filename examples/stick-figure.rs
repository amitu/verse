//! Render a skeleton using geometric primitives with auto-reload.
//!
//! Usage: cargo run --example stick-figure -- [pose_file]
//! Default pose: standing.pose.json
//!
//! Edit the pose JSON file and save - the preview updates automatically!

use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::SystemTime;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            ..default()
        })
        .init_resource::<FigureState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_controls, check_reload))
        .run();
}

// ============ Resources ============

#[derive(Resource, Default)]
struct FigureState {
    pose_file: String,
    pose_path: String,
    character_path: String,
    skeleton_path: String,
    last_modified: Option<SystemTime>,
}

// ============ Components ============

#[derive(Component)]
struct CameraController;

#[derive(Component)]
struct FigurePart;

// ============ Data Structures ============

#[derive(Debug, Clone)]
struct Skeleton {
    bones: HashMap<String, BoneDef>,
}

#[derive(Debug, Clone, PartialEq)]
enum BoneSide {
    Center,
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct BoneDef {
    default_length: f32,
    rest_direction: Vec3,
    offset: Vec3,
    side: BoneSide,
    joint_type: JointType,
    #[allow(dead_code)]
    constraints: HashMap<String, (f32, f32)>,
    children: Vec<String>,
}

#[derive(Debug, Clone)]
enum JointType {
    Root,
    BallSocket,
    Hinge,
}

#[derive(Debug, Clone)]
struct Character {
    hip_position: Vec3,
    bone_scales: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
struct Pose {
    joints: HashMap<String, HashMap<String, f32>>,
}

// ============ Loading Functions ============

fn resolve_path(path: &str) -> String {
    let paths = [
        path.to_string(),
        format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path),
        format!("assets/{}", path),
        format!("{}/assets/{}", env!("CARGO_MANIFEST_DIR"), path),
    ];
    paths
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
        .unwrap_or_else(|| panic!("File not found: {}", path))
}

fn get_modified_time(path: &str) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

fn load_skeleton(path: &str) -> Skeleton {
    let full_path = resolve_path(path);
    let content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("Failed to read skeleton: {}", full_path));
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let mut bones = HashMap::new();

    if let Some(bones_obj) = json["bones"].as_object() {
        for (name, data) in bones_obj {
            let joint_type = match data["joint"]["type"].as_str().unwrap_or("root") {
                "ball_socket" => JointType::BallSocket,
                "hinge" => JointType::Hinge,
                _ => JointType::Root,
            };

            let mut constraints = HashMap::new();
            if let Some(cons) = data["joint"]["constraints"].as_object() {
                for (axis, range) in cons {
                    if let Some(arr) = range.as_array() {
                        let min = arr[0].as_f64().unwrap_or(0.0) as f32;
                        let max = arr[1].as_f64().unwrap_or(0.0) as f32;
                        constraints.insert(axis.clone(), (min, max));
                    }
                }
            }

            let rest_direction = data["rest_direction"]
                .as_array()
                .map(|arr| Vec3::new(
                    arr[0].as_f64().unwrap_or(0.0) as f32,
                    arr[1].as_f64().unwrap_or(-1.0) as f32,
                    arr[2].as_f64().unwrap_or(0.0) as f32,
                ))
                .unwrap_or(Vec3::NEG_Y);

            let offset = data["offset"]
                .as_array()
                .map(|arr| Vec3::new(
                    arr[0].as_f64().unwrap_or(0.0) as f32,
                    arr[1].as_f64().unwrap_or(0.0) as f32,
                    arr[2].as_f64().unwrap_or(0.0) as f32,
                ))
                .unwrap_or(Vec3::ZERO);

            // Infer side from bone name
            let side = if name.contains("left") {
                BoneSide::Left
            } else if name.contains("right") {
                BoneSide::Right
            } else {
                BoneSide::Center
            };

            let children: Vec<String> = data["children"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            bones.insert(
                name.clone(),
                BoneDef {
                    default_length: data["default_length"].as_f64().unwrap_or(0.0) as f32,
                    rest_direction,
                    offset,
                    side,
                    joint_type,
                    constraints,
                    children,
                },
            );
        }
    }

    Skeleton { bones }
}

fn load_character(path: &str) -> (Character, Skeleton, String) {
    let full_path = resolve_path(path);
    let content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("Failed to read character: {}", full_path));
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let skeleton_file = json["skeleton"].as_str().unwrap_or("human.skeleton.json");
    let skeleton_path = resolve_path(skeleton_file);
    let skeleton = load_skeleton(skeleton_file);

    let hip_pos = json["hip_position"].as_array().unwrap();
    let hip_position = Vec3::new(
        hip_pos[0].as_f64().unwrap() as f32,
        hip_pos[1].as_f64().unwrap() as f32,
        hip_pos[2].as_f64().unwrap() as f32,
    );

    let mut bone_scales = HashMap::new();
    if let Some(scales) = json["bone_scales"].as_object() {
        for (name, scale) in scales {
            if let Some(s) = scale.as_f64() {
                bone_scales.insert(name.clone(), s as f32);
            }
        }
    }

    (Character { hip_position, bone_scales }, skeleton, skeleton_path)
}

fn load_pose(path: &str) -> (Pose, Character, Skeleton, String, String, String) {
    let full_path = resolve_path(path);
    let content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("Failed to read pose: {}", full_path));
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let character_file = json["character"].as_str().unwrap_or("james.character.json");
    let character_path = resolve_path(character_file);
    let (character, skeleton, skeleton_path) = load_character(character_file);

    let mut joints = HashMap::new();
    if let Some(joints_obj) = json["joints"].as_object() {
        for (bone_name, angles) in joints_obj {
            let mut bone_angles = HashMap::new();
            if let Some(angles_obj) = angles.as_object() {
                for (axis, val) in angles_obj {
                    bone_angles.insert(axis.clone(), val.as_f64().unwrap_or(0.0) as f32);
                }
            }
            joints.insert(bone_name.clone(), bone_angles);
        }
    }

    (Pose { joints }, character, skeleton, full_path, character_path, skeleton_path)
}

// ============ Angle to Rotation ============

fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

fn angles_to_rotation(joint_type: &JointType, angles: &HashMap<String, f32>, side: &BoneSide) -> Quat {
    let get = |name: &str| -> f32 {
        *angles.get(name).unwrap_or(&0.0)
    };

    // Mirror factor: for right side, negate abduction so positive = outward on both sides
    let mirror = if *side == BoneSide::Right { -1.0 } else { 1.0 };

    match joint_type {
        JointType::Root => Quat::IDENTITY,
        JointType::BallSocket => {
            // flexion = forward/back (same both sides)
            // abduction = outward/inward (mirrored for right side)
            // rotation = twist (same both sides)
            let flexion = get("flexion");
            let abduction = get("abduction") * mirror;
            let rotation = get("rotation");
            Quat::from_euler(EulerRot::ZXY,
                deg_to_rad(abduction),
                deg_to_rad(flexion),
                deg_to_rad(rotation))
        }
        JointType::Hinge => {
            let angle = get("angle");
            Quat::from_rotation_x(deg_to_rad(angle))
        }
    }
}

fn rest_direction_to_rotation(rest_dir: Vec3) -> Quat {
    // Convert rest direction to a rotation
    // Bones extend along NEG_Y by default in our coordinate system
    if rest_dir.dot(Vec3::NEG_Y).abs() > 0.999 {
        if rest_dir.y < 0.0 { Quat::IDENTITY } else { Quat::from_rotation_x(PI) }
    } else {
        Quat::from_rotation_arc(Vec3::NEG_Y, rest_dir)
    }
}

// ============ Rendering ============

fn rotation_from_direction(dir: Vec3) -> Quat {
    let default_up = Vec3::Y;
    if dir.dot(default_up).abs() > 0.999 {
        if dir.y > 0.0 { Quat::IDENTITY } else { Quat::from_rotation_x(PI) }
    } else {
        Quat::from_rotation_arc(default_up, dir)
    }
}

struct BoneVisual {
    start: Vec3,
    end: Vec3,
    rotation: Quat,
}

fn compute_bone_visuals(
    skeleton: &Skeleton,
    character: &Character,
    pose: &Pose,
    bone_name: &str,
    parent_pos: Vec3,
    parent_pose_accum: Quat,  // Only accumulated POSE rotations, not rest
) -> Vec<(String, BoneVisual)> {
    let mut result = Vec::new();

    let Some(bone_def) = skeleton.bones.get(bone_name) else {
        return result;
    };

    let scale = character.bone_scales.get(bone_name).copied().unwrap_or(1.0);
    let length = bone_def.default_length * scale;

    // Rest direction defines the bone's natural orientation (absolute, not inherited)
    let rest_rotation = rest_direction_to_rotation(bone_def.rest_direction);

    // Pose angles modify from rest position
    let angles = pose.joints.get(bone_name).cloned().unwrap_or_default();
    let pose_rotation = angles_to_rotation(&bone_def.joint_type, &angles, &bone_def.side);

    // World rotation: rest direction + accumulated parent poses + local pose
    // Key: rest_rotation is NOT inherited from parent, only pose rotations are
    let world_rotation = rest_rotation * parent_pose_accum * pose_rotation;

    // Apply offset in the rest frame (before pose modifications)
    let start_pos = parent_pos + rest_rotation * bone_def.offset;

    let dir = world_rotation * Vec3::NEG_Y;
    let end_pos = start_pos + dir * length;

    if length > 0.0 {
        result.push((
            bone_name.to_string(),
            BoneVisual {
                start: start_pos,
                end: end_pos,
                rotation: world_rotation,
            },
        ));
    }

    // For children, accumulate pose rotations (but not rest)
    let child_pose_accum = parent_pose_accum * pose_rotation;

    for child_name in &bone_def.children {
        let child_visuals = compute_bone_visuals(
            skeleton, character, pose, child_name, end_pos, child_pose_accum,
        );
        result.extend(child_visuals);
    }

    result
}

fn spawn_figure(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    skeleton: &Skeleton,
    character: &Character,
    pose: &Pose,
) {
    let joint_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.3, 0.3),
        ..default()
    });
    let torso_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.6, 0.8),
        ..default()
    });
    let leg_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.6, 0.4),
        ..default()
    });
    let arm_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.5, 0.3),
        ..default()
    });
    let extremity_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.7, 0.5),
        ..default()
    });
    let head_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.75, 0.6),
        ..default()
    });

    let joint_mesh = meshes.add(Sphere::new(2.5));

    let visuals = compute_bone_visuals(
        skeleton, character, pose, "hip", character.hip_position, Quat::IDENTITY,
    );

    // Hip joint
    commands.spawn((
        Mesh3d(joint_mesh.clone()),
        MeshMaterial3d(joint_material.clone()),
        Transform::from_translation(character.hip_position),
        FigurePart,
    ));

    for (name, visual) in &visuals {
        let length = visual.start.distance(visual.end);
        if length < 0.1 { continue; }

        let center = (visual.start + visual.end) / 2.0;
        let dir = (visual.end - visual.start).normalize();

        let is_head = name == "head";
        let is_foot = name.contains("foot");
        let is_hand = name.contains("hand");
        let is_leg = name.contains("leg");
        let is_arm = name.contains("arm") || name.contains("shoulder");
        let is_spine = name.contains("spine") || name == "neck";

        if is_head {
            // Cuboid so we can see head rotation (front is longer)
            let head_mesh = meshes.add(Cuboid::new(10.0, length, 12.0));
            let head_rotation = visual.rotation * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            commands.spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(head_material.clone()),
                Transform::from_translation(center).with_rotation(head_rotation),
                FigurePart,
            ));
        } else if is_foot {
            let foot_mesh = meshes.add(Cuboid::new(8.0, 4.0, length));
            let foot_rotation = visual.rotation * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            commands.spawn((
                Mesh3d(foot_mesh),
                MeshMaterial3d(extremity_material.clone()),
                Transform::from_translation(center).with_rotation(foot_rotation),
                FigurePart,
            ));
        } else if is_hand {
            let hand_mesh = meshes.add(Cuboid::new(6.0, 3.0, length));
            let hand_rotation = visual.rotation * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            commands.spawn((
                Mesh3d(hand_mesh),
                MeshMaterial3d(extremity_material.clone()),
                Transform::from_translation(center).with_rotation(hand_rotation),
                FigurePart,
            ));
        } else {
            let radius = if is_spine { 4.0 } else if is_leg { 3.0 } else { 2.5 };
            let bone_mesh = meshes.add(Cylinder::new(radius, length));
            let material = if is_spine {
                torso_material.clone()
            } else if is_leg {
                leg_material.clone()
            } else if is_arm {
                arm_material.clone()
            } else {
                leg_material.clone()
            };
            commands.spawn((
                Mesh3d(bone_mesh),
                MeshMaterial3d(material),
                Transform::from_translation(center).with_rotation(rotation_from_direction(dir)),
                FigurePart,
            ));
        }

        if !is_head {
            commands.spawn((
                Mesh3d(joint_mesh.clone()),
                MeshMaterial3d(joint_material.clone()),
                Transform::from_translation(visual.end),
                FigurePart,
            ));
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<FigureState>,
) {
    let args: Vec<String> = std::env::args().collect();
    let pose_file = args.get(1).map(|s| s.as_str()).unwrap_or("standing.pose.json");

    println!("Loading pose: {}", pose_file);
    let (pose, character, skeleton, pose_path, character_path, skeleton_path) = load_pose(pose_file);

    state.pose_file = pose_file.to_string();
    state.pose_path = pose_path.clone();
    state.character_path = character_path;
    state.skeleton_path = skeleton_path;
    state.last_modified = get_modified_time(&pose_path);

    spawn_figure(&mut commands, &mut meshes, &mut materials, &skeleton, &character, &pose);

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(200.0)).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2),
            ..default()
        })),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 120.0, 250.0).looking_at(Vec3::new(0.0, 100.0, 0.0), Vec3::Y),
        CameraController,
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
    ));

    println!("\nControls: WASD=move, QE=up/down, Arrows=look");
    println!("Auto-reload: Edit {} and save to update preview", pose_file);
}

fn check_reload(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<FigureState>,
    parts: Query<Entity, With<FigurePart>>,
) {
    // Check all three files for changes
    let pose_modified = get_modified_time(&state.pose_path);
    let char_modified = get_modified_time(&state.character_path);
    let skel_modified = get_modified_time(&state.skeleton_path);

    let latest = [pose_modified, char_modified, skel_modified]
        .into_iter()
        .flatten()
        .max();

    let needs_reload = match (&state.last_modified, &latest) {
        (Some(old), Some(new)) => new > old,
        (None, Some(_)) => true,
        _ => false,
    };

    if !needs_reload {
        return;
    }

    println!("Reloading...");
    state.last_modified = latest;

    // Despawn old figure
    for entity in parts.iter() {
        commands.entity(entity).despawn();
    }

    // Reload and respawn
    match std::panic::catch_unwind(|| load_pose(&state.pose_file)) {
        Ok((pose, character, skeleton, _, _, _)) => {
            spawn_figure(&mut commands, &mut meshes, &mut materials, &skeleton, &character, &pose);
            println!("Reloaded successfully!");
        }
        Err(_) => {
            println!("Error reloading - fix the JSON and save again");
        }
    }
}

fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CameraController>>,
) {
    let Ok(mut transform) = query.single_mut() else { return };

    let speed = 100.0 * time.delta_secs();
    let rot_speed = 1.0 * time.delta_secs();

    let forward = transform.forward();
    let right = transform.right();

    if keyboard.pressed(KeyCode::KeyW) { transform.translation += forward * speed; }
    if keyboard.pressed(KeyCode::KeyS) { transform.translation -= forward * speed; }
    if keyboard.pressed(KeyCode::KeyA) { transform.translation -= right * speed; }
    if keyboard.pressed(KeyCode::KeyD) { transform.translation += right * speed; }
    if keyboard.pressed(KeyCode::KeyQ) { transform.translation.y -= speed; }
    if keyboard.pressed(KeyCode::KeyE) { transform.translation.y += speed; }
    if keyboard.pressed(KeyCode::ArrowLeft) { transform.rotate_y(rot_speed); }
    if keyboard.pressed(KeyCode::ArrowRight) { transform.rotate_y(-rot_speed); }
    if keyboard.pressed(KeyCode::ArrowUp) { transform.rotate_local_x(rot_speed); }
    if keyboard.pressed(KeyCode::ArrowDown) { transform.rotate_local_x(-rot_speed); }
}
