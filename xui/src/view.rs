use std::{collections::{HashMap, HashSet, VecDeque}, sync::Arc};

use ahash::{AHashMap, AHashSet};
use mint::{Vector2, Vector3};
use parking_lot::{RwLock, RwLockReadGuard};
use rstar::RTree;
use uuid::Uuid;

use crate::{node::UINodeId, rtree::{ElementBounds, UINodesRTree}, style::{BoxBounds, FlexDirection, SizeValue, UIMaterial, Unit}, tree::UITree};

#[derive(Debug, Default)]
pub struct ContentBox {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Eq for ContentBox {}

impl PartialEq for ContentBox {
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.right == other.right && self.top == other.top && self.bottom == other.bottom
    }
}

#[derive(Debug)]
pub struct Transform {
    /// Позиция центра элемента
    position_center: Vector3<f32>,

    /// Размер элемента без учета margin/border 
    size: Vector2<f32>,

    /// Внешний размер с учетом margin/border
    outer_size: ContentBox,
    /// Внутренний размер с учетом padding
    content_size: ContentBox,
}

impl Eq for Transform {}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.position_center.x == other.position_center.x &&
        self.position_center.y == other.position_center.y &&
        self.position_center.z == other.position_center.z &&
        self.size.x == other.size.x &&
        self.size.y == other.size.y &&
        self.outer_size == other.outer_size &&
        self.content_size == other.content_size
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position_center: [0.0, 0.0, 0.0].into(),
            size: [0.0, 0.0].into(),
            outer_size: Default::default(),
            content_size: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NodeTransform {
    /// id ноды в дереве
    id: UINodeId,

    /// Используется для обновления RTree
    generation: Uuid,

    /// Трансформация ноды в пространстве
    transform: Transform,
}

impl NodeTransform {
    pub fn new(id: UINodeId, generation: Uuid, transform: Transform) -> Self {
        Self { id, generation, transform }
    }
    
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn set_transform(&mut self, transform: Transform, generation: Uuid) {
        self.generation = generation;
        self.transform = transform;
    }
}


#[derive(Debug)]
pub struct UIView {
    window_size: Vector2<f32>,
    ui_tree: Arc<RwLock<UITree>>,

    transform: AHashMap<UINodeId, NodeTransform>,

    // R-Tree для быстрого поиска по элементам (е примеру детект событий)
    rtree: UINodesRTree,

    last_generation: Uuid,
}

impl UIView {
    pub fn new(size: Vector2<f32>, ui_tree: Arc<RwLock<UITree>>) -> Self {
        Self {
            window_size: size,
            ui_tree,
            transform: Default::default(),
            rtree: Default::default(),
            last_generation: Default::default(),
        }
    }

