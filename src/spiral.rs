use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Square,
    Circle,
}

/// 2D Chunk coordinate spiral generator.
#[derive(Debug, Clone)]
pub struct ChunkSpiral {
    center_x: i32,
    center_z: i32,
    radius_chunks: i32,
    shape: Shape,
    current_index: usize,
    total_chunks: usize,
    chunks: Vec<(i32, i32)>,
}

impl ChunkSpiral {
    pub fn new(center_x: i32, center_z: i32, radius_blocks: i32, shape: Shape) -> Self {
        let radius_chunks = (radius_blocks + 15) / 16;
        let mut chunks = Vec::new();

        let min_cx = (center_x - radius_blocks) >> 4;
        let max_cx = (center_x + radius_blocks) >> 4;
        let min_cz = (center_z - radius_blocks) >> 4;
        let max_cz = (center_z + radius_blocks) >> 4;

        let center_chunk_x = center_x >> 4;
        let center_chunk_z = center_z >> 4;
        let radius_sq = (radius_chunks * radius_chunks) as f64;

        for cx in min_cx..=max_cx {
            for cz in min_cz..=max_cz {
                if shape == Shape::Circle {
                    let dx = (cx - center_chunk_x) as f64;
                    let dz = (cz - center_chunk_z) as f64;
                    if (dx * dx + dz * dz) > radius_sq {
                        continue;
                    }
                }
                chunks.push((cx, cz));
            }
        }

        // Sort by distance from center chunk outward
        chunks.sort_by_key(|&(cx, cz)| {
            let dx = cx - center_chunk_x;
            let dz = cz - center_chunk_z;
            dx * dx + dz * dz
        });

        let total_chunks = chunks.len();

        Self {
            center_x,
            center_z,
            radius_chunks,
            shape,
            current_index: 0,
            total_chunks,
            chunks,
        }
    }

    pub fn next_chunk(&mut self) -> Option<(i32, i32)> {
        if self.current_index < self.chunks.len() {
            let chunk = self.chunks[self.current_index];
            self.current_index += 1;
            Some(chunk)
        } else {
            None
        }
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn total_chunks(&self) -> usize {
        self.total_chunks
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total_chunks == 0 {
            100.0
        } else {
            (self.current_index as f64 / self.total_chunks as f64) * 100.0
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

