use std::{collections::{HashSet, VecDeque}, sync::Arc};

use ahash::{AHashMap, AHashSet};
use mint::{Vector2, Vector3};
use parking_lot::{RwLock, RwLockReadGuard};
use rstar::RTree;
use uuid::Uuid;

use crate::{node::UINodeId, rtree::{ElementBounds, UINodesRTree}, style::{BoxBounds, FlexDirection, NodeBounds, UIMaterial}, tree::UITree};

#[derive(Debug, Eq)]
pub struct NodeTransform {
    position: Vector3<u32>,
    bounds: NodeBounds,

    /// Используется для обновления RTree
    generation: Uuid,
    id: UINodeId,
}

impl NodeTransform {
    pub fn update(&mut self, transform: NodeTransform) {
        if self != &transform {
            let _prev = std::mem::replace(self, transform);
        }
    }
}

impl PartialEq for NodeTransform {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position &&
        self.bounds == other.bounds
    }
}

impl Default for NodeTransform {
    fn default() -> Self {
        Self { position: [0, 0, 0].into(), bounds: Default::default(), generation: Default::default(), id: "".into() }
    }
}

#[derive(Debug)]
pub struct UIView {
    size: Vector2<u32>,
    ui_tree: Arc<RwLock<UITree>>,

    transform_cache: AHashMap<UINodeId, NodeTransform>,

    // R-Tree для быстрого поиска по элементам (е примеру детект событий)
    rtree: UINodesRTree,

    last_generation: Uuid,
}

impl UIView {
    pub fn new(size: Vector2<u32>, ui_tree: Arc<RwLock<UITree>>) -> Self {
        Self {
            size,
            ui_tree,
            transform_cache: Default::default(),
            rtree: Default::default(),
            last_generation: Default::default(),
        }
    }

    pub fn resize(&mut self, size: Vector2<u32>) {
        self.size = size;

        self.recalculate_transform();
    }

    pub fn recalculate_transform(&mut self) {
        let ui_tree = self.ui_tree.read();

        let ui_tree = if ui_tree.need_style_recalculation() {
            drop(ui_tree);
            self.ui_tree.write().recalculate_styles();
            self.ui_tree.read()
        } else {
            ui_tree
        };

        self.last_generation = Uuid::new_v4();

        let window_bounds = NodeBounds {
            size: self.size,
            inner_space: BoxBounds {
                size: self.size,
                offset: [0,0].into(),
            },
            outer_space: BoxBounds {
                size: self.size,
                offset: [0,0].into(),
            },
        };

        let mut nodes_stack = ui_tree.root_ids().cloned().collect::<VecDeque<_>>();


        // BFS
        while let Some(node_id) = nodes_stack.pop_front() {
            let node = ui_tree.get_node(&node_id).unwrap();

            // TODO: доработать возможность определить размер исходя из дочерних элементов

            let (parent_offset, parent_bounds, prev_siblings, direction) = if let Some(parent_node_id) = node.parent() {
                let parent_transform = self.transform_cache.get(parent_node_id).unwrap();

                let parent_node = ui_tree.get_node(parent_node_id).unwrap();

                let siblings = parent_node.children().iter().take_while(|x| **x != node_id).cloned().collect::<Vec<_>>();

                (parent_transform.position, &parent_transform.bounds, siblings, parent_node.style().flex_direction)
            } else {
                ([0, 0, 0].into(), &window_bounds, vec![], FlexDirection::Col)
            };

            let bounds = node.style().calc_bounds(parent_bounds.inner_space.size);

            let (x_prev_siblings_offset, y_prev_siblings_offset) = if direction == FlexDirection::Col {
                (
                    0,
                    prev_siblings.iter().map(|x| self.transform_cache.get(x).unwrap().bounds.outer_space.size.y).sum::<u32>(),
                )
            } else {
                (
                    prev_siblings.iter().map(|x| self.transform_cache.get(x).unwrap().bounds.outer_space.size.x).sum::<u32>(),
                    0,
                )
            };

            let transform = NodeTransform {
                position: Vector3::from([parent_offset.x + x_prev_siblings_offset, parent_offset.y + y_prev_siblings_offset, parent_offset.z]),
                bounds,
                generation: self.last_generation,
                id: node_id.clone(),
            };

            let prev = self.transform_cache.entry(node_id).or_default();

            prev.update(transform);

            if !node.is_leaf() {
                nodes_stack.extend(node.children().iter().cloned().collect::<Vec<_>>());
            }
        }

        let ui_tree_node_ids = ui_tree.ids().cloned().collect::<AHashSet<_>>();

        let removed_node_ids = self.transform_cache.keys().filter(|x| !ui_tree_node_ids.contains(x))
            .cloned()
            .collect::<HashSet<_>>();

        for removed_node_id in &removed_node_ids {
            self.transform_cache.remove(removed_node_id);
        }

         // Если элемент был изменен в последней генерации - требуется его обновление в RTree
         let new_bounds = self.transform_cache.iter().filter_map(|x| {
            if x.1.generation == self.last_generation {
                Some(ElementBounds::new(x.1.position, x.1.bounds.size, x.0.clone()))
            } else {
                None
            }
        }).collect::<Vec<_>>();

        let updated_node_ids = new_bounds.iter().map(|x| x.id().clone()).collect::<Vec<_>>();

        let affected_node_ids = removed_node_ids.into_iter().chain(updated_node_ids.into_iter()).collect::<AHashSet<_>>();

        self.rtree.update(affected_node_ids, new_bounds.into_iter());
    }

    pub fn build_draw_commands(&mut self, writer: &mut impl IUIWriter) {
        let ui_tree = self.ui_tree.read();

        let mut nodes_stack = ui_tree.root_ids().cloned().collect::<VecDeque<_>>();

        // BFS
        while let Some(node_id) = nodes_stack.pop_front() {
            let node = ui_tree.get_node(&node_id).unwrap();
            let transform = self.transform_cache.get(&node_id).unwrap();

            let material_name = writer.add_material(UIMaterial {
                color: node.style().background_color.unwrap_or_else(|| [0,0,0].into())
            });

            let points = vec![
                Vector3::from([transform.position.x, transform.position.y, 0]),
                Vector3::from([transform.position.x + transform.bounds.size.x, transform.position.y, 0]),
                Vector3::from([transform.position.x + transform.bounds.size.x, transform.position.y + transform.bounds.size.y, 0]),
                Vector3::from([transform.position.x, transform.position.y + transform.bounds.size.y, 0]),
            ];

            let flat_ponts = points.iter().flat_map(|p| vec![p.x as f32, p.y as f32]).collect::<Vec<_>>();

            let indexes = earcutr::earcut(&flat_ponts, &[], 2)
                .ok()
                .map(|x| x.into_iter().map(|x| x as u16).collect())
                .unwrap();

            writer.write_shape(
                &node_id,
                points,
                indexes,
                material_name
            );

            if !node.is_leaf() {
                nodes_stack.extend(node.children().iter().cloned().collect::<Vec<_>>());
            }
        }
    }
}

pub trait IUIWriter {
    /// Add new shape to writer
    fn write_shape(
        &mut self,
        id: &UINodeId,
        points: Vec<Vector3<u32>>,
        indexes: Vec<u16>,
        material_name: String,
    );

    /// Add ui material and return material name (Suggestion: add material cache)
    fn add_material(&mut self, material: UIMaterial) -> String;
}