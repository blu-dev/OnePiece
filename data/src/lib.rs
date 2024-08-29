use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    num::ParseIntError,
    str::FromStr,
};
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SetId {
    Starter(usize),
    Booster(usize),
    Extra(usize),
    Promo,
}

impl Display for SetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Starter(id) => write!(f, "ST{id:02}"),
            Self::Booster(id) => write!(f, "OP{id:02}"),
            Self::Extra(id) => write!(f, "EB{id:02}"),
            Self::Promo => write!(f, "P"),
        }
    }
}

#[derive(Error, Debug)]
pub enum SetIdParseError {
    #[error("Invalid set ID prefix '{0}'")]
    InvalidPrefix(String),

    #[error("Invalid set sub-id: {0}")]
    InvalidInteger(#[from] ParseIntError),
}

impl FromStr for SetId {
    type Err = SetIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "P" {
            return Ok(Self::Promo);
        }

        let len = s.len();
        let sub_id = s[len - 2..].parse::<usize>()?;

        match &s[..2] {
            "ST" => Ok(Self::Starter(sub_id)),
            "OP" => Ok(Self::Booster(sub_id)),
            "EB" => Ok(Self::Extra(sub_id)),
            other => Err(SetIdParseError::InvalidPrefix(other.to_string())),
        }
    }
}

#[derive(Error, Debug)]
pub enum CardIdParseError {
    #[error(transparent)]
    SetId(#[from] SetIdParseError),

    #[error("Invalid card ID: {0}")]
    InvalidInteger(#[from] ParseIntError),

    #[error("Card IDs should take the form of '<set>-<card>' ({0} is invalid)")]
    ImproperForm(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId {
    pub set: SetId,
    pub card: usize,
}

impl Serialize for CardId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CardId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;

        Self::from_str(&string).map_err(|e| <D::Error as serde::de::Error>::custom(e))
    }
}

impl Display for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{:03}", self.set, self.card)
    }
}

impl FromStr for CardId {
    type Err = CardIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (set_id, card_id) = s
            .split_once('-')
            .ok_or_else(|| CardIdParseError::ImproperForm(s.to_string()))?;
        let set_id = SetId::from_str(set_id)?;
        let card_id = card_id.parse::<usize>()?;
        Ok(Self {
            set: set_id,
            card: card_id,
        })
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    #[serde(rename = "L")]
    Leader,
    #[serde(rename = "C")]
    Common,
    #[serde(rename = "UC")]
    Uncommon,
    #[serde(rename = "R")]
    Rare,
    #[serde(rename = "SR")]
    SuperRare,
    #[serde(rename = "SEC")]
    SecretRare,
    #[serde(rename = "SP CARD")]
    SpecialCard,
    #[serde(rename = "TR")]
    TreasureRare,
    #[serde(rename = "P")]
    Promo,
}

#[derive(Error, Debug)]
#[error("Invalid rarity spefifier '{0}'")]
pub struct ParseRarityError(String);

impl FromStr for Rarity {
    type Err = ParseRarityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(Self::Leader),
            "C" => Ok(Self::Common),
            "UC" => Ok(Self::Uncommon),
            "R" => Ok(Self::Rare),
            "SR" => Ok(Self::SuperRare),
            "SEC" => Ok(Self::SecretRare),
            "SP CARD" => Ok(Self::SpecialCard),
            "TR" => Ok(Self::TreasureRare),
            "P" => Ok(Self::Promo),
            other => Err(ParseRarityError(other.to_string())),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum CardType {
    Leader,
    Character,
    Stage,
    Event,
}

#[derive(Error, Debug)]
#[error("Invalid card type '{0}'")]
pub struct ParseCardTypeError(String);

impl Display for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl FromStr for CardType {
    type Err = ParseCardTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LEADER" => Ok(Self::Leader),
            "CHARACTER" => Ok(Self::Character),
            "STAGE" => Ok(Self::Stage),
            "EVENT" => Ok(Self::Event),
            other => Err(ParseCardTypeError(other.to_string())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Purple,
    Black,
    Yellow,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Error, Debug)]
#[error("Invalid color '{0}'")]
pub struct ParseColorError(String);

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Red" => Ok(Self::Red),
            "Green" => Ok(Self::Green),
            "Blue" => Ok(Self::Blue),
            "Purple" => Ok(Self::Purple),
            "Black" => Ok(Self::Black),
            "Yellow" => Ok(Self::Yellow),
            other => Err(ParseColorError(other.to_string())),
        }
    }
}

