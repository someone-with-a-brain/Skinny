pub mod config;
pub mod generator;
pub mod spiral;

use config::SkinnyConfig;
use generator::{PreGenTracker, TaskState};
use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};
use spiral::Shape;
use tracing::info;

pub struct SkinnyPlugin {
    config: SkinnyConfig,
    tracker: PreGenTracker,
}

impl Plugin for SkinnyPlugin {
    fn new() -> Self {
        let config = SkinnyConfig::default();
        let tracker = PreGenTracker::new(&config);
        Self { config, tracker }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Skinny World Pre-Generator".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Someone_with_a_brain".into()],
            description: "Native PumpkinMC world chunk pre-generator (Chunky equivalent). Created by Someone_with_a_brain with AI pair-programming assistance.".into(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&self, context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("============================================================");
        info!(" Skinny World Pre-Generator v1.0.0 Initialized");
        info!(" Author: Someone_with_a_brain (AI-assisted)");
        info!(" Repository: https://github.com/Someone-with-a-brain/Skinny");
        info!(" Target Engine: PumpkinMC Native WASM API");
        info!(
            " Default Bounds: Center ({}, {}) | Radius: {} blocks | Shape: {:?}",
            self.config.center_x, self.config.center_z, self.config.radius_blocks, self.config.shape
        );
        info!(" Use /skinny [center|radius|shape|start|pause|cancel|progress] to operate.");
        info!("============================================================");

        // Register /skinny command with description and permission node for PumpkinMC command dispatcher
        let cmd = pumpkin_plugin_api::command::Command::new(
            &["skinny".to_string()],
            "Skinny World Pre-Generator administration command (/skinny center|radius|shape|start|pause|continue|cancel|progress)",
        );
        context.register_command(cmd, "skinny.admin");

        Ok(())
    }
}

impl SkinnyPlugin {
    pub fn handle_command(&mut self, args: &[&str]) -> String {
        if args.is_empty() || args[0].eq_ignore_ascii_case("help") {
            return "Usage: /skinny [center <x> <z> | radius <blocks> | shape <square|circle> | start | pause | continue | cancel | progress]".into();
        }

        match args[0].to_lowercase().as_str() {
            "center" => {
                if args.len() >= 3 {
                    if let (Ok(x), Ok(z)) = (args[1].parse::<i32>(), args[2].parse::<i32>()) {
                        self.config.center_x = x;
                        self.config.center_z = z;
                        self.tracker = PreGenTracker::new(&self.config);
                        return format!("Skinny center updated to X: {}, Z: {}", x, z);
                    }
                }
                "Usage: /skinny center <x> <z>".into()
            }
            "radius" => {
                if args.len() >= 2 {
                    if let Ok(r) = args[1].parse::<i32>() {
                        self.config.radius_blocks = r;
                        self.tracker = PreGenTracker::new(&self.config);
                        return format!("Skinny pre-generation radius set to {} blocks ({} chunks)", r, (r + 15) / 16);
                    }
                }
                "Usage: /skinny radius <blocks>".into()
            }
            "shape" => {
                if args.len() >= 2 {
                    match args[1].to_lowercase().as_str() {
                        "square" => {
                            self.config.shape = Shape::Square;
                            self.tracker = PreGenTracker::new(&self.config);
                            return "Skinny shape set to Square.".into();
                        }
                        "circle" => {
                            self.config.shape = Shape::Circle;
                            self.tracker = PreGenTracker::new(&self.config);
                            return "Skinny shape set to Circle.".into();
                        }
                        _ => {}
                    }
                }
                "Usage: /skinny shape <square|circle>".into()
            }
            "start" => {
                self.tracker.start();
                format!("Started Skinny pre-generation task! Total chunks to generate: {}", self.tracker.spiral.total_chunks())
            }
            "pause" => {
                self.tracker.pause();
                "Paused Skinny pre-generation task.".into()
            }
            "continue" | "resume" => {
                self.tracker.resume();
                "Resumed Skinny pre-generation task.".into()
            }
            "cancel" | "stop" => {
                self.tracker.cancel();
                "Cancelled Skinny pre-generation task.".into()
            }
            "progress" | "status" => {
                let state = match self.tracker.state {
                    TaskState::Idle => "IDLE",
                    TaskState::Running => "RUNNING",
                    TaskState::Paused => "PAUSED",
                    TaskState::Completed => "COMPLETED",
                };
                let pct = self.tracker.spiral.progress_percentage();
                let current = self.tracker.spiral.current_index();
                let total = self.tracker.spiral.total_chunks();
                let cps = self.tracker.current_cps;
                let eta = self.tracker.eta_seconds();

                format!(
                    "Skinny Task Status: [{}] | Progress: {:.1}% ({}/{}) | Rate: {:.1} CPS | ETA: {}s",
                    state, pct, current, total, cps, eta
                )
            }
            _ => "Unknown command. Use /skinny help for commands.".into(),
        }
    }
}

pumpkin_plugin_api::register_plugin!(SkinnyPlugin);
