//! Render a skeleton using geometric primitives with proper separation:
//! - skeleton.json: bone structure, joint types, constraints
//! - character.json: bone scales, hip position
//! - pose.json: joint angles
//!
//! Usage: cargo run --example leg -- [pose_file]
//! Default pose: standing.pose.json

use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, camera_controls)
        .run();
}

#[derive(Component)]
struct CameraController;

// ============ Data Structures ============

#[derive(Debug, Clone)]
struct Skeleton {
    bones: HashMap<String, BoneDef>,
}

#[derive(Debug, Clone)]
struct BoneDef {
    default_length: f32,
    joint_type: JointType,
    constraints: HashMap<String, (f32, f32)>,
    default_angles: HashMap<String, f32>,
    children: Vec<String>,
}

#[derive(Debug, Clone)]
enum JointType {
    Root,
    BallSocket, // 3 DOF: flexion, abduction, rotation
    Hinge,      // 1 DOF: angle
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

            let mut default_angles = HashMap::new();
            if let Some(defaults) = data["joint"]["default_angles"].as_object() {
                for (axis, val) in defaults {
                    default_angles.insert(axis.clone(), val.as_f64().unwrap_or(0.0) as f32);
                }
            }

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
                    joint_type,
                    constraints,
                    default_angles,
                    children,
                },
            );
        }
    }

    Skeleton { bones }
}

fn load_character(path: &str) -> (Character, Skeleton) {
    let full_path = resolve_path(path);
    let content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("Failed to read character: {}", full_path));
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let skeleton_path = json["skeleton"].as_str().unwrap_or("human.skeleton.json");
    let skeleton = load_skeleton(skeleton_path);

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

    (Character { hip_position, bone_scales }, skeleton)
}

fn load_pose(path: &str) -> (Pose, Character, Skeleton) {
    let full_path = resolve_path(path);
    let content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|_| panic!("Failed to read pose: {}", full_path));
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let character_path = json["character"].as_str().unwrap_or("james.character.json");
    let (character, skeleton) = load_character(character_path);

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

    (Pose { joints }, character, skeleton)
}

// ============ Angle to Rotation ============

fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

fn angles_to_rotation(joint_type: &JointType, angles: &HashMap<String, f32>, defaults: &HashMap<String, f32>) -> Quat {
    let get = |name: &str| -> f32 {
        *angles.get(name).or_else(|| defaults.get(name)).unwrap_or(&0.0)
    };

    match joint_type {
        JointType::Root => Quat::IDENTITY,
        JointType::BallSocket => {
            let flexion = get("flexion");
            let abduction = get("abduction");
            let rotation = get("rotation");
            let q_abduction = Quat::from_rotation_z(deg_to_rad(abduction));
            let q_flexion = Quat::from_rotation_x(deg_to_rad(flexion));
            let q_rotation = Quat::from_rotation_y(deg_to_rad(rotation));
            q_abduction * q_flexion * q_rotation
        }
        JointType::Hinge => {
            let angle = get("angle");
            Quat::from_rotation_x(deg_to_rad(angle))
        }
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
    parent_rotation: Quat,
) -> Vec<(String, BoneVisual)> {
    let mut result = Vec::new();

    let Some(bone_def) = skeleton.bones.get(bone_name) else {
        return result;
    };

    let scale = character.bone_scales.get(bone_name).copied().unwrap_or(1.0);
    let length = bone_def.default_length * scale;

    let angles = pose.joints.get(bone_name).cloned().unwrap_or_default();
    let local_rotation = angles_to_rotation(&bone_def.joint_type, &angles, &bone_def.default_angles);
    let world_rotation = parent_rotation * local_rotation;

    let dir = world_rotation * Vec3::NEG_Y;
    let end_pos = parent_pos + dir * length;

    if length > 0.0 {
        result.push((
            bone_name.to_string(),
            BoneVisual {
                start: parent_pos,
                end: end_pos,
                rotation: world_rotation,
            },
        ));
    }

    for child_name in &bone_def.children {
        let child_visuals = compute_bone_visuals(
            skeleton, character, pose, child_name, end_pos, world_rotation,
        );
        result.extend(child_visuals);
    }

    result
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get pose file from args or use default
    let args: Vec<String> = std::env::args().collect();
    let pose_file = args.get(1).map(|s| s.as_str()).unwrap_or("standing.pose.json");

    println!("Loading pose: {}", pose_file);
    let (pose, character, skeleton) = load_pose(pose_file);

    println!("Skeleton bones: {:?}", skeleton.bones.keys().collect::<Vec<_>>());
    println!("Character hip: {:?}", character.hip_position);

    // Materials
    let joint_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.3, 0.3),
        ..default()
    });
    let bone_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.6, 0.4),
        ..default()
    });
    let foot_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.4, 0.2),
        ..default()
    });

    let joint_mesh = meshes.add(Sphere::new(3.0));
    let bone_radius = 2.5;

    // Compute all bone positions starting from hip
    let visuals = compute_bone_visuals(
        &skeleton,
        &character,
        &pose,
        "hip",
        character.hip_position,
        Quat::IDENTITY,
    );

    // Spawn hip joint
    commands.spawn((
        Mesh3d(joint_mesh.clone()),
        MeshMaterial3d(joint_material.clone()),
        Transform::from_translation(character.hip_position),
    ));

    // Spawn bones and joints
    for (name, visual) in &visuals {
        let length = visual.start.distance(visual.end);
        if length < 0.1 {
            continue;
        }

        let center = (visual.start + visual.end) / 2.0;
        let dir = (visual.end - visual.start).normalize();

        // Choose mesh based on bone type
        let is_foot = name.contains("foot");
        if is_foot {
            // Cuboid for feet
            let foot_mesh = meshes.add(Cuboid::new(8.0, 4.0, length));
            let foot_rotation = visual.rotation * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            commands.spawn((
                Mesh3d(foot_mesh),
                MeshMaterial3d(foot_material.clone()),
                Transform::from_translation(center).with_rotation(foot_rotation),
            ));
        } else {
            // Cylinder for other bones
            let bone_mesh = meshes.add(Cylinder::new(bone_radius, length));
            commands.spawn((
                Mesh3d(bone_mesh),
                MeshMaterial3d(bone_material.clone()),
                Transform::from_translation(center).with_rotation(rotation_from_direction(dir)),
            ));
        }

        // Joint at end of bone
        commands.spawn((
            Mesh3d(joint_mesh.clone()),
            MeshMaterial3d(joint_material.clone()),
            Transform::from_translation(visual.end),
        ));

        println!("  {}: {:?} -> {:?}", name, visual.start, visual.end);
    }

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
        Transform::from_xyz(150.0, 80.0, 150.0).looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
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
