use std::fmt::Display;

use data::{Attribute, CardData, CardId, CardType, Color, Subtype};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, checkbox,
        combo_box::State as ComboState,
        horizontal_rule, horizontal_space,
        image::{self, Handle},
        pick_list, row, text, text_editor,
        text_editor::{Action, Content},
        text_input, vertical_rule, vertical_space, ComboBox, Row,
    },
    Alignment, Element,
};
use nonempty::NonEmpty;

enum UniqueCardData {
    Leader {
        life: usize,
        power: usize,
        attribute: Attribute,
        secondary_attribute: Option<Attribute>,
        secondary_color: Option<Color>,
    },
    Character {
        cost: usize,
        power: usize,
        attribute: Attribute,
        secondary_attribute: Option<Attribute>,
        counter: Option<usize>,
        trigger: Option<String>,
    },
    Stage {
        cost: usize,
        trigger: Option<String>,
    },
    Event {
        cost: usize,
        trigger: Option<String>,
    },
}

impl UniqueCardData {
    fn variant(&self) -> &'static CardType {
        match self {
            Self::Leader { .. } => &CardType::Leader,
            Self::Character { .. } => &CardType::Character,
            Self::Stage { .. } => &CardType::Stage,
            Self::Event { .. } => &CardType::Event,
        }
    }

    fn convert_to_variant(self, new_variant: CardType) -> Self {
        if *self.variant() == new_variant {
            return self;
        }

        let cost_life;
        let attr;
        let second_attr;
        let power_;
        let trigger_;
        match self {
            Self::Leader {
                life,
                power,
                attribute,
                secondary_attribute,
                ..
            } => {
                cost_life = life;
                power_ = power;
                trigger_ = None;
                attr = Some(attribute);
                second_attr = secondary_attribute;
            }
            Self::Character {
                cost,
                power,
                trigger,
                attribute,
                secondary_attribute,
                ..
            } => {
                cost_life = cost;
                power_ = power;
                trigger_ = trigger;
                attr = Some(attribute);
                second_attr = secondary_attribute
            }
            Self::Stage { cost, trigger } | Self::Event { cost, trigger } => {
                cost_life = cost;
                power_ = 0;
                trigger_ = trigger;
                attr = None;
                second_attr = None;
            }
        }

        match new_variant {
            CardType::Leader => Self::Leader {
                life: cost_life,
                power: power_,
                attribute: attr.unwrap(),
                secondary_attribute: second_attr,
                secondary_color: None,
            },
            CardType::Character => Self::Character {
                cost: cost_life,
                power: power_,
                counter: None,
                attribute: attr.unwrap(),
                secondary_attribute: second_attr,
                trigger: trigger_,
            },
            CardType::Stage => Self::Stage {
                cost: cost_life,
                trigger: trigger_,
            },
            CardType::Event => Self::Event {
                cost: cost_life,
                trigger: trigger_,
            },
        }
    }
}

pub struct CardState {
    id: CardId,
    image: Handle,
    color: Color,
    card: UniqueCardData,
    subtypes: NonEmpty<Subtype>,
    effects: Vec<Content>,
    trigger: Option<String>,
}

impl CardState {
    pub fn id(&self) -> CardId {
        self.id
    }
}

impl From<&CardData> for CardState {
    fn from(value: &CardData) -> Self {
        Self {
            id: value.id,
            image: Handle::from_path(format!("../scraper/cache/images/{}.png", value.id)),
            color: value.color[0],
            card: match value.ty {
                CardType::Leader => UniqueCardData::Leader {
                    life: value.cost_life,
                    power: value.power.unwrap_or_default(),
                    attribute: value.attribute[0],
                    secondary_attribute: value.attribute.get(1).copied(),
                    secondary_color: value.color.get(1).copied(),
                },
                CardType::Character => UniqueCardData::Character {
                    cost: value.cost_life,
                    power: value.power.unwrap_or_default(),
                    counter: value.counter,
                    attribute: value.attribute[0],
                    secondary_attribute: value.attribute.get(1).copied(),
                    trigger: value.trigger.clone(),
                },
                CardType::Stage => UniqueCardData::Stage {
                    cost: value.cost_life,
                    trigger: value.trigger.clone(),
                },
                CardType::Event => UniqueCardData::Event {
                    cost: value.cost_life,
                    trigger: value.trigger.clone(),
                },
            },
            subtypes: NonEmpty::from_vec(value.subtype.clone()).unwrap(),
            effects: value
                .effect
                .iter()
                .map(|effect| Content::with_text(effect.as_str()))
                .collect(),
            trigger: value.trigger.clone(),
        }
    }
}

