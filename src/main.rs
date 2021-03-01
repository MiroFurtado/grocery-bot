use std::env;

use futures::StreamExt;
use telegram_bot::*;

struct GroceryList {
    entries: Vec<String>
}

impl GroceryList {
    fn new() -> GroceryList {
        GroceryList { entries: vec![] }
    }

}

async fn execution_loop(mut list: GroceryList, api: &telegram_bot::Api) -> Result<(), Error> {
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                if data == "/view" {
                    for element in &list.entries {
                        api.send(message.text_reply(format!(
                            "* {}",
                            &element
                        )))
                        .await?; 
                    }
                }
                println!("<{}>: {}", &message.from.first_name, data);
                list.entries.push(data.clone());
                // Answer message with "mHi".
                api.send(message.text_reply(format!(
                    "Hi, {}! You just wrote '{}'",
                    &message.from.first_name, data
                )))
                .await?;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // setup API and GroceryList data structure
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);
    let list = GroceryList::new();

    execution_loop(list, &api).await?; // enter main execution loop
    Ok(())
}
