use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::ipc::client;

#[derive(Parser)]
#[command(name = "zlaunch")]
#[command(about = "A fast application launcher for Linux")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show the launcher window
    Show,
    /// Hide the launcher window
    Hide,
    /// Toggle the launcher window visibility
    Toggle,
    /// Quit the daemon
    Quit,
    /// Reload the daemon (fully restart the process)
    Reload,
    /// Theme management
    Theme {
        #[command(subcommand)]
        action: Option<ThemeCommands>,
    },
}

#[derive(Subcommand)]
pub enum ThemeCommands {
    /// List available themes
    List,
    /// Set the active theme
    Set {
        /// Name of the theme to set
        name: String,
    },
}

/// Handle a client command by sending it to the running daemon.
pub fn handle_client_command(cmd: Commands) -> Result<()> {
    if !client::is_daemon_running() {
        anyhow::bail!("zlaunch daemon is not running. Start it first by running: zlaunch");
    }

    match cmd {
        Commands::Show => {
            client::show()?;
        }
        Commands::Hide => {
            client::hide()?;
        }
        Commands::Toggle => {
            client::toggle()?;
        }
        Commands::Quit => {
            client::quit()?;
        }
        Commands::Reload => {
            client::reload()?;
            println!("Daemon is reloading...");
        }
        Commands::Theme { action } => match action {
            None => {
                // No subcommand - show current theme
                let theme = client::get_current_theme()?;
                println!("Current theme: {}", theme);
            }
            Some(ThemeCommands::List) => {
                let themes = client::list_themes()?;
                println!("Available themes:");
                for theme in themes {
                    let source = if theme.is_bundled {
                        "(bundled)"
                    } else {
                        "(user)"
                    };
                    println!("  {} {}", theme.name, source);
                }
            }
            Some(ThemeCommands::Set { name }) => {
                client::set_theme(&name)?;
                println!("Theme set to '{}'", name);
            }
        },
    }

    Ok(())
}
