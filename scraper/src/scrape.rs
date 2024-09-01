use data::{Attribute, CardData, CardId, CardType, Color, Rarity, SetId, Subtype};
use scraper::{ElementRef, Html};
use std::{any::Any, path::Path, str::FromStr};

fn find_cardlist_element(dom: ElementRef) -> ElementRef {
    for el in dom.child_elements() {
        if Some("cardlist") == el.attr("id") {
            return el;
        }
    }

    panic!("Unable to find cardlist")
}

fn find_maincol(el: ElementRef) -> ElementRef {
    let mut found = None;
    for node in el.child_elements() {
        if Some("mainCol") == node.attr("class") {
            found = Some(node);
        }
    }

    let node = found.expect("Unable to find mainCol");

    for node in node.child_elements() {
        if node.value().name() == "article" {
            return node;
        }
    }

    panic!("Unable to find attribute in mainCol");
}

fn find_resultcol(el: ElementRef) -> ElementRef {
    let mut found = None;
    for node in el.child_elements() {
        if Some("contentsWrap isIndex") == node.attr("class") {
            found = Some(node);
        }
    }

    let el = found.expect("Unable to find contentsWrap isIndex");

    for node in el.child_elements() {
        if Some("resultCol") == node.attr("class") {
            return node;
        }
    }

    panic!("Unable to find resultCol");
}

fn gather_cards(el: ElementRef) -> Vec<ElementRef> {
    let mut cards = vec![];

    for node in el.child_elements() {
        if node.value().name() == "dl" && Some("modalCol") == node.attr("class") {
            cards.push(node);
        }
    }

    cards
}

fn get_child_element_by_name<'a>(el: ElementRef<'a>, name: &str) -> Option<ElementRef<'a>> {
    for node in el.child_elements() {
        if node.value().name() == name {
            return Some(node);
        }
    }

    None
}

fn get_child_element_by_class<'a>(el: ElementRef<'a>, classes: &str) -> Option<ElementRef<'a>> {
    for node in el.child_elements() {
        if Some(classes) == node.attr("class") {
            return Some(node);
        }
    }

    None
}

fn collect_all_elements_with_name<'a>(el: ElementRef<'a>, name: &str) -> Vec<ElementRef<'a>> {
    let mut out = vec![];
    for node in el.child_elements() {
        if node.value().name() == name {
            out.push(node);
        }
    }

    out
}

