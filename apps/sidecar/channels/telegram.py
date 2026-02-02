"""
Telegram Bot Channel
====================
Telegram integration for the AI agent.
Same agent backend, different delivery channel.

Setup:
1. Create a bot via @BotFather on Telegram
2. Set TELEGRAM_BOT_TOKEN environment variable
3. Run: python -m channels.telegram
"""

import os
import logging
from typing import Optional

from telegram import Update
from telegram.ext import (
    Application,
    CommandHandler,
    MessageHandler,
    ContextTypes,
    filters,
)

from agent.chat import ChatService
from agent.providers import OllamaProvider, AnthropicProvider, OpenAIProvider


# Configure logging
logging.basicConfig(
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    level=logging.INFO,
)
logger = logging.getLogger(__name__)


class TelegramBot:
    """
    Telegram bot that connects to the AI agent backend.
    
    Each Telegram chat gets its own conversation ID, maintaining
    separate conversation histories per user.
    """
    
    def __init__(
        self,
        token: Optional[str] = None,
        chat_service: Optional[ChatService] = None,
    ):
        self.token = token or os.getenv("TELEGRAM_BOT_TOKEN")
        if not self.token:
            raise ValueError("TELEGRAM_BOT_TOKEN not set")
        
        # Use provided service or create new one
        self.chat_service = chat_service or self._create_chat_service()
        self.default_provider = os.getenv("TELEGRAM_PROVIDER", "ollama")
        self.default_model = os.getenv("TELEGRAM_MODEL")
    
    def _create_chat_service(self) -> ChatService:
        """Create and configure chat service"""
        service = ChatService()
        
        # Configure Ollama
        ollama_host = os.getenv("OLLAMA_HOST", "http://localhost:11434")
        service.register_provider(OllamaProvider(base_url=ollama_host))
        
        # Configure cloud providers if available
        if os.getenv("ANTHROPIC_API_KEY"):
            service.register_provider(AnthropicProvider())
        if os.getenv("OPENAI_API_KEY"):
            service.register_provider(OpenAIProvider())
        
        return service
    
    async def start_command(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle /start command"""
        await update.message.reply_text(
            "👋 Hello! I'm your AI assistant.\n\n"
            "Commands:\n"
            "/start - Show this message\n"
            "/clear - Clear conversation history\n"
            "/provider <name> - Switch provider (ollama, anthropic, openai)\n"
            "/model <name> - Set model\n"
            "/status - Show current settings\n\n"
            "Just send me a message to chat!"
        )
    
    async def clear_command(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle /clear command"""
        chat_id = str(update.effective_chat.id)
        self.chat_service.clear_conversation(chat_id)
        await update.message.reply_text("🧹 Conversation cleared!")
    
    async def provider_command(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle /provider command"""
        chat_id = str(update.effective_chat.id)
        
        if not context.args:
            providers = self.chat_service.list_providers()
            provider_list = "\n".join([f"• {p['name']}" for p in providers])
            await update.message.reply_text(
                f"Available providers:\n{provider_list}\n\n"
                "Usage: /provider <name>"
            )
            return
        
        provider_name = context.args[0].lower()
        try:
            self.chat_service.get_provider(provider_name)
            conv = self.chat_service.get_or_create_conversation(chat_id)
            conv.provider_name = provider_name
            await update.message.reply_text(f"✓ Switched to {provider_name}")
        except ValueError:
            await update.message.reply_text(f"❌ Provider '{provider_name}' not available")
    
    async def model_command(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle /model command"""
        chat_id = str(update.effective_chat.id)
        
        if not context.args:
            conv = self.chat_service.get_or_create_conversation(chat_id)
            provider = self.chat_service.get_provider(conv.provider_name)
            models = provider.list_models()
            model_list = "\n".join([f"• {m}" for m in models])
            await update.message.reply_text(
                f"Available models for {conv.provider_name}:\n{model_list}\n\n"
                "Usage: /model <name>"
            )
            return
        
        model_name = context.args[0]
        conv = self.chat_service.get_or_create_conversation(chat_id)
        conv.model = model_name
        await update.message.reply_text(f"✓ Model set to {model_name}")
    
    async def status_command(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle /status command"""
        chat_id = str(update.effective_chat.id)
        conv = self.chat_service.get_or_create_conversation(chat_id)
        health = await self.chat_service.health_check()
        
        health_status = "\n".join([
            f"• {name}: {'✓' if ok else '✗'}" 
            for name, ok in health.items()
        ])
        
        await update.message.reply_text(
            f"📊 Status\n\n"
            f"Provider: {conv.provider_name}\n"
            f"Model: {conv.model or 'default'}\n"
            f"Messages: {len(conv.messages)}\n\n"
            f"Provider Health:\n{health_status}"
        )
    
    async def handle_message(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle regular text messages"""
        chat_id = str(update.effective_chat.id)
        user_message = update.message.text
        
        # Show typing indicator
        await context.bot.send_chat_action(
            chat_id=update.effective_chat.id,
            action="typing",
        )
        
        try:
            response = await self.chat_service.chat(
                conversation_id=chat_id,
                user_message=user_message,
                model=self.default_model,
            )
            
            # Telegram has 4096 char limit, split if needed
            content = response.content
            if len(content) > 4000:
                for i in range(0, len(content), 4000):
                    await update.message.reply_text(content[i:i+4000])
            else:
                await update.message.reply_text(content)
                
        except Exception as e:
            logger.error(f"Chat error: {e}")
            await update.message.reply_text(
                f"❌ Error: {str(e)}\n\nTry /clear to reset or /status to check providers."
            )
    
    async def error_handler(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle errors"""
        logger.error(f"Update {update} caused error {context.error}")
    
    def run(self):
        """Start the Telegram bot"""
        logger.info("Starting Telegram bot...")
        
        app = Application.builder().token(self.token).build()
        
        # Register handlers
        app.add_handler(CommandHandler("start", self.start_command))
        app.add_handler(CommandHandler("clear", self.clear_command))
        app.add_handler(CommandHandler("provider", self.provider_command))
        app.add_handler(CommandHandler("model", self.model_command))
        app.add_handler(CommandHandler("status", self.status_command))
        app.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, self.handle_message))
        app.add_error_handler(self.error_handler)
        
        # Start polling
        app.run_polling(allowed_updates=Update.ALL_TYPES)


def main():
    """Entry point for Telegram bot"""
    bot = TelegramBot()
    bot.run()


if __name__ == "__main__":
    main()
