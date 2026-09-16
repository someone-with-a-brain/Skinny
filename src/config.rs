use serde::{Deserialize, Serialize};
use crate::spiral::Shape;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinnyConfig {
    pub center_x: i32,
    pub center_z: i32,
    pub radius_blocks: i32,
    pub shape: Shape,
    pub max_chunks_per_tick: usize,
}

impl Default for SkinnyConfig {
    fn default() -> Self {
        Self {
            center_x: 0,
            center_z: 0,
            radius_blocks: 5000,
            shape: Shape::Square,
            max_chunks_per_tick: 10,
        }
    }
}