pub struct CardViewer {
    subtype_state: ComboState<Subtype>,
}

impl CardViewer {
    pub fn new() -> Self {
        Self {
            subtype_state: ComboState::new(Subtype::ALL.to_vec()),
        }
    }

    pub fn update(&mut self, card: &mut CardState, message: ViewMessage) {
        match message {
            ViewMessage::ChangeCardType(ty) => {
                // SAFETY: We are doing a basic mutation and don't have a default
                unsafe {
                    let old_card = std::ptr::read(&card.card);
                    std::ptr::write(&mut card.card, old_card.convert_to_variant(ty));
                }
            }
            ViewMessage::ChangeColor(color) => card.color = color,
            ViewMessage::ChangeSecondaryColor(color) => {
                let UniqueCardData::Leader {
                    secondary_color, ..
                } = &mut card.card
                else {
                    unreachable!()
                };

                *secondary_color = match color {
                    Secondary::None => None,
                    Secondary::Secondary(color) => Some(color),
                };
            }
            ViewMessage::ChangeAttribute(attr) => match &mut card.card {
                UniqueCardData::Leader { attribute, .. }
                | UniqueCardData::Character { attribute, .. } => {
                    *attribute = attr;
                }
                _ => unreachable!(),
            },
            ViewMessage::ChangeSecondaryAttribute(attr) => match &mut card.card {
                UniqueCardData::Leader {
                    secondary_attribute,
                    ..
                }
                | UniqueCardData::Character {
                    secondary_attribute,
                    ..
                } => {
                    *secondary_attribute = match attr {
                        Secondary::None => None,
                        Secondary::Secondary(v) => Some(v),
                    };
                }
                _ => unreachable!(),
            },
            ViewMessage::SetCost(new_cost) => match &mut card.card {
                UniqueCardData::Leader { life, .. } => *life = new_cost,
                UniqueCardData::Character { cost, .. }
                | UniqueCardData::Stage { cost, .. }
                | UniqueCardData::Event { cost, .. } => *cost = new_cost,
            },
            ViewMessage::SetPower(new_power) => match &mut card.card {
                UniqueCardData::Leader { power, .. } | UniqueCardData::Character { power, .. } => {
                    *power = new_power;
                }
                _ => unreachable!(),
            },
            ViewMessage::SetCounter(new_counter) => match &mut card.card {
                UniqueCardData::Character { counter, .. } => {
                    *counter = new_counter;
                }
                _ => unreachable!(),
            },
            ViewMessage::ChangeSubtype(old, new) => match (old, new) {
                (None, None) => unreachable!(),
                (None, Some(new)) => {
                    if !card.subtypes.contains(&new) {
                        card.subtypes.push(new);
                    }
                }
                (Some(old), new) => {
                    let pos = card.subtypes.iter().position(|st| *st == old).unwrap();
                    if let Some(new) = new {
                        if !card.subtypes.contains(&new) {
                            card.subtypes[pos] = new;
                        }
                    } else if pos == 0 {
                        let (_, tail) = card.subtypes.split_first();
                        card.subtypes = NonEmpty::from_slice(tail).unwrap();
                    } else {
                        let (head, tail) = card.subtypes.split_first();
                        let mut new_list = vec![*head];
                        let pos = pos - 1;
                        new_list.extend(&tail[..pos]);
                        new_list.extend(&tail[pos + 1..]);
                        card.subtypes = NonEmpty::from_vec(new_list).unwrap();
                    }
                }
            },
            ViewMessage::SetEffect(idx, eff) => {
                card.effects[idx].perform(eff);
            }
            ViewMessage::RemoveEffect(idx) => {
                card.effects.remove(idx);
            }
            ViewMessage::NewEffect => {
                card.effects.push(Content::new());
            }
            ViewMessage::SetTrigger(trigger) => {
                card.trigger = trigger;
            }
            ViewMessage::Null => {}
        }
    }

