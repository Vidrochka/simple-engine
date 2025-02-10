use ahash::AHashSet;
use mint::{Vector2, Vector3};
use rstar::{AABB, RTree, RTreeObject};

use crate::node::UINodeId;

#[derive(Debug, Clone)]
pub struct ElementBounds {
    position: Vector3<f32>,
    size: Vector2<f32>,

    id: UINodeId,
}

impl Eq for ElementBounds {}

impl PartialEq for ElementBounds {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.size == other.size && self.id == other.id
    }
}

impl ElementBounds {
    pub(crate) fn new(position: Vector3<f32>, size: Vector2<f32>, id: UINodeId) -> Self {
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
                self.position.x,
                self.position.y,
                self.position.z,
            ],
            [
                self.position.x + self.size.x,
                self.position.y + self.size.y,
                self.position.z,
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
