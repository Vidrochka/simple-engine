use std::{collections::{HashSet, HashMap}, hash::{DefaultHasher, Hash}, mem};

use mint::{Vector2, Vector3};
use rstar::{RTree, RTreeObject, AABB};
use uuid::Uuid;

use crate::{layer_id::LayerId, layers::{FlexDirection, FlexLayer, Layer, ShapesLayer, StackLayer}, tree::UITree};

#[derive(Debug, Eq)]
pub struct ElementTransform {
    position: Vector3<u32>,
    size: Vector2<u32>,

    /// Используется для обновления RTree
    generation: Uuid,
    layer_id: LayerId,
}

impl Default for ElementTransform {
    fn default() -> Self {
        Self { position: [0, 0, 0].into(), size: [0, 0].into(), generation: Default::default(), layer_id: Default::default() }
    }
}

impl PartialEq for ElementTransform {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.size == other.size
    }
}

impl ElementTransform {
    pub fn new(position: Vector3<u32>, size: Vector2<u32>, generation: Uuid, layer_id: LayerId,) -> Self {
        Self { position, size, generation, layer_id }
    }

    pub fn update(&mut self, transform: ElementTransform,) {
        if self != &transform {
            let _prev = mem::replace(self, transform);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementBounds {
    position: Vector3<u32>,
    size: Vector2<u32>,

    layer_id: LayerId,
}

impl RTreeObject for ElementBounds {
    type Envelope = AABB<[f32; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([
            self.position.x as f32,
            self.position.y as f32,
            self.position.z as f32,
        ], [
            (self.position.x + self.size.x) as f32,
            (self.position.y + self.size.y) as f32,
            self.position.z as f32,
        ])
    }
}

#[derive(Debug)]
pub struct UIDefaultRender {
    size: Vector2<u32>,

    element_transform_cache: HashMap<LayerId, ElementTransform>,

    generation: Uuid,

    rtree: RTree<ElementBounds>,
}

impl UIDefaultRender {
    pub fn new(size: Vector2<u32>) -> Self {
        Self {
            size,
            element_transform_cache: Default::default(),
            generation: Uuid::new_v4(),
            rtree: Default::default(),
        }
    }

    pub fn recalculate_transform(&mut self, view: &UITree) {
        let root_id = view.root_id();

        self.generation = Uuid::new_v4();

        self.recalculate_layer_transform(
            view,
            Vector3::from([0, 0, 0]),
            root_id,
        );

        let view_layer_ids = view.ids().cloned().collect::<HashSet<_>>();

        let removed_elements = self.element_transform_cache.keys().filter(|x| view_layer_ids.contains(x))
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|x| self.element_transform_cache.remove(&x))
            .collect::<Vec<_>>();

        let removed_elements_id = removed_elements.iter().map(|x| x.layer_id.clone()).collect::<Vec<_>>();

        // Search for elements that was changed in last generation
        let new_bounds = self.element_transform_cache.iter().filter_map(|x| {
            if x.1.generation == self.generation {
                Some(ElementBounds {
                    position: x.1.position,
                    size: x.1.size,
                    layer_id: x.0.clone(),
                })
            } else {
                None
            }
        }).collect::<Vec<_>>();

        let updated_elements_id = new_bounds.iter().map(|x| x.layer_id.clone()).collect::<Vec<_>>();

        let affected_elements_id = removed_elements_id.into_iter().chain(updated_elements_id.into_iter()).collect::<HashSet<_, ahash::RandomState>>();

        let nodes_for_remove = self.rtree.iter().filter_map(|x| {
            if affected_elements_id.contains(&x.layer_id) {
                Some(x.clone())
            } else {
                None
            }
        }).collect::<Vec<_>>();

        for node in nodes_for_remove {
            self.rtree.remove(&node);
        }

        for new_bound in new_bounds {
            self.rtree.insert(new_bound);
        }
    }

    pub fn recalculate_layer_transform(&mut self, view: &UITree, offset: Vector3<u32>, layer_id: &LayerId) -> Vector2<u32> {
        let layer = view.layer(layer_id).unwrap();

        match layer {
            Layer::Shape(shapes_layer) => self.recalculate_shape_transform(shapes_layer, offset),
            Layer::Flex(flex_layer) => self.recalculate_flex_transform(flex_layer, view, offset),
            Layer::Stack(stack_layer) => self.recalculate_stack_transform(stack_layer, view, offset),
        }
    }

    fn recalculate_shape_transform(
        &mut self,
        shape_layer: &ShapesLayer,
        offset: Vector3<u32>,
    ) -> Vector2<u32> {
        let bounds = shape_layer.bounds();

        self.update_layer_transform(shape_layer.id.clone(), offset, bounds);

        bounds
    }

    fn recalculate_stack_transform(
        &mut self,
        stack_layer: &StackLayer,
        view: &UITree,
        offset: Vector3<u32>,
    ) -> Vector2<u32> {
        let mut bounds = Vector2::from([0, 0]);

        for layer_id in &stack_layer.layers {
            //TODO: add stack size as inner bounds
            let layer_bounds = self.recalculate_layer_transform(
                view,
                offset,
                layer_id,
            );

            bounds.x = bounds.x.max(layer_bounds.x);
            bounds.y = bounds.y.max(layer_bounds.y);
        }

        self.update_layer_transform(stack_layer.id.clone(), offset, bounds);

        bounds
    }

    fn recalculate_flex_transform(
        &mut self,
        flex_layer: &FlexLayer,
        view: &UITree,
        offset: Vector3<u32>,
    ) -> Vector2<u32> {
        let mut bounds = Vector2::from([0, 0]);

        for (idx, layer_id) in flex_layer.layers.iter().enumerate() {
            match flex_layer.justify_content {
                crate::layers::JustifyContent::Start => {
                    let layer_offset = match flex_layer.direction {
                        FlexDirection::Horizontal => {
                            Vector3::from([bounds.x + offset.x, offset.y, offset.z])
                        }
                        FlexDirection::Vertical => {
                            Vector3::from([offset.x, bounds.y + offset.y, offset.z])
                        }
                    };

                    let layer_bounds = self.recalculate_layer_transform(
                        view,
                        layer_offset,
                        layer_id,
                    );

                    match flex_layer.direction {
                        FlexDirection::Horizontal => {
                            bounds.x += layer_bounds.x;
                            bounds.y = bounds.y.max(layer_bounds.y);

                            if idx != flex_layer.layers.len() - 1 {
                                bounds.x += flex_layer.gap as u32;
                            }
                        }
                        FlexDirection::Vertical => {
                            bounds.y += layer_bounds.y;
                            bounds.x = bounds.x.max(layer_bounds.x);

                            if idx != flex_layer.layers.len() - 1 {
                                bounds.y += flex_layer.gap as u32;
                            }
                        }
                    }
                }
                crate::layers::JustifyContent::End => todo!(),
                crate::layers::JustifyContent::SpaceBetween => todo!(),
            }
        }

        self.update_layer_transform(flex_layer.id.clone(), offset, bounds);

        bounds
    }

    fn update_layer_transform(&mut self, layer_id: LayerId, offset: Vector3<u32>, size: Vector2<u32>) {
        let prev = self.element_transform_cache.entry(layer_id.clone()).or_default();

        prev.update(ElementTransform::new(offset, size, self.generation, layer_id));
    }
}