#![feature(once_cell_try)]
#![feature(error_iter)]

mod compile;
mod run;

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt::Display;

#[derive(Parser, Clone, Debug)]
struct CliArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    #[command(name = "compile")]
    Compile(CompileArgs),
    #[command(name = "run")]
    Run(RunArgs),
}

#[derive(Args, Clone, Debug)]
struct CompileArgs {
    #[arg(long)]
    #[arg(default_value_t = match std::env::var("PURELANG_HOME") {
        Ok(val) => format!("{val}/Native/CompileService"),
        Err(_) => "CompileService".to_owned(),
    })]
    core: String,
    #[arg(long = "cfg-path")]
    config_path: Option<String>,
    #[arg(long)]
    compilers: Vec<String>,
    #[arg(long, short = 's')]
    sources: Vec<String>,
}

#[derive(Args, Clone, Debug)]
struct RunArgs {
    #[arg(long)]
    #[arg(default_value_t = match std::env::var("PURELANG_HOME") {
        Ok(val) => format!("{val}/Native/Runtime"),
        Err(_) => "Runtime".to_owned(),
    })]
    core: String,
    #[arg(long = "cfg-path")]
    config_path: Option<String>,
    #[arg(long = "cfg-type", default_value_t = ConfigType::Json)]
    config_type: ConfigType,
    #[arg(long = "assembly")]
    main_assembly_name: String,
    #[arg(long = "class")]
    main_class_name: String,
    #[arg(long)]
    assemblies: Vec<String>,
    #[arg(trailing_var_arg = true)]
    arguments: Vec<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ConfigType {
    Json,
}

impl ConfigType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
        }
    }
}

impl Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(thiserror::Error, Debug)]
pub struct Utf8Error;

impl Display for Utf8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

fn main() -> global::Result<()> {
    let args = CliArgs::try_parse()?;
    match args.command {
        Command::Compile(args) => compile::handle(args),
        Command::Run(args) => run::handle(args),
    }
}
