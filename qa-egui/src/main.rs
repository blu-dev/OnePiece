use std::sync::mpsc::{self, Receiver};

use data::{CardData, CardId};
use eframe::NativeOptions;
use egui::{
    emath::OrderedFloat, load::TexturePoll, Align, Color32, ColorImage, Layout, Sense, SizeHint,
    TextureHandle, TextureId, TextureOptions,
};
use egui_dock::{DockState, TabViewer};
use image::GenericImageView;

mod enum_combo;

#[derive(Default)]
pub struct CardListingState {}

pub struct CardViewState {
    card: CardData,
}

pub enum DbTab {
    CardListing(CardListingState),
    CardView(CardViewState),
}

enum ViewerCommand {
    OpenCard(CardId),
}

const BASE_CARD_SIZE: egui::Vec2 = egui::Vec2::new(240.0, 335.0);
const FULL_UVS: egui::Rect =
    egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0));

struct DbTabViewer<'a> {
    cards: &'a [LoadedCard],
    commands: &'a mut Vec<ViewerCommand>,
}

impl TabViewer for DbTabViewer<'_> {
    type Tab = DbTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            DbTab::CardListing(_) => "Listing".into(),
            DbTab::CardView(view) => view.card.id.to_string().into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DbTab::CardListing(_) => {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for card in self.cards.iter() {
                            let (ui_id, rect) = ui.allocate_space(BASE_CARD_SIZE * 0.6);
                            let response = ui.interact(rect, ui_id, Sense::click());
                            let tint = if response.hovered() {
                                Color32::from_rgb(200, 200, 200)
                            } else {
                                Color32::WHITE
                            };
                            let visuals = ui.style().interact(&response);
                            ui.painter().image(
                                card.image.id(),
                                rect.expand(visuals.expansion),
                                FULL_UVS,
                                tint,
                            );

                            if response.clicked() {
                                self.commands.push(ViewerCommand::OpenCard(card.data.id));
                            }
                        }
                    });
                });
                // ui.image("file://./cache/en/images/ST01-001.png");
            }
            DbTab::CardView(_) => {}
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, DbTab::CardListing(_))
    }
}

pub struct DbApp {
    state: DockState<DbTab>,
    cards: Vec<LoadedCard>,
    rx: Receiver<LoadedCard>,
}

impl DbApp {
    pub fn new(rx: Receiver<LoadedCard>) -> Self {
        Self {
            state: DockState::new(vec![DbTab::CardListing(Default::default())]),
            cards: vec![],
            rx,
        }
    }
}

impl eframe::App for DbApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        while let Ok(next) = self.rx.try_recv() {
            self.cards.push(next);
        }

        let mut commands = vec![];
        let mut viewer = DbTabViewer {
            cards: &self.cards,
            commands: &mut commands,
        };

        egui_dock::DockArea::new(&mut self.state).show(ctx, &mut viewer);

        for command in commands {
            match command {
                ViewerCommand::OpenCard(card_id) => {
                    let existing = self
                        .state
                        .iter_all_tabs()
                        .find(|(_, tab)| {
                            match tab {
                                DbTab::CardView(CardViewState { card }) if card.id == card_id => {
                                    return true;
                                }
                                _ => {}
                            }
                            false
                        })
                        .map(|(surface_node, _)| surface_node);

                    if let Some(existing) = existing {
                        self.state.set_focused_node_and_surface(existing);
                        continue;
                    }

                    self.state
                        .push_to_focused_leaf(DbTab::CardView(CardViewState {
                            card: self
                                .cards
                                .iter()
                                .find(|card| card.data.id == card_id)
                                .map(|card| card.data.clone())
                                .unwrap(),
                        }));
                }
            }
        }
    }
}

pub struct LoadedCard {
    data: CardData,
    image: TextureHandle,
}

fn main() {
    eframe::run_native(
        "OnePiece TCG Database App",
        NativeOptions::default(),
        Box::new(move |ctx| {
            egui_extras::install_image_loaders(&ctx.egui_ctx);

            let (tx, rx) = mpsc::channel();

            let egui_ctx = ctx.egui_ctx.clone();

            std::thread::spawn(move || {
                let card_db = std::fs::read_to_string("cache/en/card_db.jsonl").unwrap();

                for card in serde_json::Deserializer::from_str(&card_db).into_iter::<CardData>() {
                    let card = match card {
                        Ok(card) => card,
                        Err(e) => {
                            eprintln!("Failed to deserialize card data: {e}");
                            continue;
                        }
                    };

                    let image =
                        image::open(format!("cache/en/images/{}", card.image_name)).unwrap();
                    let (w, h) = image.dimensions();
                    let bytes = image.into_rgba8().into_vec();
                    let image =
                        ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &bytes);

                    let image = egui_ctx.load_texture(
                        format!("image#{}", card.image_name),
                        image,
                        TextureOptions::default(),
                    );
                    tx.send(LoadedCard { data: card, image }).unwrap();
                }
            });

            Ok(Box::new(DbApp::new(rx)))
        }),
    )
    .unwrap();
}
