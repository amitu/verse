#!/usr/bin/env python3
"""
Dump bone hierarchy and transforms from a GLB file to a hierarchical JSON pose file.

Usage:
    python dump-pose.py <file.glb> [output.json]

The output JSON has nested structure matching the bone hierarchy:
{
    "mixamorig:Hips": {
        "translation": [0, 0, 0],
        "rotation": [0, 0, 0, 1],  // quaternion (x, y, z, w)
        "scale": [1, 1, 1],
        "children": {
            "mixamorig:Spine": { ... },
            "mixamorig:LeftUpLeg": { ... }
        }
    }
}
"""

import json
import struct
import sys
from pathlib import Path


def read_glb(filepath):
    """Read and parse a GLB file."""
    with open(filepath, 'rb') as f:
        magic = f.read(4)
        if magic != b'glTF':
            raise ValueError(f"Not a valid GLB file: {filepath}")

        version = struct.unpack('<I', f.read(4))[0]
        total_length = struct.unpack('<I', f.read(4))[0]

        json_chunk_length = struct.unpack('<I', f.read(4))[0]
        json_chunk_type = f.read(4)
        json_data = json.loads(f.read(json_chunk_length).decode('utf-8'))

        return json_data


def build_hierarchy(nodes, skin):
    """Build the bone hierarchy from skin joints."""
    joints = skin.get('joints', [])

    # Build children map from nodes
    children_map = {}  # node_index -> [child_indices]
    for i, node in enumerate(nodes):
        children_map[i] = node.get('children', [])

    # Find root bones (joints that aren't children of other joints)
    joint_set = set(joints)
    root_joints = []
    for joint_idx in joints:
        # Check if this joint's parent is also a joint
        is_root = True
        for other_idx in joints:
            if joint_idx in children_map.get(other_idx, []):
                is_root = False
                break
        if is_root:
            root_joints.append(joint_idx)

    def build_bone_tree(node_idx):
        """Recursively build bone data."""
        node = nodes[node_idx]
        name = node.get('name', f'bone_{node_idx}')

        # Get transform (default to identity)
        translation = node.get('translation', [0.0, 0.0, 0.0])
        rotation = node.get('rotation', [0.0, 0.0, 0.0, 1.0])  # xyzw quaternion
        scale = node.get('scale', [1.0, 1.0, 1.0])

        bone_data = {
            'translation': translation,
            'rotation': rotation,
            'scale': scale,
        }

        # Add children that are also joints
        children = {}
        for child_idx in children_map.get(node_idx, []):
            if child_idx in joint_set:
                child_node = nodes[child_idx]
                child_name = child_node.get('name', f'bone_{child_idx}')
                children[child_name] = build_bone_tree(child_idx)

        if children:
            bone_data['children'] = children

        return bone_data

    # Build from root joints
    pose = {}
    for root_idx in root_joints:
        root_node = nodes[root_idx]
        root_name = root_node.get('name', f'bone_{root_idx}')
        pose[root_name] = build_bone_tree(root_idx)

    return pose


def dump_pose(filepath, output_path=None):
    """Main function to dump pose from GLB."""
    filepath = Path(filepath)
    if not filepath.exists():
        print(f"Error: File not found: {filepath}")
        return

    data = read_glb(filepath)
    nodes = data.get('nodes', [])
    skins = data.get('skins', [])

    if not skins:
        print(f"Error: No skins (armatures) found in {filepath}")
        return

    # Use first skin
    skin = skins[0]
    pose = build_hierarchy(nodes, skin)

    # Wrap in metadata
    output = {
        '_source': filepath.name,
        '_bone_count': len(skin.get('joints', [])),
        'pose': pose
    }

    # Output
    if output_path:
        output_path = Path(output_path)
        with open(output_path, 'w') as f:
            json.dump(output, f, indent=2)
        print(f"Pose written to: {output_path}")
    else:
        print(json.dumps(output, indent=2))


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return

    filepath = sys.argv[1]
    output_path = sys.argv[2] if len(sys.argv) > 2 else None
    dump_pose(filepath, output_path)


if __name__ == '__main__':
    main()
