use std::{
    any::type_name,
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use common::{
    card::{
        Attribute, Card, CardDescription, CardDescriptionPart, CardLimit, CardPassword, CardType,
        LinkMarkers, MonsterEffect, MonsterStats, MonsterType, Race, SpellType, TrapType,
    },
    deck::DeckEntry,
};
use console::{style, Style};

const PREV_DATA_PATH: &str = "target/sizes_prev.bin";
const BASE_DATA_PATH: &str = "target/sizes_base.bin";

type SizeMap = HashMap<String, usize>;

struct SizeDataManager {
    base: Option<SizeMap>,
    prev: Option<SizeMap>,
    current: SizeMap,
}

impl SizeDataManager {
    fn load() -> Self {
        let mut base = None;
        if Path::new(BASE_DATA_PATH).try_exists().unwrap() {
            let file = BufReader::new(File::open(BASE_DATA_PATH).unwrap());
            base = Some(bincode::deserialize_from(file).unwrap());
        }

        let mut prev = None;
        if Path::new(PREV_DATA_PATH).try_exists().unwrap() {
            let file = BufReader::new(File::open(PREV_DATA_PATH).unwrap());
            prev = Some(bincode::deserialize_from(file).unwrap());
        }

        Self {
            base,
            prev,
            current: SizeMap::default(),
        }
    }

    fn check<T>(&mut self, name: impl Into<String>) -> SizeChecker {
        let name = name.into();
        let size = size_of::<T>();

        print!("{name:20} {:>3}", style(size).bold());
        Self::print_diff("prev", &self.prev, &name, size);
        Self::print_diff("base", &self.base, &name, size);
        println!();

        self.current.insert(name, size);

        SizeChecker::new(size)
    }

    fn print_diff(kind: &'static str, map: &Option<SizeMap>, name: &String, size: usize) {
        if let Some(&old) = map.as_ref().and_then(|map| map.get(name)) {
            let diff = isize::try_from(size).unwrap() - isize::try_from(old).unwrap();

            let mut style = Style::new().bold();
            match diff.cmp(&0) {
                Ordering::Less => style = style.green(),
                Ordering::Greater => style = style.red(),
                Ordering::Equal => {}
            }

            let diff = style.apply_to(diff);
            print!("; {kind}: {diff:+}");
        }
    }
}

impl Drop for SizeDataManager {
    fn drop(&mut self) {
        let file = BufWriter::new(File::create(PREV_DATA_PATH).unwrap());
        bincode::serialize_into(file, &self.current).unwrap();

        let repo = gix::open(".").unwrap();
        if !repo.is_dirty().unwrap() {
            let file = BufWriter::new(File::create(BASE_DATA_PATH).unwrap());
            bincode::serialize_into(file, &self.current).unwrap();
        }
    }
}

struct SizeChecker {
    size: usize,
    content: Option<usize>,
}

impl SizeChecker {
    fn new(size: usize) -> Self {
        Self {
            size,
            content: None,
        }
    }

    fn field<T>(mut self, name: &'static str) -> Self {
        let size = size_of::<T>();
        println!(
            "  {name:18} {:3} {}",
            style(size).bold(),
            style(type_name::<T>()).dim()
        );

        *self.content.get_or_insert(0) += size;
        self
    }

    fn variant<T>(self, name: &'static str) -> Self {
        let size = size_of::<T>();

        println!(
            "  {:18} {:3} ({:5.3}) {}",
            name,
            style(size).bold(),
            self.relative(size),
            style(type_name::<T>()).dim()
        );

        self
    }

    fn relative(&self, size: usize) -> impl std::fmt::Display {
        #[allow(clippy::cast_precision_loss)]
        let relative = size as f64 / self.size as f64;
        let mut relative_style = Style::new().bold();

        if relative >= 0.99 {
            relative_style = relative_style.green();
        } else if relative >= 0.9 {
            relative_style = relative_style.yellow();
        } else {
            relative_style = relative_style.red();
        }

        relative_style.apply_to(relative)
    }
}

impl Drop for SizeChecker {
    fn drop(&mut self) {
        if let Some(size) = self.content {
            println!(
                "  {:18} {:3} ({:5.3})",
                "Total",
                style(size).bold(),
                self.relative(size)
            );
        }

        println!();
    }
}

// Placeholders for enum variants

struct MonsterData {
    _race: Race,
    _attribute: Attribute,
    _stats: MonsterStats,
    _effect: MonsterEffect,
    _is_tuner: bool,
}

struct NormalStats {
    _atk: u16,
    _def: u16,
    _level: u8,
    _monster_type: Option<MonsterType>,
    _pendulum_scale: Option<u8>,
}

struct LinkStats {
    _atk: u16,
    _link_value: u8,
    _link_markers: LinkMarkers,
}

fn main() {
    let mut manager = SizeDataManager::load();

    manager
        .check::<DeckEntry>("DeckEntry")
        .field::<u16>("id")
        .field::<u8>("playing count")
        .field::<u8>("side count");

    manager
        .check::<Card>("Card")
        .field::<String>("name")
        .field::<Vec<CardPassword>>("passwords")
        .field::<CardDescription>("description")
        .field::<String>("search_text")
        .field::<CardType>("card_type")
        .field::<CardLimit>("limit")
        .field::<Option<String>>("archetype");

    manager
        .check::<CardDescription>("CardDescription")
        .variant::<Vec<CardDescriptionPart>>("Regular")
        .variant::<[Vec<CardDescriptionPart>; 2]>("Pendulum");

    manager
        .check::<CardDescriptionPart>("CardDescriptionPart")
        .variant::<String>("Paragraph")
        .variant::<Vec<String>>("List");

    manager
        .check::<CardType>("CardType")
        .variant::<MonsterData>("Monster")
        .variant::<SpellType>("Spell")
        .variant::<TrapType>("Trap");

    manager
        .check::<MonsterData>("MonsterData")
        .field::<Race>("race")
        .field::<Attribute>("attribute")
        .field::<MonsterStats>("stats")
        .field::<MonsterEffect>("effect")
        .field::<bool>("is tuner");

    manager
        .check::<MonsterStats>("MonsterStats")
        .variant::<NormalStats>("Normal")
        .variant::<LinkStats>("Link");

    manager
        .check::<NormalStats>("NormalStats")
        .field::<u16>("atk")
        .field::<u16>("def")
        .field::<u8>("level")
        .field::<Option<MonsterType>>("type")
        .field::<Option<u8>>("scale");

    manager
        .check::<LinkStats>("LinkStats")
        .field::<u16>("atk")
        .field::<u8>("link")
        .field::<LinkMarkers>("markers");
}
