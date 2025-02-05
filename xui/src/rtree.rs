use ahash::AHashSet;
use mint::{Vector2, Vector3};
use rstar::{AABB, RTree, RTreeObject};

use crate::node::UINodeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementBounds {
    position: Vector3<u32>,
    size: Vector2<u32>,

    id: UINodeId,
}

impl ElementBounds {
    pub(crate) fn new(position: Vector3<u32>, size: Vector2<u32>, id: UINodeId) -> Self {
        Self { position, size, id }
    }

    pub fn id(&self) -> &UINodeId {
        &self.id
    }
}

impl RTreeObject for ElementBounds {
    type Envelope = AABB<[f32; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [
                self.position.x as f32,
                self.position.y as f32,
                self.position.z as f32,
            ],
            [
                (self.position.x + self.size.x) as f32,
                (self.position.y + self.size.y) as f32,
                self.position.z as f32,
            ],
        )
    }
}

#[derive(Debug, Default)]
pub struct UINodesRTree {
    rtree: RTree<ElementBounds>,
}

impl UINodesRTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, removed_node_ids: AHashSet<UINodeId>, new_bounds: impl Iterator<Item = ElementBounds>) {
        let nodes_for_remove = self
            .rtree
            .iter()
            .filter_map(|x| {
                if removed_node_ids.contains(&x.id) {
                    Some(x.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for node in nodes_for_remove {
            self.rtree.remove(&node);
        }

        for new_bound in new_bounds {
            self.rtree.insert(new_bound);
        }
    }
}
