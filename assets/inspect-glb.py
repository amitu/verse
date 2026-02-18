#!/usr/bin/env python3
"""
GLB Inspector - Dumps rigging and animation info from GLB files for debugging.

Usage:
    python inspect-glb.py <file.glb> [--full]

Options:
    --full    Show all nodes and detailed animation data
"""

import json
import struct
import sys
from pathlib import Path


def read_glb(filepath):
    """Read and parse a GLB file, returning the JSON data."""
    with open(filepath, 'rb') as f:
        # GLB Header
        magic = f.read(4)
        if magic != b'glTF':
            raise ValueError(f"Not a valid GLB file: {filepath}")

        version = struct.unpack('<I', f.read(4))[0]
        total_length = struct.unpack('<I', f.read(4))[0]

        # JSON chunk
        json_chunk_length = struct.unpack('<I', f.read(4))[0]
        json_chunk_type = f.read(4)
        json_data = json.loads(f.read(json_chunk_length).decode('utf-8'))

        return json_data, version, total_length


def build_node_tree(nodes):
    """Build parent-child relationships from node list."""
    children_map = {}  # node_index -> list of child indices
    parent_map = {}    # node_index -> parent index

    for i, node in enumerate(nodes):
        children = node.get('children', [])
        children_map[i] = children
        for child in children:
            parent_map[child] = i

    return children_map, parent_map


def print_skeleton_tree(nodes, children_map, parent_map, root_idx, indent=0):
    """Recursively print the skeleton hierarchy."""
    node = nodes[root_idx]
    name = node.get('name', f'<node_{root_idx}>')

    # Check if this node has a mesh, skin, or is just a bone
    extras = []
    if 'mesh' in node:
        extras.append('mesh')
    if 'skin' in node:
        extras.append('skin')
    if 'translation' in node or 'rotation' in node or 'scale' in node:
        extras.append('transform')

    extra_str = f" [{', '.join(extras)}]" if extras else ""
    print(f"{'  ' * indent}{root_idx}: {name}{extra_str}")

    for child_idx in children_map.get(root_idx, []):
        print_skeleton_tree(nodes, children_map, parent_map, child_idx, indent + 1)


