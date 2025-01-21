use crate::env::PRIMARY_LUNCH_CHANNEL;
use crate::flikisdining;
use chrono::{Datelike, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::{
    all::CreateEmbedFooter,
    builder::{CreateEmbed, CreateMessage},
    model::prelude::Message,
    prelude::Context,
};

static WEEKDAYS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"monday").unwrap(),
        Regex::new(r"tues(day)?").unwrap(),
        Regex::new(r"wed(nesday)?").unwrap(),
        Regex::new(r"thurs(day)?").unwrap(),
        Regex::new(r"fri(day)?").unwrap(),
    ]
});

pub async fn handle(context: Context, msg: Message) {
    // ignore bots
    if msg.author.bot {
        return;
    }

    // save start time so we can calculate processing time
    let start = Utc::now();

    // get the message content so we can match on it
    let content = msg.content.to_lowercase();

    // check for `what` and `lunch` (if in env `PRIMARY_LUNCH_CHANNEL`)
    // otherwise, check if it is "what lunch"
    if (msg.channel_id.to_string() == *PRIMARY_LUNCH_CHANNEL
        && content.contains("what")
        && content.contains("lunch"))
        || (content.contains("what lunch"))
    {
        // debug mode
        let debug = content.contains("whats in your head");

        // figure out date
        let mut date = Utc::now();

        // if a weekday is mentioned, create an offset from today to that weekday
        let mut days: i64 = 0;
        for (i, weekday) in WEEKDAYS.iter().enumerate() {
            if weekday.is_match(&content) {
                days = i64::try_from(i).unwrap();
                days -= i64::try_from(date.weekday().num_days_from_monday()).unwrap();
                break;
            }
        }

        // if the days matched is negative, that means it's already passed
        // offset by 7 days so we get the next week
        if days < 0 {
            days += 7;
        }

        // for each `tmr` or `tomorrow` in the content, add a day
        days += i64::try_from(content.matches("tmr").count() + content.matches("tomorrow").count())
            .unwrap();
        date = date + chrono::Duration::days(days);

        // debug log the amount of days added
        if debug {
            let _ = msg.channel_id.say(
                &context.http,
                format!(
                    "[debug] Days added: {} | Date: {}",
                    days,
                    date.format("%Y-%m-%d")
                ),
            );
        }

        // fetch lunch for that day
        let lunch = flikisdining::fetch_lunch(date).await;

        // if there was an error, send a message to the channel
        if let Err(why) = lunch {
            println!("Error fetching lunch: {:?}", why);

            // attempt to send in the channel
            if let Err(why) = msg
                .channel_id
                .say(
                    &context.http,
                    // "Failed to fetch lunch: ".to_owned() + &message,
                    format!("Failed to fetch lunch: {:?}", why),
                )
                .await
            {
                println!("Error sending message: {:?}", why);
            }

            return;
        }

        // get the lunch
        let lunch = lunch.unwrap();
        let mut thumbnail: Option<String> = None;

        // get the menu items
        let menu_items = lunch
            .into_iter()
            .map(|item| {
                let food = item.food.unwrap();

                // get the calories
                let cals = food
                    .rounded_nutrition_info
                    .unwrap_or_default()
                    .calories
                    .unwrap_or(-1.0);

                // check if this has a thumbnail
                if item.image_thumbnail.is_some() {
                    // if it does, set the thumbnail
                    thumbnail = item.image_thumbnail;
                }

                // return the formatted string
                format!(
                    "{} - `{}` cals",
                    food.name,
                    if cals == -1.0 {
                        "".to_owned()
                    } else {
                        cals.to_string()
                    }
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        // and try to send the message
        if let Err(why) = msg
            .channel_id
            .send_message(&context.http, {
                let mut embed = CreateEmbed::new()
                    .title(match days {
                        0 => "🍖 Today's Lunch".to_owned(),
                        1 => "🍖 Tomorrow's Lunch".to_owned(),
                        days => format!("🍖 Lunch in {} days", days),
                    })
                    .description(menu_items)
                    .footer(CreateEmbedFooter::new(
                        (Utc::now() - start).num_milliseconds().to_string() + " ms",
                    ))
                    .color(0xEE8B2F)
                    .timestamp(Utc::now());

                if thumbnail.is_some() {
                    embed = embed.thumbnail(thumbnail.unwrap());
                }

                CreateMessage::new().embed(embed)
            })
            .await
        {
            println!("Error sending message: {:?}", why);
        }
    }
}
