use glam::IVec3;

use crate::block::{BlockId, BlockPos};

const AIR: BlockId = 0;
const WORLD_MIN: i32 = -512;
const WORLD_SIZE: i32 = 1024;
const MAX_DEPTH: u8 = 10;

#[derive(Clone)]
enum OctreeNode {
    Branch {
        children: [Option<Box<OctreeNode>>; 8],
        aggregate: BlockId,
    },
    Leaf(BlockId),
}

/// Sparse voxel octree with pointer-based nodes and upward aggregate propagation.
pub struct SparseVoxelOctree {
    root: Option<Box<OctreeNode>>,
}

impl Default for SparseVoxelOctree {
    fn default() -> Self {
        Self::new()
    }
}

impl SparseVoxelOctree {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn get_block(&self, position: BlockPos) -> BlockId {
        let Some(root) = &self.root else {
            return AIR;
        };
        get_block_node(root.as_ref(), position, WORLD_MIN, WORLD_SIZE, 0)
    }

    pub fn is_solid(&self, position: BlockPos) -> bool {
        self.get_block(position) != AIR
    }

    pub fn set_block(&mut self, position: BlockPos, block: BlockId) {
        if block == AIR {
            self.root = remove_block_node(
                self.root.take(),
                position,
                WORLD_MIN,
                WORLD_SIZE,
                0,
            );
        } else {
            let root = self.root.take().unwrap_or_else(|| {
                Box::new(OctreeNode::Branch {
                    children: empty_children(),
                    aggregate: AIR,
                })
            });
            self.root = Some(set_block_node(root, position, block, WORLD_MIN, WORLD_SIZE, 0));
        }
    }

    pub fn for_each_solid_in_region<F>(&self, min: BlockPos, max: BlockPos, mut f: F)
    where
        F: FnMut(BlockPos, BlockId),
    {
        if let Some(root) = &self.root {
            visit_region(
                root.as_ref(),
                WORLD_MIN,
                WORLD_SIZE,
                0,
                min.0,
                max.0,
                &mut f,
            );
        }
    }
}

fn empty_children() -> [Option<Box<OctreeNode>>; 8] {
    std::array::from_fn(|_| None)
}

fn child_bounds(min: i32, size: i32, index: usize) -> (i32, i32) {
    let half = size / 2;
    let offset = if index & 1 != 0 { half } else { 0 };
    (min + offset, half)
}

fn child_index(position: IVec3, min: i32, size: i32) -> usize {
    let half = size / 2;
    let mut index = 0;
    if position.x >= min + half {
        index |= 1;
    }
    if position.y >= min + half {
        index |= 2;
    }
    if position.z >= min + half {
        index |= 4;
    }
    index
}

fn get_block_node(node: &OctreeNode, position: BlockPos, min: i32, size: i32, depth: u8) -> BlockId {
    match node {
        OctreeNode::Leaf(block) => *block,
        OctreeNode::Branch { children, aggregate } => {
            if depth >= MAX_DEPTH || size <= 1 {
                return *aggregate;
            }
            let index = child_index(position.0, min, size);
            let (child_min, child_size) = child_bounds(min, size, index);
            children[index]
                .as_ref()
                .map(|child| get_block_node(child, position, child_min, child_size, depth + 1))
                .unwrap_or(AIR)
        }
    }
}

fn set_block_node(
    mut node: Box<OctreeNode>,
    position: BlockPos,
    block: BlockId,
    min: i32,
    size: i32,
    depth: u8,
) -> Box<OctreeNode> {
    if depth >= MAX_DEPTH || size <= 1 {
        return Box::new(OctreeNode::Leaf(block));
    }

    let OctreeNode::Branch { children, .. } = &mut *node else {
        return Box::new(OctreeNode::Leaf(block));
    };

    let index = child_index(position.0, min, size);
    let (child_min, child_size) = child_bounds(min, size, index);
    let child = children[index]
        .take()
        .unwrap_or_else(|| {
            Box::new(OctreeNode::Branch {
                children: empty_children(),
                aggregate: AIR,
            })
        });
    children[index] = Some(set_block_node(
        child,
        position,
        block,
        child_min,
        child_size,
        depth + 1,
    ));

    if let OctreeNode::Branch {
        children,
        aggregate,
    } = &mut *node
    {
        *aggregate = compute_aggregate(children);
    }
    node
}

