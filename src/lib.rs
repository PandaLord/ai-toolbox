mod gpt;
mod utils;
use clap::{Args, Parser, Subcommand};
pub use gpt::*;

#[derive(Parser)]
#[command(name = "ai")]
#[command(author, version="1.0.0", about, long_about = None)]
struct Cli {
    // /// Optional name to operate on
    // name: Option<String>,

    // /// Sets a custom config file
    // #[arg(short, long, value_name = "FILE")]
    // config: Option<PathBuf>,

    // /// Turn debugging information on
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    //  user relevant commands
    // #[command(arg_required_else_help = true)]
    // User(User),
    #[command(arg_required_else_help = true)]
    GPT(GPT),
    // #[command(arg_required_else_help = true)]
    // Timer(Timer),
}
// #[derive(Debug, Args)]
// #[command(args_conflicts_with_subcommands = true)]
// struct User {
//     #[command(subcommand)]
//     command: Option<UserCommands>,
// }
// #[derive(Debug, Subcommand)]
// enum UserCommands {
//     Add { username: String },
//     Delete { userid: String },
//     Query { username: String },
// }
#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct GPT {
    #[command(subcommand)]
    command: Option<GPTCommands>,
}

#[derive(Debug, Subcommand)]
enum GPTCommands {
    CreateChat,
    CreateEdit,
    CreateImage,
    CreateImageEdit,
    CreateImageVariation,
    CreateEmbedding,
    CreateTranscription,
    CreateTranslation,
}