pub fn card_data_from_el(el: ElementRef, set_id: SetId) -> CardData {
    let dd = get_child_element_by_name(el, "dd").unwrap();
    let front_col = get_child_element_by_class(dd, "frontCol").unwrap();
    let img = get_child_element_by_name(front_col, "img").unwrap();

    let local_url = img
        .attr("data-src")
        .unwrap()
        .split_once('?')
        .map(|(first, _)| first)
        .unwrap();

    let back_col = get_child_element_by_class(dd, "backCol").unwrap();
    let divs = collect_all_elements_with_name(back_col, "div");

    let mut cost_life = None;
    let mut attribute = None;
    let mut power = None;
    let mut counter = None;
    let mut colors = None;
    let mut subtype = None;
    let mut effect = None;
    let mut trigger = None;

    for div in divs {
        if Some("color") == div.attr("class") {
            for item in div.text() {
                if !item.contains("Color") {
                    colors = Some(item.split('/').map(|s| s.to_string()).collect::<Vec<_>>());
                    break;
                }
            }
        } else if Some("feature") == div.attr("class") {
            for item in div.text() {
                if !item.contains("Type") {
                    subtype = Some(
                        item.split("/")
                            .map(|subtype| subtype.to_string())
                            .collect::<Vec<_>>(),
                    );
                    break;
                }
            }
        } else if Some("text") == div.attr("class") {
            let mut card_effect = String::new();
            for item in div.text() {
                if item != "Effect" && item != "-" {
                    if card_effect.is_empty() {
                        card_effect = item.to_string();
                    } else {
                        card_effect.push('\n');
                        card_effect.push_str(item);
                    }
                }
            }

            effect = Some((!card_effect.is_empty()).then_some(card_effect));
        } else if Some("trigger") == div.attr("class") {
            for item in div.text() {
                if item != "Trigger" {
                    trigger = Some((item != "-").then(|| item.to_string()));
                    break;
                }
            }
        } else if Some("col2") == div.attr("class") {
            let divs = collect_all_elements_with_name(div, "div");
            for div in divs {
                if Some("cost") == div.attr("class") {
                    for item in div.text() {
                        if !item.contains("Cost") && !item.contains("Life") {
                            if item == "-" {
                                cost_life = Some(0);
                            } else {
                                cost_life = Some(item.parse::<usize>().unwrap());
                            }
                            break;
                        }
                    }
                } else if Some("attribute") == div.attr("class") {
                    for item in div.text() {
                        let item = item.trim();
                        if !item.contains("Attribute") && !item.is_empty() {
                            attribute = Some(
                                item.split('/')
                                    .filter(|item| *item != "-")
                                    .map(|a| Attribute::from_str(a).unwrap())
                                    .collect::<Vec<_>>(),
                            );
                            break;
                        }
                    }
                } else if Some("power") == div.attr("class") {
                    for item in div.text() {
                        if !item.contains("Power") {
                            power = Some((item != "-").then(|| item.parse::<usize>().unwrap()));
                            break;
                        }
                    }
                } else if Some("counter") == div.attr("class") {
                    for item in div.text() {
                        if !item.contains("Counter") {
                            counter = Some((item != "-").then(|| item.parse::<usize>().unwrap()));
                            break;
                        }
                    }
                }
            }
        }
    }

    let dt = get_child_element_by_name(el, "dt").unwrap();
    let div = get_child_element_by_class(dt, "infoCol").unwrap();

    let spans = collect_all_elements_with_name(div, "span");
    let name = get_child_element_by_class(dt, "cardName").unwrap();

    let id = spans[0].text().next().unwrap().trim().to_string();
    let rarity = spans[1].text().next().unwrap().trim().to_string();
    let ty = spans[2].text().next().unwrap().trim().to_string();
    let name = name.text().next().unwrap().trim().to_string();

    println!("Parsing {}", id);

    CardData {
        id: CardId::from_str(&id).unwrap(),
        release_set: set_id,
        rarity: Rarity::from_str(&rarity).unwrap(),
        ty: CardType::from_str(&ty).unwrap(),
        name,
        image_name: local_url.rsplit_once('/').unwrap().1.to_string(),
        cost_life: cost_life.unwrap(),
        power: power.unwrap(),
        counter: counter.unwrap(),
        color: colors
            .unwrap()
            .into_iter()
            .map(|c| Color::from_str(&c).unwrap())
            .collect(),
        effect: effect.unwrap(),
        trigger: trigger.flatten(),
        subtype: subtype
            .unwrap()
            .into_iter()
            .map(|subtype| Subtype::from_str(&subtype).unwrap())
            .collect(),
        attribute: attribute.unwrap(),
    }
}

static ENGLISH_SET_IDS: &[(u32, SetId)] = &[
    (569201, SetId::Extra(1)),
    (569107, SetId::Booster(7)),
    (569106, SetId::Booster(6)),
    (569105, SetId::Booster(5)),
    (569104, SetId::Booster(4)),
    (569103, SetId::Booster(3)),
    (569102, SetId::Booster(2)),
    (569101, SetId::Booster(1)),
    (569014, SetId::Starter(14)),
    (569013, SetId::Starter(13)),
    (569012, SetId::Starter(12)),
    (569011, SetId::Starter(11)),
    (569010, SetId::Starter(10)),
    (569009, SetId::Starter(9)),
    (569008, SetId::Starter(8)),
    (569007, SetId::Starter(7)),
    (569006, SetId::Starter(6)),
    (569005, SetId::Starter(5)),
    (569004, SetId::Starter(4)),
    (569003, SetId::Starter(3)),
    (569002, SetId::Starter(2)),
    (569001, SetId::Starter(1)),
];

static JAPANESE_SET_IDS: &[(u32, SetId)] = &[
    (556701, SetId::Promo),
    (556901, SetId::Promo),
    (556801, SetId::Promo),
    (556301, SetId::PremiumBooster(1)),
    (556201, SetId::Extra(1)),
    (556107, SetId::Booster(8)),
    (556107, SetId::Booster(7)),
    (556106, SetId::Booster(6)),
    (556105, SetId::Booster(5)),
    (556104, SetId::Booster(4)),
    (556103, SetId::Booster(3)),
    (556102, SetId::Booster(2)),
    (556101, SetId::Booster(1)),
    (556020, SetId::Starter(20)),
    (556019, SetId::Starter(19)),
    (556018, SetId::Starter(18)),
    (556017, SetId::Starter(17)),
    (556016, SetId::Starter(16)),
    (556015, SetId::Starter(15)),
    (556014, SetId::Starter(14)),
    (556013, SetId::Starter(13)),
    (556012, SetId::Starter(12)),
    (556011, SetId::Starter(11)),
    (556010, SetId::Starter(10)),
    (556009, SetId::Starter(9)),
    (556008, SetId::Starter(8)),
    (556007, SetId::Starter(7)),
    (556006, SetId::Starter(6)),
    (556005, SetId::Starter(5)),
    (556004, SetId::Starter(4)),
    (556003, SetId::Starter(3)),
    (556002, SetId::Starter(2)),
    (556001, SetId::Starter(1)),
];

