use std::{
    path::Path,
    str::FromStr,
    sync::{Arc, RwLock},
};

use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::{get, on, post, MethodFilter},
    Json,
};
use data::{Attribute, CardData, CardId, Color, Subtype};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_util::io::ReaderStream;

async fn get_index() -> Html<String> {
    Html(
        tokio::fs::read_to_string("assets/index.html")
            .await
            .unwrap(),
    )
}

#[derive(Serialize, Deserialize)]
pub struct CardMetadata {
    pub id: String,
    pub name: String,
    pub ty: String,
    pub subtypes: Vec<String>,
    pub colors: Vec<String>,
    pub attributes: Vec<String>,
    pub cost_life: usize,
    pub power: Option<usize>,
    pub counter: Option<usize>,
    pub effect: Option<String>,
    pub trigger: Option<String>,
}

pub async fn get_initial_metadata() -> Json<CardMetadata> {
    Json(CardMetadata {
        id: "EB01-002".to_string(),
        name: "Izo".to_string(),
        ty: "Leader".to_string(),
        subtypes: vec!["Land of Wano".to_string(), "Whitebeard Pirates".to_string()],
        colors: vec!["Red".to_string()],
        attributes: vec!["Ranged".to_string()],
        cost_life: 5,
        power: Some(7000),
        counter: None,
        effect: Some(
            "[On Play] Give up to 1 rested Don!! card to your Leader or 1 of your Characters."
                .to_string(),
        ),
        trigger: None,
    })
}

pub async fn submit_metadata(
    State(db): State<Arc<RwLock<Vec<CardData>>>>,
    Json(meta): Json<CardMetadata>,
) {
    let id = CardId::from_str(&meta.id).unwrap();
    let mut db = db.write().unwrap();
    let card = db.iter_mut().find(|card| card.id == id).unwrap();

    card.color = meta
        .colors
        .into_iter()
        .map(|c| Color::from_str(&c).unwrap())
        .collect();
    card.attribute = meta
        .attributes
        .into_iter()
        .map(|a| Attribute::from_str(&a).unwrap())
        .collect();
    card.subtype = meta
        .subtypes
        .into_iter()
        .map(|s| Subtype::from_str(&s).unwrap())
        .collect();
    card.trigger = meta.trigger;
    card.effect = meta.effect;
    card.power = meta.power;
    card.counter = meta.counter;
    card.cost_life = meta.cost_life;
}

pub async fn next_meta(
    axum::extract::Path(current): axum::extract::Path<String>,
    State(db): State<Arc<RwLock<Vec<CardData>>>>,
) -> Json<CardMetadata> {
    let id = CardId::from_str(&current).unwrap();
    let db = db.read().unwrap();
    let pos = db.iter().position(|card| card.id == id).unwrap();
    let next = &db[pos + 1 % db.len()];
    Json(CardMetadata {
        id: next.id.to_string(),
        name: next.name.clone(),
        ty: format!("{:?}", next.ty),
        subtypes: next
            .subtype
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        colors: next
            .color
            .iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>(),
        attributes: next
            .attribute
            .iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>(),
        cost_life: next.cost_life,
        power: next.power,
        counter: next.counter,
        effect: next.effect.clone(),
        trigger: next.trigger.clone(),
    })
}

pub async fn prev_meta(
    axum::extract::Path(current): axum::extract::Path<String>,
    State(db): State<Arc<RwLock<Vec<CardData>>>>,
) -> Json<CardMetadata> {
    let id = CardId::from_str(&current).unwrap();
    let db = db.read().unwrap();
    let pos = db.iter().position(|card| card.id == id).unwrap();
    let next = &db[pos.wrapping_sub(1) % db.len()];
    Json(CardMetadata {
        id: next.id.to_string(),
        name: next.name.clone(),
        ty: format!("{:?}", next.ty),
        subtypes: next
            .subtype
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        colors: next
            .color
            .iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>(),
        attributes: next
            .attribute
            .iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>(),
        cost_life: next.cost_life,
        power: next.power,
        counter: next.counter,
        effect: next.effect.clone(),
        trigger: next.trigger.clone(),
    })
}

pub async fn get_image(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    let path = Path::new("../scraper/cache/images").join(format!("{id}.png"));
    let filename = match path.file_name() {
        Some(name) => name,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "File name couldn't be determined".to_string(),
            ))
        }
    };

    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    let content_type = match mime_guess::from_path(&path).first_raw() {
        Some(mime) => mime,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "MIME Type couldn't be determined".to_string(),
            ))
        }
    };

    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, content_type.to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{:?}\"", filename),
        ),
    ];
    Ok((headers, body))
}

pub async fn get_file(uri: Uri) -> impl IntoResponse {
    let path = uri.path().strip_prefix("/").unwrap();
    let filename = match Path::new(path).file_name() {
        Some(name) => name,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "File name couldn't be determined".to_string(),
            ))
        }
    };

    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    let content_type = match mime_guess::from_path(&path).first_raw() {
        Some(mime) => mime,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "MIME Type couldn't be determined".to_string(),
            ))
        }
    };

    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, content_type.to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{:?}\"", filename),
        ),
    ];
    Ok((headers, body))
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let file = tokio::fs::read_to_string("../scraper/cache/card_db_2.jsonl")
        .await
        .unwrap();
    let card_db: Vec<CardData> = serde_json::Deserializer::from_str(&file)
        .into_iter::<CardData>()
        .map(|card| card.unwrap())
        .collect();

    let router = axum::Router::new()
        .route("/", get(get_index))
        .route("/start", get(get_initial_metadata))
        .route("/submit", post(submit_metadata))
        .route("/next/:current", get(next_meta))
        .route("/prev/:current", get(prev_meta))
        .route("/images/:id", get(get_image))
        .fallback(on(MethodFilter::GET, get_file))
        .with_state(Arc::new(RwLock::new(card_db)));

    axum::serve(TcpListener::bind("0.0.0.0:8080").await.unwrap(), router)
        .await
        .unwrap();
}
