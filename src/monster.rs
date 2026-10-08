#[derive(Debug)]
pub struct MonsterEntry {
    pub id: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Monster {
    pub name: String,
    pub size: String,
    pub monster_type: String,
    pub alignment: String,
    /// Usually one entry; a second one applies under a spell or condition.
    pub armor_class: Vec<ArmorClass>,
    pub hit_points: u32,
    /// The dice expression the hit points are rolled with, such as `18d10+36`.
    pub hit_points_roll: String,
    pub speed: Speed,
    pub ability_scores: AbilityScores,
    pub actions: Vec<Action>,
}

impl Monster {
    /// The initiative bonus, which is the Dexterity modifier.
    pub fn initiative(&self) -> i8 {
        AbilityScores::modifier(self.ability_scores.dexterity)
    }
}

#[derive(Debug, PartialEq)]
pub struct ArmorClass {
    pub value: u8,
    pub source: ArmorClassSource,
}

#[derive(Debug, PartialEq)]
pub enum ArmorClassSource {
    Dexterity,
    Natural,
    /// The names of the worn armor pieces.
    Armor(Vec<String>),
    /// The name of the spell that grants this armor class.
    Spell(String),
    /// The name of the condition under which this armor class applies.
    Condition(String),
    Other(String),
}

/// Movement speeds as given by the API, such as `30 ft.`.
#[derive(Debug, Default, PartialEq)]
pub struct Speed {
    pub walk: Option<String>,
    pub burrow: Option<String>,
    pub climb: Option<String>,
    pub fly: Option<String>,
    pub swim: Option<String>,
    pub hover: bool,
}

#[derive(Debug, PartialEq)]
pub struct AbilityScores {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl AbilityScores {
    /// The modifier for an ability score: one for every two points above 10, rounded down.
    pub fn modifier(score: u8) -> i8 {
        (i16::from(score) - 10).div_euclid(2) as i8
    }
}

#[derive(Debug, PartialEq)]
pub struct Action {
    pub name: String,
    pub description: String,
}
