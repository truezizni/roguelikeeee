use std::collections::VecDeque;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::entity::{Item, ItemKind, Monster, MonsterKind, Player};
use crate::map::{Level, Map, Pos, Tile, MAP_H, MAP_W};

pub const FOV_RADIUS: i32 = 8;
const MAX_MESSAGES: usize = 5;

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Move(i32, i32),
    Wait,
    Pickup,
    Quaff,
    Descend,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Playing,
    Dead,
}

pub struct Game {
    pub map: Map,
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub items: Vec<Item>,
    pub visible: Vec<bool>,
    pub messages: VecDeque<String>,
    pub depth: u32,
    pub turn: u64,
    pub phase: Phase,
    pub seed: u64,
    pub stairs: Pos,
    rng: StdRng,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut game = Self {
            map: Map::new(MAP_W, MAP_H),
            player: Player::new(Pos::new(0, 0)),
            monsters: Vec::new(),
            items: Vec::new(),
            visible: vec![false; MAP_W * MAP_H],
            messages: VecDeque::new(),
            depth: 1,
            turn: 0,
            phase: Phase::Playing,
            seed,
            stairs: Pos::new(0, 0),
            rng: StdRng::seed_from_u64(seed),
        };
        game.build_level();
        game.log("Ты входишь в подземелье. Найди '>' и спускайся глубже.");
        game
    }

    fn build_level(&mut self) {
        let level_seed = self
            .seed
            .wrapping_add((self.depth as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ 0xA5A5_5A5A_C3C3_3C3C;
        let mut rng = StdRng::seed_from_u64(level_seed);
        let Level {
            map,
            player_start,
            stairs,
            spawn_points,
        } = Map::generate(&mut rng, self.depth);

        self.map = map;
        self.player.pos = player_start;
        self.stairs = stairs;
        self.monsters.clear();
        self.items.clear();

        let mut points = spawn_points;
        for i in (1..points.len()).rev() {
            let j = rng.gen_range(0..=i);
            points.swap(i, j);
        }

        let monster_count = (3 + self.depth as usize * 2).min(points.len() / 2).min(14);
        for _ in 0..monster_count {
            if let Some(pos) = points.pop() {
                let kind = roll_monster(self.depth, &mut rng);
                self.monsters.push(Monster::new(kind, pos));
            }
        }

        let potion_count = if rng.gen_bool(0.4) { 3 } else { 2 };
        for _ in 0..potion_count {
            if let Some(pos) = points.pop() {
                self.items.push(Item {
                    pos,
                    kind: ItemKind::Potion,
                });
            }
        }

        if self.depth.is_multiple_of(2) {
            if let Some(pos) = points.pop() {
                self.items.push(Item {
                    pos,
                    kind: ItemKind::Sword,
                });
            }
        }

        for _ in 0..rng.gen_range(2..=4) {
            if let Some(pos) = points.pop() {
                self.items.push(Item {
                    pos,
                    kind: ItemKind::Gold,
                });
            }
        }

        self.recompute_fov();
    }

    pub fn recompute_fov(&mut self) {
        self.visible = self.map.compute_fov(self.player.pos, FOV_RADIUS);
        self.map.mark_explored(&self.visible);
    }

    fn log(&mut self, message: impl Into<String>) {
        self.messages.push_back(message.into());
        while self.messages.len() > MAX_MESSAGES {
            self.messages.pop_front();
        }
    }

    pub fn player_action(&mut self, action: Action) {
        if self.phase != Phase::Playing {
            return;
        }

        let acted = match action {
            Action::Move(dx, dy) => self.try_move(dx, dy),
            Action::Wait => {
                self.log("Ты ждёшь.");
                true
            }
            Action::Pickup => self.pickup(),
            Action::Quaff => self.quaff(),
            Action::Descend => self.descend(),
        };
        if !acted {
            return;
        }

        if !matches!(action, Action::Descend) {
            self.monsters_act();
            if self.player.hp <= 0 {
                self.player.hp = 0;
                self.phase = Phase::Dead;
                self.log("Ты погиб. R — начать заново.");
            }
        }

        self.turn += 1;
        self.recompute_fov();
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let Some(target) = self.map.neighbor(self.player.pos, dx, dy) else {
            return false;
        };

        if let Some(index) = self.monster_index_at(target) {
            let damage = self.player.attack + self.rng.gen_range(0..=2);
            let name = self.monsters[index].kind.name();
            self.monsters[index].hp -= damage;
            self.monsters[index].awake = true;
            let dead = self.monsters[index].hp <= 0;
            self.log(format!("Ты бьёшь {name}: -{damage} HP."));
            if dead {
                self.monsters.remove(index);
                self.player.kills += 1;
                self.log(format!("{name} повержен!"));
            }
            return true;
        }

        if self.map.is_walkable(target) {
            self.player.pos = target;
            self.pickup_gold_here();
            return true;
        }

        self.log("Стена.");
        false
    }

    fn pickup_gold_here(&mut self) {
        if let Some(index) = self
            .items
            .iter()
            .position(|item| item.pos == self.player.pos && item.kind == ItemKind::Gold)
        {
            self.items.remove(index);
            let amount = self.rng.gen_range(5..=15);
            self.player.gold += amount;
            self.log(format!("Ты подобрал {amount} золота."));
        }
    }

    fn pickup(&mut self) -> bool {
        let Some(index) = self
            .items
            .iter()
            .position(|item| item.pos == self.player.pos)
        else {
            self.log("Здесь нечего подбирать.");
            return false;
        };

        let item = self.items.remove(index);
        match item.kind {
            ItemKind::Potion => {
                self.player.potions += 1;
                self.log("Подобрано зелье. Выпить — q.");
            }
            ItemKind::Sword => {
                self.player.attack += 1;
                self.log("Подобран меч: атака +1.");
            }
            ItemKind::Gold => {
                let amount = self.rng.gen_range(5..=15);
                self.player.gold += amount;
                self.log(format!("Подобрано {amount} золота."));
            }
        }
        true
    }

    fn quaff(&mut self) -> bool {
        if self.player.potions == 0 {
            self.log("Зелий нет.");
            return false;
        }
        self.player.potions -= 1;
        let healed = self.player.heal(8);
        self.log(format!("Зелье восстановило {healed} HP."));
        true
    }

    fn descend(&mut self) -> bool {
        if self.map.get(self.player.pos) != Tile::StairsDown {
            self.log("Здесь нет лестницы вниз. Ищи '>'.");
            return false;
        }
        self.depth += 1;
        self.build_level();
        self.log(format!("Глубина {}. Ты спускаешься ниже...", self.depth));
        true
    }

    fn monsters_act(&mut self) {
        for index in 0..self.monsters.len() {
            if !self.monsters[index].is_alive() {
                continue;
            }

            let pos = self.monsters[index].pos;
            let sees_player = pos.chebyshev(self.player.pos) <= 10
                && self.map.line_of_sight(pos, self.player.pos);
            if sees_player {
                self.monsters[index].awake = true;
            }

            if !self.monsters[index].awake {
                if self.rng.gen_bool(0.25) {
                    self.wander(index);
                }
                continue;
            }

            if pos.chebyshev(self.player.pos) <= 1 {
                let damage = self.monsters[index].attack + self.rng.gen_range(0..=1);
                let name = self.monsters[index].kind.name();
                self.player.hp -= damage;
                self.log(format!("{name} бьёт тебя: -{damage} HP."));
                if self.player.hp <= 0 {
                    break;
                }
            } else {
                self.move_monster_towards(index);
            }
        }
    }

    fn move_monster_towards(&mut self, index: usize) {
        let from = self.monsters[index].pos;
        let target = self.player.pos;
        let dx = (target.x as i32 - from.x as i32).signum();
        let dy = (target.y as i32 - from.y as i32).signum();

        for (step_x, step_y) in [(dx, dy), (dx, 0), (0, dy)] {
            if step_x == 0 && step_y == 0 {
                continue;
            }
            let Some(next) = self.map.neighbor(from, step_x, step_y) else {
                continue;
            };
            if self.is_free_for_monster(next, index) {
                self.monsters[index].pos = next;
                return;
            }
        }
    }

    fn wander(&mut self, index: usize) {
        let from = self.monsters[index].pos;
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (dx, dy) = directions[self.rng.gen_range(0..directions.len())];
        if let Some(next) = self.map.neighbor(from, dx, dy) {
            if self.is_free_for_monster(next, index) {
                self.monsters[index].pos = next;
            }
        }
    }

    fn is_free_for_monster(&self, pos: Pos, self_index: usize) -> bool {
        if !self.map.is_walkable(pos) || pos == self.player.pos {
            return false;
        }
        !self
            .monsters
            .iter()
            .enumerate()
            .any(|(index, monster)| index != self_index && monster.is_alive() && monster.pos == pos)
    }

    fn monster_index_at(&self, pos: Pos) -> Option<usize> {
        self.monsters
            .iter()
            .position(|monster| monster.is_alive() && monster.pos == pos)
    }
}

fn roll_monster(depth: u32, rng: &mut impl Rng) -> MonsterKind {
    match depth {
        1..=2 => {
            if rng.gen_bool(0.7) {
                MonsterKind::Rat
            } else {
                MonsterKind::Goblin
            }
        }
        3..=4 => {
            if rng.gen_bool(0.5) {
                MonsterKind::Goblin
            } else {
                MonsterKind::Orc
            }
        }
        5..=6 => {
            if rng.gen_bool(0.5) {
                MonsterKind::Orc
            } else {
                MonsterKind::Troll
            }
        }
        _ => {
            if rng.gen_bool(0.3) {
                MonsterKind::Orc
            } else {
                MonsterKind::Troll
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_neighbor(game: &Game) -> Pos {
        let pos = game.player.pos;
        [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .find_map(|(dx, dy)| {
                game.map
                    .neighbor(pos, *dx, *dy)
                    .filter(|next| game.map.is_walkable(*next))
            })
            .expect("у стартовой комнаты должен быть свободный сосед")
    }

    #[test]
    fn new_game_is_playable() {
        for seed in 0..10 {
            let game = Game::new(seed);
            assert_eq!(game.phase, Phase::Playing);
            assert!(game.map.is_walkable(game.player.pos));
            assert!(!game.monsters.is_empty(), "seed {seed}: нет монстров");
            assert!(!game.items.is_empty(), "seed {seed}: нет предметов");
            assert!(game.visible[game.map.idx(game.player.pos)]);
            assert_eq!(game.player.hp, game.player.max_hp);
        }
    }

    #[test]
    fn quaff_heals_and_consumes_potion() {
        let mut game = Game::new(11);
        game.monsters.clear();
        game.player.hp = 5;
        game.player.potions = 1;

        game.player_action(Action::Quaff);

        assert_eq!(game.player.hp, 13);
        assert_eq!(game.player.potions, 0);
    }

    #[test]
    fn bumping_a_monster_attacks_it() {
        let mut game = Game::new(7);
        game.monsters.clear();
        game.player.attack = 5;
        let player_pos = game.player.pos;
        let target = open_neighbor(&game);

        let mut rat = Monster::new(MonsterKind::Rat, target);
        rat.hp = 1;
        game.monsters.push(rat);

        game.player_action(Action::Move(
            target.x as i32 - player_pos.x as i32,
            target.y as i32 - player_pos.y as i32,
        ));

        assert!(game.monsters.is_empty());
        assert_eq!(game.player.kills, 1);
        assert_eq!(game.player.pos, player_pos);
    }

    #[test]
    fn stairs_descend_to_a_new_level() {
        let mut game = Game::new(3);
        game.player.pos = game.stairs;

        game.player_action(Action::Descend);

        assert_eq!(game.depth, 2);
        assert_eq!(game.phase, Phase::Playing);
        assert!(game.map.is_walkable(game.player.pos));
        assert!(!game.monsters.is_empty());
    }

    #[test]
    fn adjacent_monster_can_kill_the_player() {
        let mut game = Game::new(5);
        game.monsters.clear();
        let target = open_neighbor(&game);

        let mut orc = Monster::new(MonsterKind::Orc, target);
        orc.awake = true;
        game.monsters.push(orc);
        game.player.hp = 1;

        for _ in 0..10 {
            if game.phase == Phase::Dead {
                break;
            }
            game.player_action(Action::Wait);
        }

        assert_eq!(game.phase, Phase::Dead);
        assert_eq!(game.player.hp, 0);
    }

    #[test]
    fn sword_increases_attack_on_pickup() {
        let mut game = Game::new(9);
        game.monsters.clear();
        game.items.clear();
        let pos = game.player.pos;
        game.items.push(Item {
            pos,
            kind: ItemKind::Sword,
        });

        game.player_action(Action::Pickup);

        assert_eq!(game.player.attack, 3);
        assert!(game.items.is_empty());
    }

    #[test]
    fn random_play_keeps_state_valid() {
        use std::collections::HashSet;

        let mut game = Game::new(2024);
        let mut rng = StdRng::seed_from_u64(99);

        for _ in 0..800 {
            if game.phase == Phase::Dead {
                break;
            }

            let action = match rng.gen_range(0..9) {
                0 => Action::Move(0, -1),
                1 => Action::Move(0, 1),
                2 => Action::Move(-1, 0),
                3 => Action::Move(1, 0),
                4 => Action::Wait,
                5 => Action::Pickup,
                6 => Action::Quaff,
                7 => Action::Descend,
                _ => Action::Move(1, 1),
            };
            game.player_action(action);

            assert!(game.map.is_walkable(game.player.pos));
            assert!((0..=game.player.max_hp).contains(&game.player.hp));

            let mut occupied = HashSet::new();
            for monster in &game.monsters {
                assert!(monster.is_alive());
                assert!(game.map.is_walkable(monster.pos));
                assert_ne!(monster.pos, game.player.pos);
                assert!(
                    occupied.insert(monster.pos),
                    "монстры стоят на одной клетке"
                );
            }

            for item in &game.items {
                assert!(game.map.is_walkable(item.pos));
            }
        }
    }
}
