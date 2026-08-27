use clap::{Parser, Subcommand};

/// hoi is a command-line tool to help create simple command-line powered utilities.
#[derive(Parser, Debug)]
#[command(
    name = "hoi",
    version,
    about,
    allow_external_subcommands = true,
    disable_help_subcommand = true
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<CommandArg>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CommandArg {
    /// List commands from the discovered configuration
    List,
    /// Create a new .hoi.yml (or global) configuration file
    Init {
        /// Write ~/.hoi/.hoi.global.yml instead of ./.hoi.yml
        #[arg(long)]
        global: bool,
        /// Replace an existing configuration file
        #[arg(long)]
        force: bool,
    },
    /// Inspect discovered configuration files
    Config {
        /// Print local and global config paths
        #[arg(long)]
        path: bool,
        /// Load, merge, and validate configuration
        #[arg(long)]
        check: bool,
    },
    /// Same as `hoi config --check`
    Validate,
    #[command(external_subcommand)]
    External(Vec<String>),
}
