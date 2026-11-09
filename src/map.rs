use rand::Rng;

pub const MAP_W: usize = 60;
pub const MAP_H: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Wall,
    Floor,
    StairsDown,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(self, Tile::Wall)
    }

    pub fn blocks_sight(self) -> bool {
        matches!(self, Tile::Wall)
    }

    pub fn glyph(self) -> char {
        match self {
            Tile::Wall => '#',
            Tile::Floor => '·',
            Tile::StairsDown => '>',
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

impl Pos {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn chebyshev(self, other: Pos) -> i32 {
        let dx = (self.x as i32 - other.x as i32).abs();
        let dy = (self.y as i32 - other.y as i32).abs();
        dx.max(dy)
    }

    pub fn distance_sq(self, other: Pos) -> i32 {
        let dx = self.x as i32 - other.x as i32;
        let dy = self.y as i32 - other.y as i32;
        dx * dx + dy * dy
    }
}

/// Все клетки на отрезке между `a` и `b` (включительно).
pub fn bresenham(a: Pos, b: Pos) -> Vec<Pos> {
    let mut points = Vec::new();
    let (mut x0, mut y0) = (a.x as i32, a.y as i32);
    let (x1, y1) = (b.x as i32, b.y as i32);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push(Pos::new(x0 as usize, y0 as usize));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    points
}

#[derive(Clone)]
pub struct Map {
    pub tiles: Vec<Tile>,
    pub explored: Vec<bool>,
    pub width: usize,
    pub height: usize,
}

pub struct Level {
    pub map: Map,
    pub player_start: Pos,
    pub stairs: Pos,
    pub spawn_points: Vec<Pos>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            tiles: vec![Tile::Wall; width * height],
            explored: vec![false; width * height],
            width,
            height,
        }
    }

    pub fn idx(&self, pos: Pos) -> usize {
        pos.y * self.width + pos.x
    }

    pub fn in_bounds(&self, pos: Pos) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    pub fn get(&self, pos: Pos) -> Tile {
        self.tiles[self.idx(pos)]
    }

    pub fn set(&mut self, pos: Pos, tile: Tile) {
        if self.in_bounds(pos) {
            let index = self.idx(pos);
            self.tiles[index] = tile;
        }
    }

    pub fn is_walkable(&self, pos: Pos) -> bool {
        self.in_bounds(pos) && self.get(pos).walkable()
    }

    pub fn neighbor(&self, pos: Pos, dx: i32, dy: i32) -> Option<Pos> {
        let x = pos.x as i32 + dx;
        let y = pos.y as i32 + dy;
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            None
        } else {
            Some(Pos::new(x as usize, y as usize))
        }
    }

    pub fn line_of_sight(&self, from: Pos, to: Pos) -> bool {
        let points = bresenham(from, to);
        if points.len() <= 2 {
            return true;
        }
        points[1..points.len() - 1]
            .iter()
            .all(|pos| !self.get(*pos).blocks_sight())
    }

    pub fn compute_fov(&self, origin: Pos, radius: i32) -> Vec<bool> {
        let mut visible = vec![false; self.tiles.len()];
        let radius_sq = radius * radius;
        let min_x = origin.x.saturating_sub(radius as usize);
        let max_x = (origin.x + radius as usize).min(self.width - 1);
        let min_y = origin.y.saturating_sub(radius as usize);
        let max_y = (origin.y + radius as usize).min(self.height - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let pos = Pos::new(x, y);
                if pos.distance_sq(origin) <= radius_sq && self.line_of_sight(origin, pos) {
                    visible[self.idx(pos)] = true;
                }
            }
        }

        if self.in_bounds(origin) {
            visible[self.idx(origin)] = true;
        }
        visible
    }

    pub fn mark_explored(&mut self, visible: &[bool]) {
        for (explored, seen) in self.explored.iter_mut().zip(visible.iter()) {
            if *seen {
                *explored = true;
            }
        }
    }

    fn carve_room(&mut self, room: Room) {
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                self.set(Pos::new(x, y), Tile::Floor);
            }
        }
    }

    fn carve_h(&mut self, x1: usize, x2: usize, y: usize) {
        let (lo, hi) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        for x in lo..=hi {
            self.set(Pos::new(x, y), Tile::Floor);
        }
    }

    fn carve_v(&mut self, y1: usize, y2: usize, x: usize) {
        let (lo, hi) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        for y in lo..=hi {
            self.set(Pos::new(x, y), Tile::Floor);
        }
    }

    fn carve_corridor(&mut self, a: Pos, b: Pos, rng: &mut impl Rng) {
        if rng.gen_bool(0.5) {
            self.carve_h(a.x, b.x, a.y);
            self.carve_v(a.y, b.y, b.x);
        } else {
            self.carve_v(a.y, b.y, a.x);
            self.carve_h(a.x, b.x, b.y);
        }
    }

    pub fn generate(rng: &mut impl Rng, depth: u32) -> Level {
        let mut map = Map::new(MAP_W, MAP_H);
        let mut rooms: Vec<Room> = Vec::new();

        let first = Room {
            x: rng.gen_range(1..=12),
            y: rng.gen_range(1..=8),
            w: rng.gen_range(7..=10),
            h: rng.gen_range(5..=7),
        };
        map.carve_room(first);
        rooms.push(first);

        let target_rooms = (8 + depth.min(6) as usize).min(14);
        for _ in 0..120 {
            if rooms.len() >= target_rooms {
                break;
            }
            let w = rng.gen_range(5..=10usize);
            let h = rng.gen_range(4..=7usize);
            if MAP_W < w + 2 || MAP_H < h + 2 {
                continue;
            }
            let x = rng.gen_range(1..=(MAP_W - w - 1));
            let y = rng.gen_range(1..=(MAP_H - h - 1));
            let room = Room { x, y, w, h };
            if rooms.iter().any(|other| room.overlaps(*other)) {
                continue;
            }
            map.carve_room(room);
            if let Some(previous) = rooms.last() {
                map.carve_corridor(previous.center(), room.center(), rng);
            }
            rooms.push(room);
        }

        // На карте всегда должно быть минимум две комнаты.
        if rooms.len() < 2 {
            let fallback = Room {
                x: MAP_W / 2 + 4,
                y: MAP_H / 2 + 4,
                w: 8,
                h: 6,
            };
            map.carve_room(fallback);
            if let Some(previous) = rooms.last() {
                map.carve_corridor(previous.center(), fallback.center(), rng);
            }
            rooms.push(fallback);
        }

        let player_start = rooms[0].center();
        let stairs = rooms[rooms.len() - 1].center();
        map.set(stairs, Tile::StairsDown);

        let mut spawn_points = Vec::new();
        for room in rooms.iter().skip(1) {
            for y in room.y..room.y + room.h {
                for x in room.x..room.x + room.w {
                    let pos = Pos::new(x, y);
                    if pos != stairs {
                        spawn_points.push(pos);
                    }
                }
            }
        }

        Level {
            map,
            player_start,
            stairs,
            spawn_points,
        }
    }
}

