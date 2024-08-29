use card_view::{CardState, CardViewer, ViewMessage};
use data::{CardData, CardId};
use iced::{
    executor,
    widget::{button, text, vertical_space, Button, Column, Text},
    Application, Command, Settings, Theme,
};
use serde_json::Deserializer;

mod card_view;

struct DatabaseApp {
    cards: Vec<CardState>,
    selected: usize,
    viewer: CardViewer,
}

#[derive(Debug, Clone)]
enum DbMessage {
    ChangeCard(CardId),
    Viewer(ViewMessage),
}

impl DatabaseApp {
    pub fn new() -> Self {
        let cards = std::fs::read_to_string("../scraper/cache/card_db_2.jsonl").unwrap();
        let cards = Deserializer::from_str(&cards)
            .into_iter::<CardData>()
            .map(|card| CardState::from(&card.unwrap()))
            .collect();
        Self {
            cards,
            selected: 0,
            viewer: CardViewer::new(),
        }
    }
}

impl Default for DatabaseApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Application for DatabaseApp {
    type Message = DbMessage;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::new(), Command::none())
    }

    fn title(&self) -> String {
        "Database App".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            DbMessage::ChangeCard(card) => {
                self.selected = self.cards.iter().position(|c| c.id() == card).unwrap();
            }
            DbMessage::Viewer(viewer) => self.viewer.update(&mut self.cards[self.selected], viewer),
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let mut card_ids = Column::new();
        card_ids = card_ids.extend(self.cards.iter().flat_map(|card| {
            [
                button(text(card.id()))
                    .width(120.0)
                    .on_press(DbMessage::ChangeCard(card.id()))
                    .into(),
                vertical_space().height(3.0).into(),
            ]
        }));

        iced::widget::row![
            iced::widget::scrollable(card_ids),
            self.viewer
                .view(&self.cards[self.selected])
                .map(DbMessage::Viewer)
        ]
        .into()
    }
}

fn main() {
    DatabaseApp::run(Settings {
        antialiasing: true,
        ..Default::default()
    })
    .unwrap();
}