fn distribute<F, T, R>(
    tasks: Vec<T>,
    max: usize,
    f: F,
) -> Result<Vec<R>, Box<dyn Any + Send + 'static>>
where
    F: Fn(T) -> R + Send + Sync + Clone,
    T: Send,
    R: Send,
{
    let mut balanced = vec![];
    for _ in 0..max {
        balanced.push(vec![]);
    }

    for (idx, task) in tasks.into_iter().enumerate() {
        balanced[idx % max].push(task);
    }

    std::thread::scope(move |scope| {
        let mut handles = vec![];
        for set in balanced {
            if set.is_empty() {
                continue;
            };

            let f = f.clone();

            handles.push(scope.spawn(move || {
                let mut results = Vec::with_capacity(set.len());
                for task in set {
                    results.push(f(task));
                }
                results
            }));
        }

        let mut results = vec![];

        for handle in handles {
            results.extend(handle.join()?);
        }

        Ok(results)
    })
}

fn scrape_ids(ids: &[(u32, SetId)], tld: &str, path: &Path) {
    std::fs::create_dir_all(path.join("html")).unwrap();
    std::fs::create_dir_all(path.join("images")).unwrap();
    use std::fmt::Write;
    let non_cached_ids = ids
        .iter()
        .filter(|(id, _)| !path.join(format!("html/{id}.html")).exists())
        .map(|(id, _)| *id)
        .collect();

    distribute(non_cached_ids, 16, |id| {
        let response = ureq::get(&format!(
            "https://{tld}.onepiece-cardgame.com/cardlist/?series={id}"
        ))
        .call()
        .unwrap()
        .into_string()
        .unwrap();
        println!("Fetched https://{tld}.onepiece-cardgame.com/cardlist/?series={id}");
        std::fs::write(path.join(format!("html/{id}.html")), &response).unwrap();
    })
    .unwrap();

    let mut all_cards = vec![];
    for (url_id, set_id) in ids {
        let html = std::fs::read_to_string(path.join(format!("html/{url_id}.html"))).unwrap();
        let html = Html::parse_document(&html);
        println!("Parsed html/{url_id}.html");

        let cardlist = find_cardlist_element(html.root_element());
        let maincol = find_maincol(cardlist);
        let resultcol = find_resultcol(maincol);
        let raw_cards = gather_cards(resultcol);

        all_cards.extend(
            raw_cards
                .into_iter()
                .map(|card| card_data_from_el(card, *set_id)),
        );
    }

    let non_cached_images = all_cards
        .iter()
        .filter(|card| !path.join(format!("images/{}", card.image_name)).exists())
        .map(|card| {
            (
                format!(
                    "https://{tld}.onepiece-cardgame.com/images/cardlist/card/{}",
                    card.image_name
                ),
                &card.image_name,
            )
        })
        .collect();

    distribute(non_cached_images, 32, |(url, name)| {
        let mut buffer = vec![];
        ureq::get(&url)
            .call()
            .unwrap()
            .into_reader()
            .read_to_end(&mut buffer)
            .unwrap();
        println!("Downloaded {url}");
        std::fs::write(path.join(format!("images/{name}")), &buffer).unwrap();
    })
    .unwrap();

    let mut output = String::new();
    all_cards.sort_by_key(|a| a.id);
    for card in all_cards.iter() {
        writeln!(&mut output, "{}", serde_json::to_string(card).unwrap()).unwrap();
    }

    std::fs::write(path.join("card_db.jsonl"), &output).unwrap()
}

pub fn scrape() {
    scrape_ids(ENGLISH_SET_IDS, "en", Path::new("./cache/en"));
    scrape_ids(JAPANESE_SET_IDS, "asia-en", Path::new("./cache/jp"));
}