    pub fn view<'a>(&'a self, card: &'a CardState) -> Element<'a, ViewMessage> {
        const COLUMN_WIDTH: f32 = 160.0;

        let mut edit_col = iced::widget::column![
            row![
                text("Card Kind").width(COLUMN_WIDTH),
                horizontal_space().width(10.0),
                pick_list(
                    [
                        CardType::Leader,
                        CardType::Character,
                        CardType::Stage,
                        CardType::Event,
                    ],
                    Some(*card.card.variant()),
                    ViewMessage::ChangeCardType,
                )
                .width(200.0),
            ]
            .align_items(Alignment::Center),
            vertical_space().height(3.0),
            row![
                text("Primary Color").width(COLUMN_WIDTH),
                horizontal_space().width(10.0),
                pick_list(
                    [
                        Color::Red,
                        Color::Green,
                        Color::Blue,
                        Color::Purple,
                        Color::Black,
                        Color::Yellow
                    ],
                    Some(card.color),
                    ViewMessage::ChangeColor
                )
                .width(200.0)
            ]
            .align_items(Alignment::Center),
        ];

        if let UniqueCardData::Leader {
            secondary_color, ..
        } = &card.card
        {
            let color = secondary_color
                .map(Secondary::Secondary)
                .unwrap_or_default();
            edit_col = edit_col.extend([
                vertical_space().height(3.0).into(),
                row![
                    text("Secondary Color").width(COLUMN_WIDTH),
                    horizontal_space().width(10.0),
                    pick_list(
                        [
                            Secondary::None,
                            Secondary::Secondary(Color::Red),
                            Secondary::Secondary(Color::Green),
                            Secondary::Secondary(Color::Blue),
                            Secondary::Secondary(Color::Purple),
                            Secondary::Secondary(Color::Black),
                            Secondary::Secondary(Color::Yellow),
                        ],
                        Some(color),
                        ViewMessage::ChangeSecondaryColor
                    )
                    .width(200.0)
                ]
                .align_items(Alignment::Center)
                .into(),
            ]);
        }

        if let UniqueCardData::Character {
            attribute,
            secondary_attribute,
            ..
        }
        | UniqueCardData::Leader {
            attribute,
            secondary_attribute,
            ..
        } = &card.card
        {
            let secondary = secondary_attribute
                .map(Secondary::Secondary)
                .unwrap_or_default();
            edit_col = edit_col.extend([
                vertical_space().height(3.0).into(),
                row![
                    text("Attribute").width(COLUMN_WIDTH),
                    horizontal_space().width(10.0),
                    pick_list(
                        [
                            Attribute::Ranged,
                            Attribute::Slash,
                            Attribute::Special,
                            Attribute::Strike,
                            Attribute::Wisdom,
                        ],
                        Some(attribute),
                        ViewMessage::ChangeAttribute
                    )
                    .width(200)
                ]
                .align_items(Alignment::Center)
                .into(),
                vertical_space().height(3.0).into(),
                row![
                    text("Secondary Attribute").width(COLUMN_WIDTH),
                    horizontal_space().width(10.0),
                    pick_list(
                        [
                            Secondary::None,
                            Secondary::Secondary(Attribute::Ranged),
                            Secondary::Secondary(Attribute::Slash),
                            Secondary::Secondary(Attribute::Special),
                            Secondary::Secondary(Attribute::Strike),
                            Secondary::Secondary(Attribute::Wisdom),
                        ],
                        Some(secondary),
                        ViewMessage::ChangeSecondaryAttribute
                    )
                    .width(200)
                ]
                .align_items(Alignment::Center)
                .into(),
            ]);
        }

        enum EditableNumber {
            Optional(Option<usize>),
            Always(usize),
        }

        fn num_edit<'a>(
            label: &str,
            current: EditableNumber,
            step: usize,
            change: &'a impl Fn(EditableNumber) -> ViewMessage,
        ) -> Row<'a, ViewMessage> {
            let mut row = Row::new();
            row = row.extend([
                text(label).width(COLUMN_WIDTH).into(),
                horizontal_space().width(10.0).into(),
            ]);