    pub fn resize(&mut self, size: Vector2<f32>) {
        self.window_size = size;

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

        // let window_bounds = NodeBounds {
        //     inner_size: self.window_size,
        //     content_size: BoxBounds {
        //         size: self.window_size,
        //         offset: [0,0].into(),
        //     },
        //     outer_bounds: BoxBounds {
        //         size: self.window_size,
        //         offset: [0,0].into(),
        //     },
        // };

        /// Просчет идет в 2 этапа
        /// 1 - Вычисление относительный размера контента (ребенок + ребенок -> родитель)
        /// 2 - Вычисление размера контента (родитель ->> (ребенок, ребенок))
        
        #[derive(Debug)]
        pub struct NodeVirtualSize {
            /// Ширина из процентной части и целого числа пикселей
            width: (f32, f32),
            /// Ширина из процентной части и целого числа пикселей
            height: (f32, f32),
        }

        // Размер элементов (фиксированный или % просчитанный от детей)
        let mut virtual_size = HashMap::new();

        for node_id in ui_tree.toposort().rev() {
            tracing::info!("Process (phase 1) node id '{node_id:?}'");

            let node = ui_tree.get_node(node_id).unwrap();

            if ui_tree.is_leaf(node.id()) {
                virtual_size.insert(node_id.clone(), NodeVirtualSize {
                    width: match node.style().width.unwrap_or(SizeValue::Auto) {
                        SizeValue::Auto => (100.0, 0.0),
                        SizeValue::FitContent => (0.0, 0.0),
                        SizeValue::Unit(unit) => match unit {
                            Unit::Pixel(px) => (0.0, px as f32),
                            Unit::Percent(percent) => (percent as f32, 0.0),
                        }
                    },
                    height: match node.style().height.unwrap_or(SizeValue::Auto) {
                        SizeValue::Auto => (100.0, 0.0),
                        SizeValue::FitContent => (0.0, 0.0),
                        SizeValue::Unit(unit) => match unit {
                            Unit::Pixel(px) => (0.0, px as f32),
                            Unit::Percent(percent) => (percent as f32, 0.0),
                        }
                    },
                });
            } else {
                fn calc_offset(mut size: (f32, f32), unit: Option<Unit>) -> (f32, f32) {
                    if let Some(unit) = unit {
                        match unit {
                            Unit::Pixel(px) => size.1 += px as f32,
                            Unit::Percent(percent) => size.0 += percent as f32,
                        }
                    }

                    size
                }

                virtual_size.insert(node_id.clone(), NodeVirtualSize {
                    width: match node.style().width.unwrap_or(SizeValue::Auto) {
                        SizeValue::Auto => (100.0, 0.0),
                        SizeValue::FitContent => {
                            let mut size = if node.style().flex_direction == FlexDirection::Row {
                                let mut size = ui_tree.get_child_node_ids(node.id()).fold((0.0, 0.0), |acc, child_node_id| {
                                    let size = virtual_size.get(child_node_id).unwrap();

                                    (acc.0 + size.width.0, acc.1 + size.width.1)
                                });

                                if let Some(gap) = node.style().gap {
                                        let child_count = ui_tree.get_child_node_ids(node.id()).count();
                                        let gap_count = (child_count - 1) as f32;

                                        match gap {
                                            Unit::Pixel(px) => size.1 += px as f32 * gap_count,
                                            Unit::Percent(percent) => size.0 += percent as f32 * gap_count,
                                        }
                                }

                                size
                            } else {
                                ui_tree.get_child_node_ids(node.id()).fold((0.0, 0.0), |acc, child_node_id| {
                                    let size = virtual_size.get(&child_node_id).unwrap();
    
                                    (if acc.0 > size.width.0 { acc.0 } else { size.width.0 }, if acc.1 > size.width.1 { acc.1 } else { size.width.1 })
                                })
                            };
                            
                            // size = calc_offset(size, node.style().margin.left);
                            // size = calc_offset(size, node.style().margin.right);

                            size = calc_offset(size, node.style().padding.left);
                            size = calc_offset(size, node.style().padding.right);

                            size
                        },
                        SizeValue::Unit(unit) => match unit {
                            Unit::Pixel(px) => (0.0, px as f32),
                            Unit::Percent(percent) => (percent as f32, 0.0),
                        }
                    },
                    height: match node.style().height.unwrap_or(SizeValue::Auto) {
                        SizeValue::Auto => (100.0, 0.0),
                        SizeValue::FitContent => {
                            let mut size = if node.style().flex_direction == FlexDirection::Col {
                                let mut size = ui_tree.get_child_node_ids(node.id()).fold((0.0, 0.0), |acc, child_node_id| {
                                    let size = virtual_size.get(&child_node_id).unwrap();
    
                                    (acc.0 + size.height.0, acc.1 + size.height.1)
                                });
    
                                if let Some(gap) = node.style().gap {
                                        let child_count = ui_tree.get_child_node_ids(node.id()).count();
                                        let gap_count = (child_count - 1) as f32;
    
                                        match gap {
                                            Unit::Pixel(px) => size.1 += px as f32 * gap_count,
                                            Unit::Percent(percent) => size.0 += percent as f32 * gap_count,
                                        }
                                }

                                size
                            } else {
                                ui_tree.get_child_node_ids(node.id()).fold((0.0, 0.0), |acc, child_node_id| {
                                    let size = virtual_size.get(&child_node_id).unwrap();
    
                                    (if acc.0 > size.height.0 { acc.0 } else { size.height.0 }, if acc.1 > size.height.1 { acc.1 } else { size.height.1 })
                                })
                            };

                            // size = calc_offset(size, node.style().margin.bottom);
                            // size = calc_offset(size, node.style().margin.top);

                            size = calc_offset(size, node.style().padding.bottom);
                            size = calc_offset(size, node.style().padding.top);

                            size
                        },
                        SizeValue::Unit(unit) => match unit {
                            Unit::Pixel(px) => (0.0, px as f32),
                            Unit::Percent(percent) => (percent as f32, 0.0),
                        }
                    },
                });
            }
        }

        tracing::info!("Virtual size: [{virtual_size:#?}]");

        // TODO: придумать как сделать нормально
        let mut gap_cache = HashMap::new();
        
        for node_id in ui_tree.toposort() {
            tracing::info!("Process (phase 2) node id '{node_id:?}'");

            let node = ui_tree.get_node(node_id).unwrap();
            
            let node_virtual_size = virtual_size.get(node_id).unwrap();

            let new_transform = if let Some(parent_node_id) = ui_tree.get_parent(node.id()) {
                let parent_node = ui_tree.get_node(parent_node_id).unwrap();

                tracing::info!("Try get parent '{parent_node_id:?}' for '{node_id:?}'");

                let parent_node_transform = self.transform.get(parent_node_id).unwrap();

                // Размер это процент от родителя + абсолютный размер в пикселях
                let size = Vector2::from([
                    (parent_node_transform.transform().content_size.left + parent_node_transform.transform().content_size.right) / 100.0 * node_virtual_size.width.0 + node_virtual_size.width.1,
                    (parent_node_transform.transform().content_size.top + parent_node_transform.transform().content_size.bottom) / 100.0 * node_virtual_size.height.0 + node_virtual_size.height.1
                ]);

                let outer_size = node.style().calc_outer_box(size, [
                    parent_node_transform.transform.content_size.left + parent_node_transform.transform.content_size.right,
                    parent_node_transform.transform.content_size.top + parent_node_transform.transform.content_size.bottom
                ].into());

                let content_size = node.style().calc_content_box(size, [
                    parent_node_transform.transform.content_size.left + parent_node_transform.transform.content_size.right,
                    parent_node_transform.transform.content_size.top + parent_node_transform.transform.content_size.bottom
                ].into());

                let parent_gap = gap_cache.get(parent_node_id).copied().unwrap_or_default();

                tracing::info!("Add parent gap {parent_gap}");

                let sibling_offset: f32 = ui_tree.get_child_node_ids(parent_node.id()).take_while(|x| x != &node_id).map(|x| {
                    tracing::info!("Calc sibling size for '{node_id:?}', take sibling '{x:?}'");
                    
                    let sibling_transform = &self.transform.get(x).unwrap().transform;

                    if parent_node.style().flex_direction == FlexDirection::Row {
                        let parent_margin = parent_node_transform.transform.outer_size.left - (parent_node_transform.transform.size.x / 2.0);

                        sibling_transform.outer_size.left + sibling_transform.outer_size.right + parent_gap
                    } else {

                        let parent_margin = (parent_node_transform.transform.outer_size.top - (parent_node_transform.transform.size.y / 2.0));
                        
                        sibling_transform.outer_size.top + sibling_transform.outer_size.bottom + parent_gap
                    }
                }).sum();

                tracing::info!("Add sibling offset {sibling_offset:?}");

                // let parent_margin_left = (parent_node_transform.transform.outer_size.left - (parent_node_transform.transform.size.x / 2.0));
                // let parent_margin_top = (parent_node_transform.transform.outer_size.top - (parent_node_transform.transform.size.y / 2.0));

                // tracing::info!("Parent margin left '{parent_margin_left}' top '{parent_margin_top}'");

                // Центр родителя - смещение области контента + собственное смещение до центра + смещения других дочерних элементов = центр объекта
                //  ________________________
                // |            + центра родителя
                // | - <-------- смещение к области контента
                // |  _____________         |
                // | |             |        |        
                // | |----> + смещение к центру текущего элемента
                // | |_____________|        |
                // |________________________|
                let position = [
                    (parent_node_transform.transform.position_center.x - parent_node_transform.transform.content_size.left) + outer_size.left +
                        if parent_node.style().flex_direction == FlexDirection::Row {
                            sibling_offset
                        } else {
                            0.0
                        },
                    (parent_node_transform.transform.position_center.y - parent_node_transform.transform.content_size.top) + outer_size.top +
                        if parent_node.style().flex_direction == FlexDirection::Col {
                            sibling_offset
                        } else {
                            0.0
                        },
                    0.0
                ].into();

                tracing::info!(
                    "
                        Parent pos x {}
                        Parent content size {}
                        Outer size left {}
                        Sibling offset {}
                    ",
                    parent_node_transform.transform.position_center.x,
                    parent_node_transform.transform.content_size.left,
                    outer_size.left,
                    sibling_offset,
                );


                tracing::info!("Node '{node_id:?}' position '{position:?}'");

                if node.style().flex_direction == FlexDirection::Row {
                    gap_cache.insert(node_id.clone(), node.style().gap.unwrap_or(Unit::Pixel(0)).calc(parent_node_transform.transform.size.x));
                } else {
                    gap_cache.insert(node_id.clone(), node.style().gap.unwrap_or(Unit::Pixel(0)).calc(parent_node_transform.transform.size.y));
                }

                Transform {
                    position_center: position,
                    size,
                    outer_size,
                    content_size,
                }
            } else {
                let size = Vector2::from([
                    self.window_size.x / 100.0 * node_virtual_size.width.0 + node_virtual_size.width.1,
                    self.window_size.y / 100.0 * node_virtual_size.height.0 + node_virtual_size.height.1
                ]);

                let outer_size = node.style().calc_outer_box(size, self.window_size);

                let content_size = node.style().calc_content_box(size, self.window_size);

                // Смещение внешней границы по сути помещение центра элемента от угла. Тут рут элемент, соответственно угол (0, 0)
                let position = [outer_size.left, outer_size.top, 0.0].into();

                if node.style().flex_direction == FlexDirection::Row {
                    gap_cache.insert(node_id.clone(), node.style().gap.unwrap_or(Unit::Pixel(0)).calc(self.window_size.x));
                } else {
                    gap_cache.insert(node_id.clone(), node.style().gap.unwrap_or(Unit::Pixel(0)).calc(self.window_size.y));
                }

                Transform {
                    position_center: position,
                    size,
                    outer_size,
                    content_size,
                }
            };

            match self.transform.get_mut(node_id) {
                Some(node_transform) => {
                    if node_transform.transform != new_transform {
                        node_transform.set_transform(new_transform, self.last_generation);
                    }
                },
                None => {
                    self.transform.insert(node_id.clone(), NodeTransform::new(
                        node_id.clone(),
                        self.last_generation,
                        new_transform,
                    ));
                },
            }
        }
        
        // let mut nodes_stack = ui_tree.root_ids().cloned().collect::<VecDeque<_>>();

        // // BFS
        // while let Some(node_id) = nodes_stack.pop_front() {
        //     let node = ui_tree.get_node(&node_id).unwrap();

        //     // TODO: доработать возможность определить размер исходя из дочерних элементов

        //     let (parent_offset, parent_bounds, prev_siblings, direction) = if let Some(parent_node_id) = node.parent() {
        //         let parent_transform = self.transform.get(parent_node_id).unwrap();

        //         let parent_node = ui_tree.get_node(parent_node_id).unwrap();

        //         let siblings = parent_node.children().iter().take_while(|x| **x != node_id).cloned().collect::<Vec<_>>();

        //         (parent_transform.position, &parent_transform.bounds, siblings, parent_node.style().flex_direction)
        //     } else {
        //         ([0, 0, 0].into(), &window_bounds, vec![], FlexDirection::Col)
        //     };

        //     let bounds = node.style().calc_bounds(parent_bounds.content_size.size);

        //     let (x_prev_siblings_offset, y_prev_siblings_offset) = if direction == FlexDirection::Col {
        //         (
        //             0,
        //             prev_siblings.iter().map(|x| self.transform.get(x).unwrap().bounds.outer_bounds.size.y).sum::<u32>(),
        //         )
        //     } else {
        //         (
        //             prev_siblings.iter().map(|x| self.transform.get(x).unwrap().bounds.outer_bounds.size.x).sum::<u32>(),
        //             0,
        //         )
        //     };

        //     let transform = NodeTransform {
        //         position: Vector3::from([parent_offset.x + x_prev_siblings_offset, parent_offset.y + y_prev_siblings_offset, parent_offset.z]),
        //         bounds,
        //         generation: self.last_generation,
        //         id: node_id.clone(),
        //     };

        //     let prev = self.transform.entry(node_id).or_default();

        //     prev.update(transform);

        //     if !node.is_leaf() {
        //         nodes_stack.extend(node.children().iter().cloned().collect::<Vec<_>>());
        //     }
        // }

        let ui_tree_node_ids = ui_tree.ids().cloned().collect::<AHashSet<_>>();

        let removed_node_ids = self.transform.keys().filter(|x| !ui_tree_node_ids.contains(x))
            .cloned()
            .collect::<HashSet<_>>();

        for removed_node_id in &removed_node_ids {
            self.transform.remove(removed_node_id);
        }

         // Если элемент был изменен в последней генерации - требуется его обновление в RTree
         let new_bounds = self.transform.iter().filter_map(|x| {
            if x.1.generation == self.last_generation {
                Some(ElementBounds::new(x.1.transform.position_center, [
                    x.1.transform.content_size.left + x.1.transform.content_size.right,
                    x.1.transform.content_size.top + x.1.transform.content_size.bottom
                ].into(), x.0.clone()))
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

        // BFS
        for node_id in ui_tree.toposort() {
            let node = ui_tree.get_node(&node_id).unwrap();
            let node_transform = self.transform.get(&node_id).unwrap();

            let material_name = writer.add_material(UIMaterial {
                color: node.style().background_color.unwrap_or_else(|| [0,0,0].into())
            });

            let points = vec![
                Vector3::from([
                    node_transform.transform.position_center.x - (node_transform.transform.size.x / 2.0),
                    node_transform.transform.position_center.y - (node_transform.transform.size.y / 2.0),
                    0.0
                ]),
                Vector3::from([
                    node_transform.transform.position_center.x + (node_transform.transform.size.x / 2.0),
                    node_transform.transform.position_center.y - (node_transform.transform.size.y / 2.0),
                    0.0
                ]),
                Vector3::from([
                    node_transform.transform.position_center.x + (node_transform.transform.size.x / 2.0),
                    node_transform.transform.position_center.y + (node_transform.transform.size.y / 2.0),
                    0.0
                ]),
                Vector3::from([
                    node_transform.transform.position_center.x - (node_transform.transform.size.x / 2.0),
                    node_transform.transform.position_center.y + (node_transform.transform.size.y / 2.0),
                    0.0
                ]),
            ];

            let flat_points = points.iter().flat_map(|p| vec![p.x, p.y]).collect::<Vec<_>>();

            let indexes = earcutr::earcut(&flat_points, &[], 2)
                .ok()
                .map(|x| x.into_iter().map(|x| x as u16).collect())
                .unwrap();

            writer.write_shape(
                &node_id,
                points,
                indexes,
                material_name
            );
        }
    }
}

pub trait IUIWriter {
    /// Add new shape to writer
    fn write_shape(
        &mut self,
        id: &UINodeId,
        points: Vec<Vector3<f32>>,
        indexes: Vec<u16>,
        material_name: String,
    );

    /// Add ui material and return material name (Suggestion: add material cache)
    fn add_material(&mut self, material: UIMaterial) -> String;
}