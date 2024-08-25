use std::{collections::HashSet, time::Duration};

use enigo::{Enigo, Keyboard};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardData {
    id: String,
    // id: CardId,
    rarity: String,
    ty: String,
    name: String,
    image_url: String,
    image_name: String,
    cost_life: usize,
    power: Option<usize>,
    counter: Option<usize>,
    color: String,
    effect: Option<String>,
    trigger: Option<String>,
    subtype: Vec<String>,
    attribute: Option<String>,
}

fn main() {
    let file = std::fs::read_to_string("../scraper/cache/card_db.jsonl").unwrap();
    let deserializer = serde_json::Deserializer::from_str(&file).into_iter::<CardData>();
    let mut rarities = HashSet::new();
    for item in deserializer {
        let item = item.unwrap();
        rarities.insert(item.attribute);
    }

    let mut rarities = rarities.into_iter().collect::<Vec<_>>();
    rarities.sort();

    println!("{:?}", rarities);
    // let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();

    // let cards = ["OP01-001", "OP01-002", "OP01-003"];

    // for card in cards {
    //     std::process::Command::new("zed")
    //         .arg(format!(
    //             "/Users/blujay/Documents/OnePiece/scraper/cache/images/{card}.png"
    //         ))
    //         .output()
    //         .unwrap();

    //     std::thread::sleep(Duration::from_millis(50));

    //     enigo
    //         .key(enigo::Key::Meta, enigo::Direction::Press)
    //         .unwrap();
    //     enigo
    //         .key(enigo::Key::Unicode(']'), enigo::Direction::Press)
    //         .unwrap();

    //     enigo
    //         .key(enigo::Key::Meta, enigo::Direction::Release)
    //         .unwrap();
    //     enigo
    //         .key(enigo::Key::Unicode(']'), enigo::Direction::Release)
    //         .unwrap();

    //     std::process::Command::new("zed")
    //         .arg("-w")
    //         .arg(format!(
    //             "/Users/blujay/Documents/OnePiece/runtime/scripts/OP01/{card}.lua"
    //         ))
    //         .output()
    //         .unwrap();

    //     std::thread::sleep(Duration::from_millis(50));

    //     enigo
    //         .key(enigo::Key::Meta, enigo::Direction::Press)
    //         .unwrap();
    //     enigo
    //         .key(enigo::Key::Unicode('['), enigo::Direction::Press)
    //         .unwrap();

    //     enigo
    //         .key(enigo::Key::Meta, enigo::Direction::Release)
    //         .unwrap();
    //     enigo
    //         .key(enigo::Key::Unicode('['), enigo::Direction::Release)
    //         .unwrap();

    //     std::thread::sleep(Duration::from_millis(50));
    // }

    // let lua = mlua::Lua::new();

    // lua.load("print(\"Hello world from Lua!\")").exec().unwrap();
}