            match current {
                EditableNumber::Optional(optional) => {
                    row = row.extend([
                        checkbox("", optional.is_some())
                            .on_toggle(|b| change(EditableNumber::Optional(b.then_some(0))))
                            .into(),
                        horizontal_space().width(10.0).into(),
                    ]);

                    if let Some(current) = optional {
                        row = row.extend([
                            text(current)
                                .horizontal_alignment(Horizontal::Right)
                                .width(40.0)
                                .into(),
                            horizontal_space().width(20.0).into(),
                            button(text("-").horizontal_alignment(Horizontal::Center))
                                .width(45.0)
                                .on_press_maybe((current > 0).then(|| {
                                    change(EditableNumber::Optional(Some(
                                        current.saturating_sub(step),
                                    )))
                                }))
                                .into(),
                            horizontal_space().width(5.0).into(),
                            button(text("+").horizontal_alignment(Horizontal::Center))
                                .width(45.0)
                                .on_press(change(EditableNumber::Optional(Some(current + step))))
                                .into(),
                        ]);
                    }
                }
                EditableNumber::Always(current) => {
                    row = row.extend([
                        text(current)
                            .horizontal_alignment(Horizontal::Right)
                            .width(80.0)
                            .into(),
                        horizontal_space().width(20.0).into(),
                        button(text("-").horizontal_alignment(Horizontal::Center))
                            .width(45.0)
                            .on_press_maybe((current > 0).then(|| {
                                change(EditableNumber::Always(current.saturating_sub(step)))
                            }))
                            .into(),
                        horizontal_space().width(5.0).into(),
                        button(text("+").horizontal_alignment(Horizontal::Center))
                            .width(45.0)
                            .on_press(change(EditableNumber::Always(current + step)))
                            .into(),
                    ]);
                }
            };

