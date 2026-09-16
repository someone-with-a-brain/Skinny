use serde::{Deserialize, Serialize};
use crate::config::SkinnyConfig;
use crate::spiral::ChunkSpiral;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskState {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone)]
pub struct PreGenTracker {
    pub state: TaskState,
    pub spiral: ChunkSpiral,
    pub start_timestamp: u64,
    pub generated_chunks: usize,
    pub last_cps_check: u64,
    pub last_cps_chunks: usize,
    pub current_cps: f64,
}

impl PreGenTracker {
    pub fn new(config: &SkinnyConfig) -> Self {
        let spiral = ChunkSpiral::new(
            config.center_x,
            config.center_z,
            config.radius_blocks,
            config.shape,
        );

        Self {
            state: TaskState::Idle,
            spiral,
            start_timestamp: 0,
            generated_chunks: 0,
            last_cps_check: 0,
            last_cps_chunks: 0,
            current_cps: 0.0,
        }
    }

    pub fn start(&mut self) {
        self.state = TaskState::Running;
        self.start_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_cps_check = self.start_timestamp;
    }

    pub fn pause(&mut self) {
        if self.state == TaskState::Running {
            self.state = TaskState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == TaskState::Paused {
            self.state = TaskState::Running;
        }
    }

    pub fn cancel(&mut self) {
        self.state = TaskState::Idle;
        self.spiral.reset();
        self.generated_chunks = 0;
        self.current_cps = 0.0;
    }

    pub fn update_progress(&mut self, chunks_processed: usize) {
        self.generated_chunks += chunks_processed;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let elapsed = now.saturating_sub(self.last_cps_check);
        if elapsed >= 1 {
            let chunk_diff = self.generated_chunks.saturating_sub(self.last_cps_chunks);
            self.current_cps = chunk_diff as f64 / elapsed as f64;
            self.last_cps_check = now;
            self.last_cps_chunks = self.generated_chunks;
        }

        if self.spiral.current_index() >= self.spiral.total_chunks() {
            self.state = TaskState::Completed;
        }
    }

    pub fn eta_seconds(&self) -> u64 {
        if self.current_cps <= 0.0 {
            return 0;
        }
        let remaining_chunks = self.spiral.total_chunks().saturating_sub(self.spiral.current_index());
        (remaining_chunks as f64 / self.current_cps) as u64
    }
}

