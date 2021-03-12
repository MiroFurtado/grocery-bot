#![feature(proc_macro_hygiene, decl_macro)]
use std::env;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;

use futures::StreamExt;
use telegram_bot::*;

trait IngredientFilter {
    fn look_up_item(&self, entry: &String) -> Option<usize>;

    // fn look_up_item_name(&self, entry: &String) -> Option<String>;
}

struct RulesFilter {
    boxes: Vec<HashSet<String>>
}

impl RulesFilter {
    fn build_filter() -> RulesFilter {
        let s = include_str!("filters.txt");
        let mut boxes = Vec::new();
        for rule_line in s.split("\n") {
            let mut filter_list = Vec::new();
            for element in rule_line.split(" ") {
                filter_list.push(String::from(element));
            }
            boxes.push(HashSet::<String>::from_iter(filter_list));
        }
        return RulesFilter { boxes: boxes }
    }
}

impl IngredientFilter for RulesFilter {
    fn look_up_item(&self, entry: &String) -> Option<usize> {
        for word in entry.as_str().split(" ") {
            for (box_num, a_box) in self.boxes.iter().enumerate() {
                if a_box.contains(&String::from(word)) {
                    return Some(box_num)
                }
            }
        }
        None
    }
}

struct GroceryList {
    entries: Vec<Vec<String>>,
}

impl GroceryList {
    fn new() -> GroceryList {
        GroceryList { entries: vec![] }
    }

    fn consolidate(&mut self, strategy: &impl IngredientFilter) {
        let mut label = HashMap::new();
        let mut consolidated_list = Vec::new();

        for entry_line in self.entries.iter() {
            if entry_line.len() > 1 { continue; }
            let entry = &entry_line[0];
            // 3 cases: 1. Entry doesn't match ingredient filter, 
            //          2. Entry matches and hasn't been seen before
            //          3. Entry matches and *has* been seen before.
            if let Some(box_num) = strategy.look_up_item(entry) {
                match label.get(&box_num) {
                    None => { // not seen before
                        label.insert(box_num, consolidated_list.len());
                        consolidated_list.push(vec![entry.clone()]);
                    },
                    Some(idx) => { // seen before
                        consolidated_list[*idx].push(entry.clone());
                    }
                }
            } else { // doesn't match
                consolidated_list.push(vec![entry.clone()]);
            }
        }
        self.entries = consolidated_list;
    }

}

#[macro_use] extern crate rocket;
use rocket::http::RawStr;

#[get("/list/<list_id>")]
fn get_list(list_id: &RawStr) -> String{
    format!("Hello, {}!", list_id.as_str())
}



fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

async fn execution_loop(mut list: GroceryList, api: &telegram_bot::Api) -> Result<(), Error> {
    let filter_ingredients = RulesFilter::build_filter();

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                if data == "/view" {
                    list.consolidate(&filter_ingredients);
                    for element in &list.entries {
                        api.send(message.to_source_chat().text(format!(
                            "* {}",
                            &element.join(", ")
                        )))
                        .await?; 
                    }
                }
                // println!("<{}>: {}", &message.from.first_name, data);
                for item in data.split("\n") {
                    let mut item_str = String::from(item);
                    remove_whitespace(&mut item_str);
                    if item_str != "" {
                        list.entries.push(vec![String::from(item)]);
                    }
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    rocket::ignite().mount("/", routes![get_list]).launch();
    // setup API and GroceryList data structure
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);
    let list = GroceryList::new();

    execution_loop(list, &api).await?; // enter main execution loop
    Ok(())
}
