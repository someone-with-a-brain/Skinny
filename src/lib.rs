pub mod config;
pub mod generator;
pub mod spiral;

use std::sync::{Arc, Mutex};

use config::SkinnyConfig;
use generator::{PreGenTracker, TaskState};
use pumpkin_plugin_api::command::{
    Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
};
use pumpkin_plugin_api::command_wit::{Number, PermissionLevel};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::permission::{
    Permission, PermissionDefault, PermissionLevel as PermissionNodeLevel,
};
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};
use spiral::Shape;
use tracing::info;

// ─── Plugin struct (Plugin::on_load takes &self → use interior mutability) ──

pub struct SkinnyPlugin {
    state: Arc<Mutex<SkinnyState>>,
}

struct SkinnyState {
    config: SkinnyConfig,
    tracker: PreGenTracker,
}

impl Plugin for SkinnyPlugin {
    fn new() -> Self {
        let config = SkinnyConfig::default();
        let tracker = PreGenTracker::new(&config);
        Self {
            state: Arc::new(Mutex::new(SkinnyState { config, tracker })),
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            // Pumpkin uses this value as the namespace for plugin-defined
            // permissions, so it must be a valid, stable identifier.
            name: "skinny".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Someone_with_a_brain".into()],
            description: "Native PumpkinMC world chunk pre-generator (Chunky equivalent). \
                          Created by Someone_with_a_brain with AI pair-programming assistance."
                .into(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&self, context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("============================================================");
        info!(
            " Skinny World Pre-Generator v{} Initialized",
            env!("CARGO_PKG_VERSION")
        );
        info!(" Author: Someone_with_a_brain (AI-assisted)");
        info!(" Repository: https://github.com/Someone-with-a-brain/Skinny");
        info!(" Target Engine: PumpkinMC Native WASM API");
        {
            let s = self.state.lock().unwrap();
            info!(
                " Default Bounds: Center ({}, {}) | Radius: {} blocks | Shape: {:?}",
                s.config.center_x, s.config.center_z, s.config.radius_blocks, s.config.shape
            );
        }
        info!(" Use /skinny [center|radius|shape|start|pause|continue|cancel|progress|help]");
        info!("============================================================");

        // ── Build the /skinny command tree with full autofill nodes ──────
        //
        // Command::then() consumes by value, so we chain it builder-style.

        let handler = SkinnyCommandHandler {
            state: Arc::clone(&self.state),
        };

        // Command registration checks this permission before invoking a handler.
        // Declare it explicitly and grant it to level-two operators by default;
        // otherwise Pumpkin can complete `/skinny` but reject it as unknown.
        context.register_permission(&Permission {
            node: "skinny.admin".into(),
            description: "Allows use of Skinny world pre-generation commands.".into(),
            default: PermissionDefault::Op(PermissionNodeLevel::Two),
            children: vec![],
        })?;

        let cmd = Command::new(
            &["skinny".to_string()],
            "Skinny World Pre-Generator administration commands (operators only)",
        )
        // Sub-literal nodes — these appear as tab-complete suggestions
        .then(
            CommandNode::literal("center")
                .then(
                    CommandNode::argument("x", &ArgumentType::Integer((None, None))).then(
                        CommandNode::argument("z", &ArgumentType::Integer((None, None)))
                            .execute(handler.clone()),
                    ),
                )
                .execute(handler.clone()),
        )
        .then(
            CommandNode::literal("radius")
                .then(
                    CommandNode::argument("blocks", &ArgumentType::Integer((Some(1), None)))
                        .execute(handler.clone()),
                )
                .execute(handler.clone()),
        )
        .then(
            CommandNode::literal("shape")
                .then(CommandNode::literal("square").execute(handler.clone()))
                .then(CommandNode::literal("circle").execute(handler.clone()))
                .execute(handler.clone()),
        )
        .then(CommandNode::literal("start").execute(handler.clone()))
        .then(CommandNode::literal("pause").execute(handler.clone()))
        .then(CommandNode::literal("continue").execute(handler.clone()))
        .then(CommandNode::literal("resume").execute(handler.clone()))
        .then(CommandNode::literal("cancel").execute(handler.clone()))
        .then(CommandNode::literal("stop").execute(handler.clone()))
        .then(CommandNode::literal("progress").execute(handler.clone()))
        .then(CommandNode::literal("status").execute(handler.clone()))
        .then(CommandNode::literal("help").execute(handler.clone()))
        // Attach execution handler (called when the command is run)
        .execute(handler);

        // Register with a permission node — PumpkinMC gates this to ops by default
        context.register_command(cmd, "skinny.admin");

        Ok(())
    }
}

// ─── Command execution handler ───────────────────────────────────────────────

#[derive(Clone)]
struct SkinnyCommandHandler {
    state: Arc<Mutex<SkinnyState>>,
}

impl CommandHandler for SkinnyCommandHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: pumpkin_plugin_api::Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        // Double-check operator level (≥ 2 = gamemaster/op) even though the
        // permission node already restricts access, for safety.
        if !sender.has_permission_level(PermissionLevel::Two) {
            sender.send_error(TextComponent::text(
                "§cYou must be an operator to use /skinny.",
            ));
            return Ok(0);
        }

        let reply = self.dispatch(&args);
        sender.send_message(TextComponent::text(&reply));
        Ok(0)
    }
}