fn remove_block_node(
    node: Option<Box<OctreeNode>>,
    position: BlockPos,
    min: i32,
    size: i32,
    depth: u8,
) -> Option<Box<OctreeNode>> {
    let mut node = node?;
    if depth >= MAX_DEPTH || size <= 1 {
        return None;
    }

    let OctreeNode::Branch { children, .. } = &mut *node else {
        return None;
    };

    let index = child_index(position.0, min, size);
    let (child_min, child_size) = child_bounds(min, size, index);
    children[index] = remove_block_node(
        children[index].take(),
        position,
        child_min,
        child_size,
        depth + 1,
    );

    if children.iter().all(|child| child.is_none()) {
        return None;
    }

    if let OctreeNode::Branch {
        children,
        aggregate,
    } = &mut *node
    {
        *aggregate = compute_aggregate(children);
    }
    Some(node)
}

fn compute_aggregate(children: &[Option<Box<OctreeNode>>; 8]) -> BlockId {
    let mut found = AIR;
    let mut mixed = false;
    for child in children.iter().flatten() {
        let block = node_aggregate(child);
        if block == AIR {
            continue;
        }
        if found == AIR {
            found = block;
        } else if found != block {
            mixed = true;
        }
    }
    if found == AIR {
        AIR
    } else if mixed {
        found
    } else {
        found
    }
}

fn node_aggregate(node: &OctreeNode) -> BlockId {
    match node {
        OctreeNode::Leaf(block) => *block,
        OctreeNode::Branch { aggregate, .. } => *aggregate,
    }
}

fn visit_region<F>(
    node: &OctreeNode,
    min: i32,
    size: i32,
    depth: u8,
    region_min: IVec3,
    region_max: IVec3,
    f: &mut F,
) where
    F: FnMut(BlockPos, BlockId),
{
    let node_min = IVec3::new(min, min, min);
    let node_max = IVec3::new(min + size, min + size, min + size);
    if region_max.x <= node_min.x
        || region_max.y <= node_min.y
        || region_max.z <= node_min.z
        || region_min.x >= node_max.x
        || region_min.y >= node_max.y
        || region_min.z >= node_max.z
    {
        return;
    }

    match node {
        OctreeNode::Leaf(block) => {
            if *block != AIR {
                let pos = BlockPos::new(min, min, min);
                if pos.0.x >= region_min.x
                    && pos.0.y >= region_min.y
                    && pos.0.z >= region_min.z
                    && pos.0.x < region_max.x
                    && pos.0.y < region_max.y
                    && pos.0.z < region_max.z
                {
                    f(pos, *block);
                }
            }
        }
        OctreeNode::Branch { children, .. } => {
            if depth >= MAX_DEPTH || size <= 1 {
                return;
            }
            for index in 0..8 {
                if let Some(child) = &children[index] {
                    let (child_min, child_size) = child_bounds(min, size, index);
                    visit_region(
                        child,
                        child_min,
                        child_size,
                        depth + 1,
                        region_min,
                        region_max,
                        f,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_round_trip() {
        let mut world = SparseVoxelOctree::default();
        let pos = BlockPos::new(1, 2, 3);
        world.set_block(pos, 2);
        assert_eq!(world.get_block(pos), 2);
        world.set_block(pos, AIR);
        assert_eq!(world.get_block(pos), AIR);
    }

    #[test]
    fn aggregate_propagates_to_root() {
        let mut world = SparseVoxelOctree::default();
        world.set_block(BlockPos::new(4, 4, 4), 3);
        let root = world.root.as_ref().expect("root exists");
        match root.as_ref() {
            OctreeNode::Branch { aggregate, .. } => assert_ne!(*aggregate, AIR),
            OctreeNode::Leaf(_) => panic!("expected branch root"),
        }
    }

    #[test]
    fn sparse_air_uses_no_storage() {
        let world = SparseVoxelOctree::default();
        assert!(world.root.is_none());
        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), AIR);
    }
}
