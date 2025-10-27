use crate::faucet::FaucetService;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use tracing::{error, info};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Faucet commands:")]
enum Command {
    #[command(description = "Start the bot and request tokens")]
    Start,
}

pub async fn run_bot(token: String, faucet_service: Arc<FaucetService>) {
    info!("Starting Telegram bot...");

    let bot = Bot::new(token);

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(dptree::endpoint(message_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![faucet_service])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    _faucet_service: Arc<FaucetService>,
) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "Welcome to the faucet! 🚰\n\nPlease send me your Ethereum address (0x...) to receive tokens.",
            )
            .await?;
        }
    }
    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    faucet_service: Arc<FaucetService>,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        let tg_user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or_default();
        let address = text.trim();

        // Basic validation that it looks like an address
        if !address.starts_with("0x") || address.len() != 42 {
            bot.send_message(
                msg.chat.id,
                "❌ Invalid address format. Please send a valid Ethereum address starting with 0x.",
            )
            .await?;
            return Ok(());
        }

        // Enforce per-user single claim
        if faucet_service
            .store()
            .has_telegram_user(tg_user_id)
            .unwrap_or(false)
        {
            bot.send_message(
                msg.chat.id,
                "❌ You've already received tokens from this faucet.",
            )
            .await?;
            return Ok(());
        }

        bot.send_message(msg.chat.id, "Processing your request... ⏳")
            .await?;

        match faucet_service.send_native(address).await {
            Ok(tx_hash) => {
                // Mark telegram user as served
                let _ = faucet_service.store().mark_telegram_user(tg_user_id);
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "✅ Tokens sent successfully\\!\n\nTransaction hash:\n`{}`",
                        tx_hash
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
            }
            Err(e) => {
                let error_msg = e.to_string();
                let response = if error_msg.contains("already_sent") {
                    "❌ This address has already received tokens from the faucet.".to_string()
                } else if error_msg.contains("Invalid") {
                    format!("❌ Invalid address: {}", error_msg)
                } else {
                    error!("Telegram faucet error: {}", e);
                    "❌ Failed to send tokens. Please try again later.".to_string()
                };
                bot.send_message(msg.chat.id, response).await?;
            }
        }
    }
    Ok(())
}
