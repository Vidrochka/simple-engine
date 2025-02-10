use ahash::AHashMap;
use derive_builder::Builder;
use mint::{Vector2, Vector3, Vector4};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{node::UINodeId, view::ContentBox};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UIStyle {
    pub id: UIStyleId,
    pub class: String,
    pub rules: UIStyleRules
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Builder)]
#[builder(try_setter, default, setter(into, strip_option))]
pub struct UIStyleRules {
    pub display: Display,
    pub background_color: Option<Vector3<u8>>,

    pub flex_direction: FlexDirection,

    pub gap: Option<Unit>,

    pub margin: Margin,

    pub padding: Padding,

    pub width: Option<SizeValue>,
    pub height: Option<SizeValue>,
}

impl UIStyleRules {
    /// Высчитывает контент относительно размера
    pub (crate) fn calc_content_box(&self, size: Vector2<f32>, parent_size: Vector2<f32>) -> ContentBox {
        /// Если размер по контенту, фактическое пространство с которого рассчитывается размер вложенного контента - внешний родитель
        let size: Vector2<f32> = [
            if let Some(SizeValue::FitContent) = self.width {
                // parent_size.x
                size.x
            } else {
                size.x
            },
            if let Some(SizeValue::FitContent) = self.height {
                // parent_size.y
                size.y
            } else {
                size.y
            },
        ].into();

        let half_width = size.x / 2.0;
        let half_height = size.y / 2.0;

        ContentBox {
            left: half_width - self.padding.left.unwrap_or(Unit::Pixel(0)).calc(parent_size.x),
            right: half_width - self.padding.right.unwrap_or(Unit::Pixel(0)).calc(parent_size.x),
            top: half_height - self.padding.top.unwrap_or(Unit::Pixel(0)).calc(parent_size.y),
            bottom: half_height - self.padding.bottom.unwrap_or(Unit::Pixel(0)).calc(parent_size.y),
        }
    }

    /// Высчитывает контент относительно размера
    pub (crate) fn calc_outer_box(&self, size: Vector2<f32>, parent_size: Vector2<f32>) -> ContentBox {
        let half_width = size.x / 2.0;
        let half_height = size.y / 2.0;

        ContentBox {
            left: half_width + self.margin.left.unwrap_or(Unit::Pixel(0)).calc(parent_size.x),
            right: half_width + self.margin.right.unwrap_or(Unit::Pixel(0)).calc(parent_size.x),
            top: half_height + self.margin.top.unwrap_or(Unit::Pixel(0)).calc(parent_size.y),
            bottom: half_height + self.margin.bottom.unwrap_or(Unit::Pixel(0)).calc(parent_size.y),
        }
    }

    // pub(crate) fn calc_bounds(&self, available_space: Vector2<u32>) -> NodeBounds {
    //     let width = if let Some(width) = self.width {
    //         width.calc(available_space.x)
    //     } else {
    //         available_space.x
    //     };

    //     let height = if let Some(height) = self.height {
    //         height.calc(available_space.y)
    //     } else {
    //         available_space.y
    //     };

    //     let size = Vector2 { x: width, y: height };

    //     NodeBounds {
    //         inner_size: size,
    //         content_size: self.padding.calc_inner_size(size),
    //         outer_bounds: self.margin.calc_outer_size(size),
    //     }
    // }

    pub fn merge<'a>(styles: impl Iterator<Item = &'a UIStyleRules>) -> UIStyleRules {
        styles.fold(UIStyleRules::default(), |mut merged_style, style| {
            merged_style.display = style.display;

            tracing::info!("Add style: {style:?}");

            if let Some(background_color) = style.background_color {
                merged_style.background_color = Some(background_color);
            }

            if let Some(gap) = style.gap {
                merged_style.gap = Some(gap);
            }

            if let Some(mb) = style.margin.bottom {
                merged_style.margin.bottom = Some(mb);
            }

            if let Some(mt) = style.margin.top {
                merged_style.margin.top = Some(mt);
            }

            if let Some(ml) = style.margin.left {
                merged_style.margin.left = Some(ml);
            }
            
            if let Some(mr) = style.margin.right {
                merged_style.margin.right = Some(mr);
            }

            if let Some(pb) = style.padding.bottom {
                merged_style.padding.bottom = Some(pb);
            }

            if let Some(pt) = style.padding.top {
                merged_style.padding.top = Some(pt);
            }

            if let Some(pl) = style.padding.left {
                merged_style.padding.left = Some(pl);
            }
            
            if let Some(pr) = style.padding.right {
                merged_style.padding.right = Some(pr);
            }

            if let Some(width) = style.width {
                merged_style.width = Some(width);
            }

            if let Some(height) = style.height {
                merged_style.height = Some(height);
            }

            merged_style
        })
    } 
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Builder)]
#[builder(try_setter, setter(into))]
pub struct Margin {
    pub left: Option<Unit>,
    pub right: Option<Unit>,
    pub bottom: Option<Unit>,
    pub top: Option<Unit>,
}