macro_rules! decl_subtypes {
    ($($name:ident -> $val:literal $($alias:literal)?),*) => {
        #[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Subtype {
            $(
                #[serde(rename = $val $(, alias = $alias)?)]
                $name,
            )*
        }

        impl Subtype {
            pub const ALL: &'static [Self] = &[
                $(
                    Self::$name,
                )*
            ];
        }

        #[derive(Error, Debug)]
        #[error("Invalid subtype '{0}'")]
        pub struct ParseSubtypeError(String);

        impl FromStr for Subtype {
            type Err = ParseSubtypeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $val $(| $alias)? => Ok(Self::$name),
                    )*
                    other => Err(ParseSubtypeError(other.to_string()))
                }
            }
        }

        impl Display for Subtype {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$name => f.write_str($val),
                    )*
                }
            }
        }
    }
}

decl_subtypes! {
    Alabasta -> "Alabasta",
    AlvidaPirates -> "Alvida Pirates",
    AmazonLily -> "Amazon Lily",
    Animal -> "Animal",
    AnimalKingdomPirates -> "Animal Kingdom Pirates",
    ArlongPirates -> "Arlong Pirates",
    AsukaIsland -> "Asuka Island",
    BaroqueWorks -> "Baroque Works",
    BartoClub -> "Barto Club",
    BeautifulPirates -> "Beautiful Pirates",
    BellamyPirates -> "Bellamy Pirates",
    BigMomPirates -> "Big Mom Pirates",
    BiologicalWeapon -> "Biological Weapon",
    BlackCatPirates -> "Black Cat Pirates",
    BlackbeardPirates -> "Blackbeard Pirates",
    BluejamPirates -> "Bluejam Pirates",
    BonneyPirates -> "Bonney Pirates",
    BowinIsland -> "Bowin Island",
    BuggyPirates -> "Buggy Pirates",
    BuggysDeivery -> "Buggy's Delivery",
    CP0 -> "CP0",
    CP6 -> "CP6",
    CP7 -> "CP7",
    CP9 -> "CP9",
    CaribouPirates -> "Caribou Pirates",
    CelestialDragons -> "Celestial Dragons",
    CrownIsland -> "Crown Island",
    DonquixotePirates -> "Donquixote Pirates",
    DrakePirates -> "Drake Pirates",
    Dressrosa -> "Dressrosa",
    DrumKingdom -> "Drum Kingdom",
    EastBlue -> "East Blue",
    Egghead -> "Egghead",
    EldoraggoCrew -> "Eldoraggo Crew",
    Film -> "FILM",
    FallenMonkPirates -> "Fallen Monk Pirates",
    FiretankPirates -> "Firetank Pirates",
    FishMan -> "Fish-Man",
    FishManIsland -> "Fish-Man Island",
    FlyingPirates -> "Flying Pirates",
    FoolshoutIsland -> "Foolshout Island",
    FormerArlongPirates -> "Former Arlong Pirates",
    FormerBaroqueWorks -> "Former Baroque Works",
    FormerNavy -> "Former Navy",
    FormerRocksPirates -> "Former Rocks Pirates",
    FormerRogerPirates -> "Former Roger Pirates",
    FormerRumbarPirates -> "Former Rumbar Pirates",
    FormerWhitebeardPirates -> "Former Whitebeard Pirates",
    FoxyPirates -> "Foxy Pirates",
    FrostMoonVillage -> "Frost Moon Village",
    Germa66 -> "GERMA 66",
    GalleyLaCompany -> "Galley-La Company",
    GaspardePirates -> "Gasparde Pirates",
    Giant -> "Giant",
    GoaKingdom -> "Goa Kingdom",
    GoldenLionPirates -> "Golden Lion Pirates",
    Grantesoro -> "Grantesoro",
    GyroPirates -> "Gyro Pirates",
    HapposuiArmy -> "Happosui Army",
    HawkinsPirates -> "Hawkins Pirates",
    HeartPirates -> "Heart Pirates",
    Homies -> "Homies",
    ImpelDown -> "Impel Down",
    JailerBeast -> "Jailer Beast",
    JellyfishPirates -> "Jellyfish Pirates",
    Journalist -> "Journalist",
    KidPirates -> "Kid Pirates",
    KingdomOfGerma -> "Kingdom of GERMA",
    KingdomOfprodence -> "Kingdom of Prodence",
    KouzukiClan -> "Kouzuki Clan",
    KriegPirates -> "Krieg Pirates",
    KujaPirates -> "Kuja Pirates",
    KurozumiClan -> "Kurozumi Clan",
    LandOfWano -> "Land of Wano",
    LongRingLongLand -> "Long Ring Long Land",
    LuluciaKingdom -> "Lulucia Kingdom",
    MaryGeoise -> "Mary Geoise",
    MechaIsland -> "Mecha Island",
    Merfolk -> "Merfolk",
    Minks -> "Minks",
    MonkeyMountainAlliance -> "Monkey Mountain Alliance",
    MountainBandits -> "Mountain Bandits",
    MuggyKingdom -> "Muggy Kingdom",
    MugiwaraChase -> "Mugiwara Chase",
    Music -> "Music" "音楽",
    Navy -> "Navy",
    NeoNavy -> "Neo Navy",
    NewFishManPirates -> "New Fish-Man Pirates",
    NewGiantPirates -> "New Giant Pirates",
    Odyssey -> "ODYSSEY",
    OmatsuriIsland -> "Omatsuri Island",
    OnAirPirates -> "On-Air Pirates",
    Plague -> "Plague",
    PunkHazard -> "Punk Hazard",
    RedHairedPirates -> "Red-Haired Pirates",
    RevolutionaryArmy -> "Revolutionary Army",
    RumbarPirates -> "Rumbar Pirates",
    Smile -> "SMILE" "Smile",
    Scientist -> "Scientist",
    ShandianWarrior -> "Shandian Warrior",
    ShipbuildingTown -> "Shipbuilding Town",
    SkyIsland -> "Sky Island",
    SniperIsland -> "Sniper Island",
    SpadePirates -> "Spade Pirates",
    StrawHatCrew -> "Straw Hat Crew",
    Supernovas -> "Supernovas",
    TheAkazyaNine -> "The Akazaya Nine",
    TheFlyingFishRiders -> "The Flying Fish Riders",
    TheFourEmperors -> "The Four Emperors",
    TheFrankyFamily -> "The Franky Family",
    TheHouseOfLambs -> "The House of Lambs",
    TheMoon -> "The Moon",
    ThePiratesFest -> "The Pirates Fest",
    TheSevenWarlordsOfTheSea -> "The Seven Warlords of the Sea",
    TheSunPirates -> "The Sun Pirates",
    TheTontattas -> "The Tontattas",
    TheVinsmokeFamily -> "The Vinsmoke Family",
    ThrillerBarkPirates -> "Thriller Bark Pirates",
    TrumpPirates -> "Trump Pirates",
    Vassals -> "Vassals",
    WaterSeven -> "Water Seven",
    WeevilsMother -> "Weevil's Mother",
    WhitebeardPirates -> "Whitebeard Pirates",
    WhitebeardPiratesAllies -> "Whitebeard Pirates Allies",
    WholeCakeIsland -> "Whole Cake Island",
    WindmillVillage -> "Windmill Village",
    WorldGovernment -> "World Government",
    WorldPirates -> "World Pirates",
    YontaMariaFleet -> "Yonta Maria Fleet"
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum Attribute {
    Ranged,
    Slash,
    Special,
    Strike,
    Wisdom,
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Error, Debug)]
#[error("Invalid attribute '{0}'")]
pub struct ParseAttributeError(String);

impl FromStr for Attribute {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ranged" => Ok(Self::Ranged),
            "Slash" => Ok(Self::Slash),
            "Special" => Ok(Self::Special),
            "Strike" => Ok(Self::Strike),
            "Wisdom" => Ok(Self::Wisdom),
            other => Err(ParseAttributeError(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CardData {
    pub id: CardId,
    pub rarity: Rarity,
    pub ty: CardType,
    pub name: String,
    pub image_url: String,
    pub image_name: String,
    pub cost_life: usize,
    pub power: Option<usize>,
    pub counter: Option<usize>,
    pub color: Vec<Color>,
    pub effect: Option<String>,
    pub trigger: Option<String>,
    pub subtype: Vec<Subtype>,
    pub attribute: Vec<Attribute>,
}