def inspect_glb(filepath, full=False):
    """Main inspection function."""
    filepath = Path(filepath)
    if not filepath.exists():
        print(f"Error: File not found: {filepath}")
        return

    try:
        data, version, total_length = read_glb(filepath)
    except Exception as e:
        print(f"Error reading {filepath}: {e}")
        return

    print(f"\n{'='*60}")
    print(f"FILE: {filepath.name}")
    print(f"{'='*60}")
    print(f"glTF Version: {version}")
    print(f"Total Size: {total_length:,} bytes")

    nodes = data.get('nodes', [])
    meshes = data.get('meshes', [])
    skins = data.get('skins', [])
    animations = data.get('animations', [])
    accessors = data.get('accessors', [])

    print(f"\n--- SUMMARY ---")
    print(f"Nodes: {len(nodes)}")
    print(f"Meshes: {len(meshes)}")
    print(f"Skins (Armatures): {len(skins)}")
    print(f"Animations: {len(animations)}")

    # Skins (Armatures)
    if skins:
        print(f"\n--- SKINS (ARMATURES) ---")
        for i, skin in enumerate(skins):
            skin_name = skin.get('name', f'<skin_{i}>')
            joints = skin.get('joints', [])
            skeleton_root = skin.get('skeleton')
            print(f"\nSkin {i}: {skin_name}")
            print(f"  Joints (bones): {len(joints)}")
            if skeleton_root is not None:
                root_name = nodes[skeleton_root].get('name', f'<node_{skeleton_root}>')
                print(f"  Skeleton root: {skeleton_root} ({root_name})")

            # Show bone names
            print(f"  Bone names:")
            for j, joint_idx in enumerate(joints):
                bone_name = nodes[joint_idx].get('name', f'<node_{joint_idx}>')
                if full or j < 10:
                    print(f"    {j}: {bone_name}")
            if not full and len(joints) > 10:
                print(f"    ... and {len(joints) - 10} more (use --full to see all)")

    # Node hierarchy
    children_map, parent_map = build_node_tree(nodes)
    root_nodes = [i for i in range(len(nodes)) if i not in parent_map]

    print(f"\n--- NODE HIERARCHY ---")
    print(f"Root nodes: {root_nodes}")
    for root_idx in root_nodes:
        if full or len(nodes) <= 30:
            print_skeleton_tree(nodes, children_map, parent_map, root_idx)
        else:
            # Just show first few levels
            node = nodes[root_idx]
            name = node.get('name', f'<node_{root_idx}>')
            print(f"{root_idx}: {name}")
            for child_idx in children_map.get(root_idx, [])[:5]:
                child_name = nodes[child_idx].get('name', f'<node_{child_idx}>')
                print(f"  {child_idx}: {child_name}")
                for grandchild_idx in children_map.get(child_idx, [])[:3]:
                    gc_name = nodes[grandchild_idx].get('name', f'<node_{grandchild_idx}>')
                    print(f"    {grandchild_idx}: {gc_name}")
            if len(children_map.get(root_idx, [])) > 5:
                print(f"  ... (use --full to see all)")

    # Animations
    if animations:
        print(f"\n--- ANIMATIONS ---")
        for i, anim in enumerate(animations):
            anim_name = anim.get('name', f'<animation_{i}>')
            channels = anim.get('channels', [])
            samplers = anim.get('samplers', [])

            print(f"\nAnimation {i}: \"{anim_name}\"")
            print(f"  Channels: {len(channels)}")
            print(f"  Samplers: {len(samplers)}")

            # Calculate duration from samplers
            max_time = 0
            for sampler in samplers:
                input_accessor = sampler.get('input')
                if input_accessor is not None and input_accessor < len(accessors):
                    accessor = accessors[input_accessor]
                    if 'max' in accessor:
                        max_time = max(max_time, accessor['max'][0])

            if max_time > 0:
                print(f"  Duration: {max_time:.2f}s")

            # Show which nodes are animated
            animated_nodes = set()
            target_paths = {}  # node_idx -> list of paths
            for channel in channels:
                target = channel.get('target', {})
                node_idx = target.get('node')
                path = target.get('path')  # translation, rotation, scale, weights
                if node_idx is not None:
                    animated_nodes.add(node_idx)
                    if node_idx not in target_paths:
                        target_paths[node_idx] = []
                    target_paths[node_idx].append(path)

            print(f"  Animated nodes: {len(animated_nodes)}")
            if full:
                for node_idx in sorted(animated_nodes):
                    node_name = nodes[node_idx].get('name', f'<node_{node_idx}>') if node_idx < len(nodes) else f'<invalid_{node_idx}>'
                    paths = ', '.join(target_paths.get(node_idx, []))
                    print(f"    {node_idx}: {node_name} ({paths})")
            else:
                # Show first few
                for node_idx in sorted(animated_nodes)[:5]:
                    node_name = nodes[node_idx].get('name', f'<node_{node_idx}>') if node_idx < len(nodes) else f'<invalid_{node_idx}>'
                    paths = ', '.join(target_paths.get(node_idx, []))
                    print(f"    {node_idx}: {node_name} ({paths})")
                if len(animated_nodes) > 5:
                    print(f"    ... and {len(animated_nodes) - 5} more (use --full)")

    # Bone name prefix analysis
    print(f"\n--- BONE NAME ANALYSIS ---")
    bone_names = [node.get('name', '') for node in nodes if node.get('name')]
    prefixes = {}
    for name in bone_names:
        if ':' in name:
            prefix = name.split(':')[0] + ':'
            prefixes[prefix] = prefixes.get(prefix, 0) + 1

    if prefixes:
        print("Detected prefixes:")
        for prefix, count in sorted(prefixes.items(), key=lambda x: -x[1]):
            print(f"  {prefix} ({count} nodes)")
    else:
        print("No prefixes detected (no ':' in node names)")

    print()


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        print("\nAvailable GLB files in current directory:")
        for f in Path('.').glob('*.glb'):
            print(f"  {f.name}")
        return

    full = '--full' in sys.argv
    files = [arg for arg in sys.argv[1:] if not arg.startswith('--')]

    for filepath in files:
        inspect_glb(filepath, full=full)


if __name__ == '__main__':
    main()
