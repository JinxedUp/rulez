use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    fs,
    path::Path,
};

use pumpkin::{
    command::{
        args::Arg,
        dispatcher::CommandError,
        tree::CommandTree,
        CommandExecutor,
        CommandSender,
    },
    plugin::Context,
    server::Server,
};

use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault},
    text::TextComponent,
};

const COMMAND_NAMES: [&str; 1] = ["rules"];
const DESCRIPTION: &str = "Shows the server rules.";
const PERMISSION_NODE: &str = "rulez:rules";


// ---------------- COMMAND EXECUTOR ----------------

struct RulesExecutor {
    rules_path: String,
}

impl CommandExecutor for RulesExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a HashMap<&'a str, Arg<'a>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), CommandError>> + Send + 'a>> {
        let path = self.rules_path.clone();

        Box::pin(async move {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| "§cRules file could not be read.".to_string());

            sender
                .send_message(TextComponent::text(content))
                .await;

            Ok(())
        })
    }
}


// ---------------- PLUGIN LIFECYCLE ----------------

#[plugin_method]
async fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    server.init_log();
    log::info!("Rulez plugin loaded");

    // Convert PathBuf → String (IMPORTANT FIX)
    let data_folder_path = server.get_data_folder();
    let data_folder = data_folder_path.to_string_lossy().to_string();

    fs::create_dir_all(&data_folder).ok();

    let rules_path = format!("{}/rules.txt", data_folder);

    // Create default rules.txt if missing
    if !Path::new(&rules_path).exists() {
        let default_rules = "\
§6§lServer Rules§r
§8» §7Be respectful to everyone
§8» §7No cheating or exploiting bugs
§8» §7No spamming or advertising
§8» §7Listen to §cstaff §7at all times
§8» §7Have fun and use common sense §c❤
";
        fs::write(&rules_path, default_rules).ok();
    }

    // Register permission (everyone)
    let permission = Permission::new(
        PERMISSION_NODE,
        "Allows players to view server rules",
        PermissionDefault::Op(PermissionLvl::Zero),
    );
    server.register_permission(permission).await?;

    // Register /rules command
    let command = CommandTree::new(COMMAND_NAMES, DESCRIPTION)
        .execute(RulesExecutor { rules_path });

    server.register_command(command, PERMISSION_NODE).await;

    Ok(())
}


// ---------------- PLUGIN BASE ----------------

#[plugin_impl]
pub struct RulezPlugin;

impl RulezPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RulezPlugin {
    fn default() -> Self {
        Self::new()
    }
}
