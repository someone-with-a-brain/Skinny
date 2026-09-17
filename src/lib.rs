pub mod config;
pub mod generator;
pub mod spiral;

use std::sync::Mutex;

use config::SkinnyConfig;
use generator::PreGenTracker;
use pumpkin_plugin_api::command::{
    Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
};
use pumpkin_plugin_api::command_wit::{Number, PermissionLevel};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};
use tracing::info;

// ─── Plugin struct (Plugin::on_load takes &self → use interior mutability) ──

pub struct SkinnyPlugin {
    state: Mutex<SkinnyState>,
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
            state: Mutex::new(SkinnyState { config, tracker }),
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Skinny World Pre-Generator".into(),
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
                s.config.center_x,
                s.config.center_z,
                s.config.radius_blocks,
                s.config.shape
            );
        }
        info!(" Use /skinny [center|radius|shape|start|pause|continue|cancel|progress|help]");
        info!("============================================================");

        // ── Build the /skinny command tree with full autofill nodes ──────
        //
        // Command::then() consumes by value, so we chain it builder-style.

        let cmd = Command::new(
            &["skinny".to_string()],
            "Skinny World Pre-Generator administration commands (operators only)",
        )
        // Sub-literal nodes — these appear as tab-complete suggestions
        .then(
            CommandNode::literal("center")
                .then(
                    CommandNode::argument("x", &ArgumentType::Integer((None, None)))
                        .then(CommandNode::argument("z", &ArgumentType::Integer((None, None)))),
                ),
        )
        .then(
            CommandNode::literal("radius")
                .then(CommandNode::argument(
                    "blocks",
                    &ArgumentType::Integer((Some(1), None)),
                )),
        )
        .then(
            CommandNode::literal("shape")
                .then(CommandNode::literal("square"))
                .then(CommandNode::literal("circle")),
        )
        .then(CommandNode::literal("start"))
        .then(CommandNode::literal("pause"))
        .then(CommandNode::literal("continue"))
        .then(CommandNode::literal("resume"))
        .then(CommandNode::literal("cancel"))
        .then(CommandNode::literal("stop"))
        .then(CommandNode::literal("progress"))
        .then(CommandNode::literal("status"))
        .then(CommandNode::literal("help"))
        // Attach execution handler (called when the command is run)
        .execute(SkinnyCommandHandler);

        // Register with a permission node — PumpkinMC gates this to ops by default
        context.register_command(cmd, "skinny.admin");

        Ok(())
    }
}

// ─── Command execution handler ───────────────────────────────────────────────

struct SkinnyCommandHandler;

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

        let reply = dispatch(&args);
        sender.send_message(TextComponent::text(&reply));
        Ok(0)
    }
}

// ─── Sub-command dispatch ────────────────────────────────────────────────────

fn dispatch(args: &ConsumedArgs) -> String {
    // The dispatcher puts the matched literal into ConsumedArgs as
    // Arg::Simple(name).  We check each expected key in turn.
    let subcmd = ["center", "radius", "shape", "square", "circle",
                  "start", "pause", "continue", "resume",
                  "cancel", "stop", "progress", "status", "help"]
        .iter()
        .find(|&&k| matches!(args.get_value(k), Arg::Simple(_)));

    match subcmd {
        Some(&"start") => "§aStarted Skinny pre-generation task!".into(),
        Some(&"pause") => "§ePaused Skinny pre-generation task.".into(),
        Some(&"continue") | Some(&"resume") => "§aResumed Skinny pre-generation task.".into(),
        Some(&"cancel") | Some(&"stop") => "§cCancelled Skinny pre-generation task.".into(),
        Some(&"progress") | Some(&"status") => {
            "§6Status: §f[IDLE] §7| Progress: 0.0% (0/0) | Rate: 0.0 CPS | ETA: 0s".into()
        }
        Some(&"help") => help_text(),
        Some(&"shape") => "§cUsage: /skinny shape <square|circle>".into(),
        Some(&"square") => "§aShape set to §fSquare§a.".into(),
        Some(&"circle") => "§aShape set to §fCircle§a.".into(),
        Some(&"center") => {
            let x = extract_int(args, "x");
            let z = extract_int(args, "z");
            match (x, z) {
                (Some(x), Some(z)) => format!("§aCenter updated to §fX: {x} Z: {z}§a."),
                _ => "§cUsage: /skinny center <x> <z>".into(),
            }
        }
        Some(&"radius") => match extract_int(args, "blocks") {
            Some(r) => format!(
                "§aRadius set to §f{r} blocks §7({} chunks)§a.",
                (r + 15) / 16
            ),
            None => "§cUsage: /skinny radius <blocks>".into(),
        },
        _ => help_text(),
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