// ─── Sub-command dispatch ────────────────────────────────────────────────────

impl SkinnyCommandHandler {
    fn dispatch(&self, args: &ConsumedArgs) -> String {
        // The dispatcher puts the matched literal into ConsumedArgs as
        // Arg::Simple(name).  We check each expected key in turn.
        let subcmd = [
            // A child literal is also accompanied by its parent literal in
            // ConsumedArgs, so child literals must be checked first.
            "square", "circle", "center", "radius", "shape", "start", "pause", "continue", "resume",
            "cancel", "stop", "progress", "status", "help",
        ]
        .iter()
        .find(|&&k| matches!(args.get_value(k), Arg::Simple(_)));

        match subcmd {
            Some(&"start") => self.start(),
            Some(&"pause") => self.pause(),
            Some(&"continue") | Some(&"resume") => self.resume(),
            Some(&"cancel") | Some(&"stop") => self.cancel(),
            Some(&"progress") | Some(&"status") => self.progress(),
            Some(&"help") => help_text(),
            Some(&"shape") => "§cUsage: /skinny shape <square|circle>".into(),
            Some(&"square") => self.set_shape(Shape::Square),
            Some(&"circle") => self.set_shape(Shape::Circle),
            Some(&"center") => {
                let x = extract_int(args, "x");
                let z = extract_int(args, "z");
                match (x, z) {
                    (Some(x), Some(z)) => self.set_center(x, z),
                    _ => "§cUsage: /skinny center <x> <z>".into(),
                }
            }
            Some(&"radius") => match extract_int(args, "blocks") {
                Some(r) => self.set_radius(r),
                None => "§cUsage: /skinny radius <blocks>".into(),
            },
            _ => help_text(),
        }
    }
}

fn set_config_guard(state: &SkinnyState) -> Option<String> {
    matches!(state.tracker.state, TaskState::Running | TaskState::Paused).then(|| {
        "§cCannot change settings while a task is running or paused. Cancel it first.".into()
    })
}

fn rebuild_tracker(state: &mut SkinnyState) {
    state.tracker = PreGenTracker::new(&state.config);
}

impl SkinnyCommandHandler {
    fn set_center(&self, x: i32, z: i32) -> String {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(message) = set_config_guard(&state) {
            return message;
        }
        state.config.center_x = x;
        state.config.center_z = z;
        rebuild_tracker(&mut state);
        format!("§aCenter updated to §fX: {x} Z: {z}§a.")
    }