impl Margin {
    // pub fn calc_outer_size(&self, size: Vector2<u32>) -> BoxBounds {
    //     fn calc_margin_offset(size: u32, margin: Option<Unit>) -> u32 {
    //         if let Some(margin) = margin {
    //             margin.calc(size)
    //         } else {
    //             0
    //         }
    //     };

    //     let lm = calc_margin_offset(size.x, self.left);
    //     let rm = calc_margin_offset(size.x, self.right);
    //     let tm = calc_margin_offset(size.y, self.top);
    //     let bm = calc_margin_offset(size.y, self.bottom);

    //     BoxBounds {
    //         size: Vector2 {
    //             x: size.x + lm + rm,
    //             y: size.y + tm + bm,
    //         },
    //         offset: Vector2 {
    //             x: lm,
    //             y: tm,
    //         }
    //     }
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Builder)]
#[builder(try_setter, setter(into))]
pub struct Padding {
    pub left: Option<Unit>,
    pub right: Option<Unit>,
    pub bottom: Option<Unit>,
    pub top: Option<Unit>,
}

impl Padding {
    // pub fn calc_inner_size(&self, size: Vector2<u32>) -> BoxBounds {
    //     fn calc_padding_offset(size: u32, padding: Option<Unit>) -> u32 {
    //         if let Some(padding) = padding {
    //             padding.calc(size)
    //         } else {
    //             0
    //         }
    //     };

    //     let lp = calc_padding_offset(size.x, self.left);
    //     let rp = calc_padding_offset(size.x, self.right);
    //     let tp = calc_padding_offset(size.y, self.top);
    //     let bp = calc_padding_offset(size.y, self.bottom);

    //     let (lp, rp) = if lp + rp > size.x {
    //         let row_size = lp + rp;

    //         (
    //             size.x / row_size * lp,
    //             size.x / row_size * rp
    //         )
    //     } else {
    //         (lp, rp)
    //     };

    //     let (tp, bp) = if tp + bp > size.y {
    //         let col_size = tp + bp;

    //         (
    //             size.y / col_size * tp,
    //             size.y / col_size * bp
    //         )
    //     } else {
    //         (tp, bp)
    //     };


    //     BoxBounds {
    //         size: Vector2 {
    //             x: size.x - lp - rp,
    //             y: size.y - tp - bp,
    //         },
    //         offset: Vector2 {
    //             x: lp,
    //             y: tp,
    //         }
    //     }
    // }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum FlexDirection {
    #[default]
    Row,
    Col,
}



#[derive(Debug, PartialEq, Eq)]
pub struct BoxBounds {
    pub (crate) size: Vector2<u32>,
    pub (crate) offset: Vector2<u32>,
}

impl Default for BoxBounds {
    fn default() -> Self {
        Self { size: [0,0].into(), offset: [0,0].into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Display {
    // Block,
    #[default]
    Flex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    Pixel(u32),
    Percent(u16),
}

impl Unit {
    pub fn calc(&self, size: f32) -> f32 {
        match self {
            Unit::Pixel(px) => *px as f32,
            Unit::Percent(percent) => size / 100.0 * (*percent as f32).min(100.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SizeValue {
    #[default]
    Auto,
    FitContent,
    Unit(Unit)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleClass {
    node_ids: Vec<UINodeId>,
    rules: AHashMap<UIStyleId, UIStyleRules>,
}

impl StyleClass {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_styles(&mut self, id: UIStyleId, rule: UIStyleRules) {
        self.rules.insert(id, rule);
    }

    pub fn add_node(&mut self, id: UINodeId) {
        self.node_ids.push(id);
    }

    pub fn node_ids(&self) -> &[UINodeId] {
        &self.node_ids
    }

    pub fn styles(&self) -> impl Iterator<Item = &UIStyleRules> {
        self.rules.values()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct UIStyleId(String);

impl UIStyleId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl<T: ToString> From<T> for UIStyleId {
    fn from(value: T) -> Self {
        UIStyleId(value.to_string())
    }
} 

#[derive(Debug, Hash)]
pub struct UIMaterial {
    pub color: Vector3<u8>,
}