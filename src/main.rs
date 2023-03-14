use std::env;

use clap::{Subcommand, Parser};
use anyhow::Result;
use dotenv::dotenv;


use ai_toolbox::{api::{self, Api}, token::Token, datamap::{Message, ChatPayload, Model}};
#[derive(Parser)]
#[command(name = "ai")]
#[command(author, version="1.0.0", about, long_about = None)]
struct Cli {

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    //  user relevant commands
    // #[command(arg_required_else_help = true)]
    // User(User),
    #[command(arg_required_else_help = true)]
    Chat { content: Option<String> },
    // #[command(arg_required_else_help = true)]
    // Timer(Timer),
}


#[tokio::main]
async fn main() -> Result<()>{

    let cli  = Cli::parse();
    dotenv().ok();
    match cli.command {
        Commands::Chat { content } => {
            let api_secret = env::var("GPT_TOKEN").unwrap_or("no GPT token".to_string());
            let token = Token::new(api_secret);
            let request = Api::new(token);
            let content = content.unwrap_or("no input".to_string());
            // chat payload
            let msg: Message = Message {
                role: "user".to_string(),
                content,
            };
            let chat_payload: ChatPayload = ChatPayload {
                model: Model::Gpt35Turbo,
                messages: vec![msg],
                ..Default::default()
            };
            let res = request.chat(chat_payload).await?;
            println!("{:?}", res);

        }
    }
        
    Ok(())
}
