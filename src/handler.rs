use serenity::{
    async_trait,
    model::prelude::Message,
    prelude::{Context, EventHandler},
};

use crate::{env::PRIMARY_LUNCH_CHANNEL, lunch, search};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        // get the message content so we can match on it
        let content = msg.content.to_lowercase();

        // check for `what` and `lunch` (if in env `PRIMARY_LUNCH_CHANNEL`)
        // otherwise, check if it is "what lunch"
        if (msg.channel_id.to_string() == *PRIMARY_LUNCH_CHANNEL
            && content.contains("what")
            && content.contains("lunch"))
            || (content.contains("what lunch"))
        {
            lunch::handle(context, msg).await;
            return;
        }

        // check if starts with "when will we have"
        // check if starts with "when will we have"
        if msg.channel_id.to_string() == *PRIMARY_LUNCH_CHANNEL
            && content.starts_with("when will we have")
        {
            search::handle(context, msg).await;
        }
    }
}
