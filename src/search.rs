use crate::env::PRIMARY_LUNCH_CHANNEL;
use crate::flikisdining;
use chrono::{DateTime, Utc};
use serenity::{builder::CreateEmbed, model::prelude::Message, prelude::Context};
use tantivy::{
    doc,
    query::QueryParser,
    schema::{Field, Schema},
    Index,
};
use tokio::task::JoinSet;

// static TANTIVY_SCHEMA: Schema = {
//     let mut schema = tantivy::schema::Schema::builder();

//     // add an indexed text field
//     schema.add_text_field("content", tantivy::schema::TEXT | tantivy::schema::STORED);
//     // and an unindexed date field
//     schema.add_date_field("date", tantivy::schema::STORED);

//     schema.build()
// };

fn create_index() -> (tantivy::Index, Schema, Field, Field) {
    // create the schema
    let mut schema = tantivy::schema::Schema::builder();
    let content = schema.add_text_field("content", tantivy::schema::TEXT | tantivy::schema::STORED);
    let date = schema.add_date_field("date", tantivy::schema::STORED);
    let schema = schema.build();

    // create the index
    let index = Index::create_in_ram(schema.clone());

    return (index, schema, content, date);
}

pub async fn handle(context: Context, msg: Message) {
    // ignore bots
    if msg.author.bot {
        return;
    }

    // save start time so we can calculate processing time
    let start = Utc::now();

    // get the message content so we can match on it
    let content = msg.content.to_lowercase();

    // check if starts with "when will we have"
    if msg.channel_id.to_string() == *PRIMARY_LUNCH_CHANNEL
        && content.starts_with("when will we have")
    {
        // find the content after "when will we have"
        let search_term = content
            .split("when will we have")
            .collect::<Vec<&str>>()
            .get(1)
            .unwrap_or(&"")
            .trim();

        println!("Searching for lunch: {}", search_term);

        // if nothing, return
        if search_term.is_empty() {
            return;
        }

        // search the next 3 weeks
        let (index, _schema, content, date) = create_index();
        let mut index_writer = index.writer(3_000_000).unwrap();

        // fetch 3 weeks of lunch
        let mut set = JoinSet::new();

        for n in 0..3 {
            let date = DateTime::<Utc>::from(Utc::now() + chrono::Duration::weeks(n));
            set.spawn(flikisdining::fetch_week_lunch(date));
        }

        while let Some(res) = set.join_next().await {
            if let Err(why) = res {
                println!("Error fetching lunch: {:?}", why);

                // attempt to send in channel
                let _ = msg
                        .channel_id
                        .say(&context.http, format!("[warn] failed to fetch a week of lunch, result may be missing entries: {:?}", why)).await;

                continue;
            }

            res.unwrap().into_iter().for_each(|week| {
                    week.into_iter().for_each(|day| {
                        let date_value = day.date;

                        day.menu_items.into_iter().for_each(|food| {
                            if let Err(why) = index_writer.add_document(doc!(
                                content => food.food.unwrap().name,
                                date => date_value.clone()
                            )) {
                                println!("Error adding document: {:?}", why);

                                // attempt to send in channel
                                let _ = msg
                                    .channel_id
                                    .say(&context.http, format!("[warn] failed to add document to index, result may be missing entries: {:?}", why));
                            }
                        })
                    })
                })
        }

        // commit the index so we can search it
        if let Err(why) = index_writer.commit() {
            println!("Error committing index: {:?}", why);

            // attempt to send in channel
            let _ = msg
                .channel_id
                .say(
                    &context.http,
                    format!(
                        "[warn] failed to commit index, result may be missing entries: {:?}",
                        why
                    ),
                )
                .await;
        }

        let committed_time = Utc::now();
        println!(
            "Committed index ({:?} ms)",
            (Utc::now() - start).num_milliseconds()
        );

        // search the index
        let reader = index.reader().unwrap();
        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(&index, vec![content]);

        let query = query_parser.parse_query(search_term).unwrap();
        let top_docs = searcher
            .search(&query, &tantivy::collector::TopDocs::with_limit(10))
            .unwrap_or(vec![]);

        println!(
            "Searched index ({:?} ms)",
            (Utc::now() - committed_time).num_milliseconds()
        );

        // format the top documents
        let description = top_docs
            .into_iter()
            .enumerate()
            .map(|(idx, (score, doc_address))| {
                // get the document
                let retrieved_doc = searcher.doc(doc_address);

                if retrieved_doc.is_err() {
                    return format!("{}: Error retrieving document", idx);
                }

                let retrieved_doc = retrieved_doc.unwrap();

                // get the content
                let content = retrieved_doc.get_first(content).unwrap().as_text().unwrap();
                let date = retrieved_doc.get_first(date).unwrap().as_text().unwrap();

                // parse the date (yyyy-mm-dd) and set to midday EST
                let date = DateTime::parse_from_rfc3339(&(date.to_owned() + "T12:00:00-05:00"))
                    .unwrap()
                    .with_timezone(&Utc);

                format!(
                    "{}) **{}**\n> <t:{}:F>\n> Score: {}",
                    idx,
                    content,
                    (date.timestamp_millis() / 1000),
                    score
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        // now send the embed
        let mut embed = CreateEmbed::default();
        embed.title("🔍 Search Results");
        embed.description(description);
        embed.color(0x00FF00);
        embed.footer(|f| f.text((Utc::now() - start).num_milliseconds().to_string() + " ms"));

        let _ = msg
            .channel_id
            .send_message(&context.http, |m| m.set_embed(embed))
            .await;
    }
}
