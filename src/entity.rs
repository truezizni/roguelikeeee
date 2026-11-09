use crate::map::Pos;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MonsterKind {
    Rat,
    Goblin,
    Orc,
    Troll,
}

impl MonsterKind {
    pub fn name(self) -> &'static str {
        match self {
            MonsterKind::Rat => "крыса",
            MonsterKind::Goblin => "гоблин",
            MonsterKind::Orc => "орк",
            MonsterKind::Troll => "тролль",
        }
    }

    pub fn glyph(self) -> char {
        match self {
            MonsterKind::Rat => 'r',
            MonsterKind::Goblin => 'g',
            MonsterKind::Orc => 'o',
            MonsterKind::Troll => 'T',
        }
    }

    pub fn color(self) -> (f32, f32, f32) {
        match self {
            MonsterKind::Rat => (0.70, 0.66, 0.60),
            MonsterKind::Goblin => (0.40, 0.80, 0.35),
            MonsterKind::Orc => (0.92, 0.55, 0.25),
            MonsterKind::Troll => (0.88, 0.30, 0.30),
        }
    }

    pub fn base_hp(self) -> i32 {
        match self {
            MonsterKind::Rat => 2,
            MonsterKind::Goblin => 4,
            MonsterKind::Orc => 7,
            MonsterKind::Troll => 12,
        }
    }

    pub fn base_attack(self) -> i32 {
        match self {
            MonsterKind::Rat => 1,
            MonsterKind::Goblin => 2,
            MonsterKind::Orc => 3,
            MonsterKind::Troll => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Monster {
    pub pos: Pos,
    pub kind: MonsterKind,
    pub hp: i32,
    pub attack: i32,
    pub awake: bool,
}

impl Monster {
    pub fn new(kind: MonsterKind, pos: Pos) -> Self {
        Self {
            pos,
            kind,
            hp: kind.base_hp(),
            attack: kind.base_attack(),
            awake: false,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: Pos,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub potions: u32,
    pub gold: u32,
    pub kills: u32,
}

impl Player {
    pub fn new(pos: Pos) -> Self {
        Self {
            pos,
            hp: 20,
            max_hp: 20,
            attack: 2,
            potions: 1,
            gold: 0,
            kills: 0,
        }
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let before = self.hp;
        self.hp = (self.hp + amount).min(self.max_hp);
        self.hp - before
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Potion,
    Sword,
    Gold,
}

impl ItemKind {
    pub fn glyph(self) -> char {
        match self {
            ItemKind::Potion => '!',
            ItemKind::Sword => '/',
            ItemKind::Gold => '$',
        }
    }

    pub fn color(self) -> (f32, f32, f32) {
        match self {
            ItemKind::Potion => (0.90, 0.40, 0.90),
            ItemKind::Sword => (0.50, 0.80, 0.95),
            ItemKind::Gold => (1.00, 0.80, 0.20),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub pos: Pos,
    pub kind: ItemKind,
}