    fn set_radius(&self, radius: i32) -> String {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(message) = set_config_guard(&state) {
            return message;
        }
        state.config.radius_blocks = radius;
        rebuild_tracker(&mut state);
        format!(
            "§aRadius set to §f{radius} blocks §7({} chunks)§a.",
            (radius + 15) / 16
        )
    }

    fn set_shape(&self, shape: Shape) -> String {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(message) = set_config_guard(&state) {
            return message;
        }
        state.config.shape = shape;
        rebuild_tracker(&mut state);
        let name = if shape == Shape::Square {
            "Square"
        } else {
            "Circle"
        };
        format!("§aShape set to §f{name}§a.")
    }

    fn start(&self) -> String {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        match state.tracker.state {
            TaskState::Running => "§eSkinny pre-generation is already running.".into(),
            TaskState::Paused => "§eSkinny is paused; use /skinny continue to resume it.".into(),
            TaskState::Idle | TaskState::Completed => {
                rebuild_tracker(&mut state);
                state.tracker.start();
                format!(
                    "§aStarted Skinny pre-generation task for §f{}§a chunks.",
                    state.tracker.spiral.total_chunks()
                )
            }
        }
    }

    fn pause(&self) -> String {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.tracker.state != TaskState::Running {
            return "§eThere is no running Skinny task to pause.".into();
        }
        state.tracker.pause();
        "§ePaused Skinny pre-generation task.".into()
    }

    fn resume(&self) -> String {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.tracker.state != TaskState::Paused {
            return "§eThere is no paused Skinny task to resume.".into();
        }
        state.tracker.resume();
        "§aResumed Skinny pre-generation task.".into()
    }

    fn cancel(&self) -> String {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if !matches!(state.tracker.state, TaskState::Running | TaskState::Paused) {
            return "§eThere is no active Skinny task to cancel.".into();
        }
        state.tracker.cancel();
        "§cCancelled Skinny pre-generation task and reset its progress.".into()
    }

    fn progress(&self) -> String {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let tracker = &state.tracker;
        let status = match tracker.state {
            TaskState::Idle => "IDLE",
            TaskState::Running => "RUNNING",
            TaskState::Paused => "PAUSED",
            TaskState::Completed => "COMPLETED",
        };
        format!(
            "§6Status: §f[{status}] §7| Progress: {:.1}% ({}/{}) | Rate: {:.1} CPS | ETA: {}s",
            tracker.spiral.progress_percentage(),
            tracker.spiral.current_index(),
            tracker.spiral.total_chunks(),
            tracker.current_cps,
            tracker.eta_seconds()
        )
    }
}

/// Pull an integer value from a consumed argument node.
fn extract_int(args: &ConsumedArgs, key: &str) -> Option<i32> {
    match args.get_value(key) {
        Arg::Num(Ok(Number::Int32(v))) => Some(v),
        Arg::Num(Ok(Number::Int64(v))) => Some(v as i32),
        Arg::Num(Ok(Number::Float32(v))) => Some(v as i32),
        Arg::Num(Ok(Number::Float64(v))) => Some(v as i32),
        _ => None,
    }
}

fn help_text() -> String {
    "§6Skinny §eWorld Pre-Generator §r(operators only):\n\
     §b/skinny center <x> <z> §7– Set the generation center\n\
     §b/skinny radius <blocks> §7– Set the radius in blocks\n\
     §b/skinny shape <square|circle> §7– Choose the shape\n\
     §b/skinny start §7– Start pre-generation\n\
     §b/skinny pause §7– Pause pre-generation\n\
     §b/skinny continue §7– Resume pre-generation\n\
     §b/skinny cancel §7– Cancel pre-generation\n\
     §b/skinny progress §7– Show progress & ETA"
        .into()
}

pumpkin_plugin_api::register_plugin!(SkinnyPlugin);