#[derive(Clone, Copy)]
struct Room {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Room {
    fn center(self) -> Pos {
        Pos::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    fn overlaps(self, other: Room) -> bool {
        self.x < other.x + other.w + 1
            && other.x < self.x + self.w + 1
            && self.y < other.y + other.h + 1
            && other.y < self.y + self.h + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn map_from_ascii(rows: &[&str]) -> Map {
        let height = rows.len();
        let width = rows[0].chars().count();
        let mut map = Map::new(width, height);
        for (y, row) in rows.iter().enumerate() {
            for (x, glyph) in row.chars().enumerate() {
                let tile = if glyph == '#' {
                    Tile::Wall
                } else {
                    Tile::Floor
                };
                map.set(Pos::new(x, y), tile);
            }
        }
        map
    }

    fn reachable(map: &Map, start: Pos) -> usize {
        let mut seen = vec![false; map.tiles.len()];
        let mut queue = std::collections::VecDeque::new();
        seen[map.idx(start)] = true;
        queue.push_back(start);
        let mut count = 0;

        while let Some(pos) = queue.pop_front() {
            count += 1;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(next) = map.neighbor(pos, dx, dy) {
                    if map.is_walkable(next) && !seen[map.idx(next)] {
                        seen[map.idx(next)] = true;
                        queue.push_back(next);
                    }
                }
            }
        }

        count
    }

    #[test]
    fn generated_levels_are_connected() {
        for seed in 0..25 {
            for depth in [1, 3, 7] {
                let mut rng = StdRng::seed_from_u64(seed);
                let level = Map::generate(&mut rng, depth);
                assert!(level.map.is_walkable(level.player_start));
                assert!(level.map.is_walkable(level.stairs));
                assert_eq!(level.map.get(level.stairs), Tile::StairsDown);

                let walkable = level
                    .map
                    .tiles
                    .iter()
                    .filter(|tile| tile.walkable())
                    .count();
                assert_eq!(
                    reachable(&level.map, level.player_start),
                    walkable,
                    "seed {seed}, depth {depth}: есть недостижимые клетки"
                );
            }
        }
    }

    #[test]
    fn fov_is_blocked_by_walls() {
        let map = map_from_ascii(&[
            "#####", //
            "#...#", //
            "#.#.#", //
            "#...#", //
            "#####", //
        ]);
        let origin = Pos::new(1, 1);
        let visible = map.compute_fov(origin, 10);
        let idx = |x, y| map.idx(Pos::new(x, y));

        assert!(visible[idx(1, 1)], "игрок всегда виден");
        assert!(visible[idx(3, 1)], "прямая видимость по коридору");
        assert!(!visible[idx(3, 3)], "диагональ перекрыта стеной");
        assert!(visible[idx(1, 3)], "вдоль стены видно");
    }

    #[test]
    fn line_of_sight_handles_endpoints() {
        let map = map_from_ascii(&["###", "#.#", "###"]);
        assert!(map.line_of_sight(Pos::new(1, 1), Pos::new(1, 1)));
        assert!(map.line_of_sight(Pos::new(1, 1), Pos::new(1, 0)));

        let walls = map_from_ascii(&["###", "###", "###"]);
        assert!(!walls.line_of_sight(Pos::new(0, 0), Pos::new(2, 2)));
    }
}