            row.align_items(Alignment::Center)
        }

        let (cost_life, label) = match &card.card {
            UniqueCardData::Leader { life, .. } => (*life, "Life"),
            UniqueCardData::Character { cost, .. }
            | UniqueCardData::Stage { cost, .. }
            | UniqueCardData::Event { cost, .. } => (*cost, "Cost"),
        };

        edit_col = edit_col.extend([
            vertical_space().height(3.0).into(),
            num_edit(label, EditableNumber::Always(cost_life), 1, &|value| {
                let EditableNumber::Always(cost_life) = value else {
                    unreachable!()
                };

                ViewMessage::SetCost(cost_life)
            })
            .into(),
        ]);

        if let UniqueCardData::Leader { power, .. } | UniqueCardData::Character { power, .. } =
            &card.card
        {
            edit_col = edit_col.extend([
                vertical_space().height(3.0).into(),
                num_edit("Power", EditableNumber::Always(*power), 1000, &|value| {
                    let EditableNumber::Always(power) = value else {
                        unreachable!()
                    };

                    ViewMessage::SetPower(power)
                })
                .into(),
            ]);
        }

        if let UniqueCardData::Character { counter, .. } = &card.card {
            edit_col = edit_col.extend([
                vertical_space().height(3.0).into(),
                num_edit(
                    "Counter",
                    EditableNumber::Optional(*counter),
                    1000,
                    &|value| {
                        let EditableNumber::Optional(counter) = value else {
                            unreachable!()
                        };

                        ViewMessage::SetCounter(counter)
                    },
                )
                .into(),
            ])
        }

        edit_col = edit_col.extend([
            vertical_space().height(8.0).into(),
            text("Subtypes").size(24.0).into(),
            vertical_space().height(2.0).into(),
            horizontal_rule(2.0).into(),
        ]);

        for subtype in card.subtypes.iter() {
            let subtype = *subtype;
            edit_col = edit_col.extend([
                vertical_space().height(3.0).into(),
                row![
                    ComboBox::new(
                        &self.subtype_state,
                        "Change subtype...",
                        Some(&subtype),
                        move |new_subtype| ViewMessage::ChangeSubtype(
                            Some(subtype),
                            Some(new_subtype)
                        ),
                    )
                    .width(275.0),
                    horizontal_space().width(10.0),
                    button(text("-").horizontal_alignment(Horizontal::Center))
                        .on_press_maybe(
                            (card.subtypes.len() > 1)
                                .then_some(ViewMessage::ChangeSubtype(Some(subtype), None))
                        )
                        .width(40.0)
                ]
                .align_items(Alignment::Center)
                .into(),
            ]);
        }

        edit_col = edit_col.extend([
            vertical_space().height(3.0).into(),
            row![ComboBox::new(
                &self.subtype_state,
                "Add subtype...",
                None,
                move |new_subtype| ViewMessage::ChangeSubtype(None, Some(new_subtype)),
            )
            .width(275.0)]
            .align_items(Alignment::Center)
            .into(),
        ]);

        edit_col = edit_col.extend([
            vertical_space().height(8.0).into(),
            text("Effects").size(24.0).into(),
            vertical_space().height(2.0).into(),
            horizontal_rule(2.0).into(),
        ]);

        for (idx, effect) in card.effects.iter().enumerate() {
            edit_col = edit_col.extend([
                vertical_space().height(3.0).into(),
                row![
                    iced::widget::column![text_editor(effect)
                        .on_action(move |action| ViewMessage::SetEffect(idx, action))]
                    .width(400.0)
                    .max_width(400.0),
                    horizontal_space().width(10.0),
                    button(
                        text("-")
                            .horizontal_alignment(Horizontal::Center)
                            .vertical_alignment(Vertical::Center)
                    )
                    .width(40.0)
                    .height(80.0)
                    .on_press(ViewMessage::RemoveEffect(idx)),
                ]
                .height(80.0)
                .into(),
            ]);
        }

        edit_col = edit_col.extend([
            vertical_space().height(3.0).into(),
            button(text("Add new effect"))
                .on_press(ViewMessage::NewEffect)
                .into(),
        ]);

        if let UniqueCardData::Character { trigger, .. }
        | UniqueCardData::Stage { trigger, .. }
        | UniqueCardData::Event { trigger, .. } = &card.card
        {
            let mut row = row![checkbox("Trigger", trigger.is_some())
                .on_toggle(|b| ViewMessage::SetTrigger(b.then(|| String::new())))];

            if let Some(trigger) = trigger {
                row = row.extend([
                    horizontal_space().width(10.0).into(),
                    text_input("", trigger.as_str())
                        .on_input(|input| ViewMessage::SetTrigger(Some(input)))
                        .width(350.0)
                        .into(),
                ]);
            }

            edit_col = edit_col.extend([
                vertical_space().height(3.0).into(),
                row.align_items(Alignment::Center).into(),
            ]);
        }

        row![
            image::viewer(card.image.clone()),
            horizontal_space().width(10.0),
            vertical_rule(3.0),
            horizontal_space().width(10.0),
            edit_col
        ]
        .into()
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Secondary<T> {
    #[default]
    None,
    Secondary(T),
}

impl<T: Display> Display for Secondary<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str("N/A"),
            Self::Secondary(value) => Display::fmt(value, f),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    ChangeCardType(CardType),
    ChangeColor(Color),
    ChangeSecondaryColor(Secondary<Color>),
    ChangeAttribute(Attribute),
    ChangeSecondaryAttribute(Secondary<Attribute>),
    SetCost(usize),
    SetPower(usize),
    SetCounter(Option<usize>),
    ChangeSubtype(Option<Subtype>, Option<Subtype>),
    SetEffect(usize, Action),
    RemoveEffect(usize),
    NewEffect,
    SetTrigger(Option<String>),
    Null,
}
