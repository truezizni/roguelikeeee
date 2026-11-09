use macroquad::prelude::*;

use crate::game::{Game, Phase};
use crate::map::{Pos, Tile, MAP_H, MAP_W};

pub const TILE: f32 = 20.0;
pub const MAP_X: f32 = 10.0;
pub const MAP_Y: f32 = 10.0;
pub const WINDOW_W: i32 = (MAP_W as f32 * TILE) as i32 + 20;
pub const WINDOW_H: i32 = (MAP_H as f32 * TILE) as i32 + 190;

const BG: Color = Color {
    r: 0.05,
    g: 0.05,
    b: 0.07,
    a: 1.0,
};
const HUD_BG: Color = Color {
    r: 0.02,
    g: 0.02,
    b: 0.03,
    a: 1.0,
};
const WALL_VISIBLE: Color = Color {
    r: 0.55,
    g: 0.50,
    b: 0.44,
    a: 1.0,
};
const WALL_DIM: Color = Color {
    r: 0.17,
    g: 0.16,
    b: 0.15,
    a: 1.0,
};
const FLOOR_VISIBLE: Color = Color {
    r: 0.32,
    g: 0.32,
    b: 0.36,
    a: 1.0,
};
const FLOOR_DIM: Color = Color {
    r: 0.10,
    g: 0.10,
    b: 0.12,
    a: 1.0,
};
const STAIRS_COLOR: Color = Color {
    r: 1.0,
    g: 0.95,
    b: 0.55,
    a: 1.0,
};
const PLAYER_COLOR: Color = Color {
    r: 1.0,
    g: 0.86,
    b: 0.25,
    a: 1.0,
};
const HP_GOOD: Color = Color {
    r: 0.30,
    g: 0.80,
    b: 0.35,
    a: 1.0,
};
const HP_WARN: Color = Color {
    r: 0.95,
    g: 0.75,
    b: 0.20,
    a: 1.0,
};
const HP_BAD: Color = Color {
    r: 0.90,
    g: 0.25,
    b: 0.25,
    a: 1.0,
};
const HP_BG: Color = Color {
    r: 0.25,
    g: 0.10,
    b: 0.10,
    a: 1.0,
};
const TEXT: Color = Color {
    r: 0.85,
    g: 0.85,
    b: 0.82,
    a: 1.0,
};
const TEXT_DIM: Color = Color {
    r: 0.55,
    g: 0.55,
    b: 0.55,
    a: 1.0,
};
const OVERLAY: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.68,
};

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Roguelikeeee".to_owned(),
        window_width: WINDOW_W,
        window_height: WINDOW_H,
        window_resizable: false,
        ..Default::default()
    }
}

pub fn draw(game: &Game, font: &Font) {
    clear_background(BG);
    draw_map(game, font);
    draw_entities(game, font);
    draw_hud(game, font);

    if game.phase == Phase::Dead {
        draw_death_screen(game, font);
    }
}

fn draw_map(game: &Game, font: &Font) {
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let pos = Pos::new(x, y);
            let index = game.map.idx(pos);
            if !game.map.explored[index] {
                continue;
            }

            let visible = game.visible[index];
            let tile = game.map.get(pos);
            let color = match tile {
                Tile::Wall => {
                    if visible {
                        WALL_VISIBLE
                    } else {
                        WALL_DIM
                    }
                }
                Tile::Floor => {
                    if visible {
                        FLOOR_VISIBLE
                    } else {
                        FLOOR_DIM
                    }
                }
                Tile::StairsDown => {
                    if visible {
                        STAIRS_COLOR
                    } else {
                        FLOOR_DIM
                    }
                }
            };

            draw_glyph(font, tile.glyph(), tile_x(x), tile_y(y), 18.0, color);
        }
    }
}

fn draw_entities(game: &Game, font: &Font) {
    for item in &game.items {
        if !game.visible[game.map.idx(item.pos)] {
            continue;
        }
        let (r, g, b) = item.kind.color();
        draw_glyph(
            font,
            item.kind.glyph(),
            tile_x(item.pos.x),
            tile_y(item.pos.y),
            18.0,
            Color { r, g, b, a: 1.0 },
        );
    }

    for monster in &game.monsters {
        if !monster.is_alive() || !game.visible[game.map.idx(monster.pos)] {
            continue;
        }
        let (r, g, b) = monster.kind.color();
        draw_glyph(
            font,
            monster.kind.glyph(),
            tile_x(monster.pos.x),
            tile_y(monster.pos.y),
            18.0,
            Color { r, g, b, a: 1.0 },
        );
    }

    draw_glyph(
        font,
        '@',
        tile_x(game.player.pos.x),
        tile_y(game.player.pos.y),
        20.0,
        PLAYER_COLOR,
    );
}

fn draw_hud(game: &Game, font: &Font) {
    let hud_top = MAP_Y + MAP_H as f32 * TILE + 6.0;
    draw_rectangle(
        0.0,
        hud_top,
        WINDOW_W as f32,
        WINDOW_H as f32 - hud_top,
        HUD_BG,
    );
    draw_line(0.0, hud_top, WINDOW_W as f32, hud_top, 1.0, TEXT_DIM);

    let ratio = (game.player.hp as f32 / game.player.max_hp as f32).clamp(0.0, 1.0);
    draw_rectangle(MAP_X, hud_top + 8.0, 160.0, 16.0, HP_BG);
    let hp_color = if ratio > 0.5 {
        HP_GOOD
    } else if ratio > 0.25 {
        HP_WARN
    } else {
        HP_BAD
    };
    draw_rectangle(MAP_X, hud_top + 8.0, 160.0 * ratio, 16.0, hp_color);

    let stats = format!(
        "HP {}/{}   Глубина {}   Атака {}   Зелья {}   Золото {}   Убито {}",
        game.player.hp,
        game.player.max_hp,
        game.depth,
        game.player.attack,
        game.player.potions,
        game.player.gold,
        game.player.kills,
    );
    draw_text_ex(
        &stats,
        190.0,
        hud_top + 22.0,
        TextParams {
            font: Some(font),
            font_size: 18,
            color: TEXT,
            ..Default::default()
        },
    );

    let messages: Vec<&String> = game.messages.iter().rev().take(4).collect();
    for (index, message) in messages.iter().rev().enumerate() {
        draw_text_ex(
            message,
            MAP_X,
            hud_top + 52.0 + index as f32 * 20.0,
            TextParams {
                font: Some(font),
                font_size: 16,
                color: TEXT_DIM,
                ..Default::default()
            },
        );
    }

    let controls = "[WASD/стрелки] ход   [.] ждать   [g] поднять   [q] зелье   [Enter/>] вниз   [R] заново   [Esc] выход";
    draw_text_ex(
        controls,
        MAP_X,
        WINDOW_H as f32 - 10.0,
        TextParams {
            font: Some(font),
            font_size: 14,
            color: TEXT_DIM,
            ..Default::default()
        },
    );
}

fn draw_death_screen(game: &Game, font: &Font) {
    draw_rectangle(
        MAP_X,
        MAP_Y,
        MAP_W as f32 * TILE,
        MAP_H as f32 * TILE,
        OVERLAY,
    );
    draw_centered(font, "ТЫ ПОГИБ", MAP_Y + 220.0, 52, HP_BAD);
    let stats = format!(
        "Глубина: {}    Золото: {}    Убито: {}",
        game.depth, game.player.gold, game.player.kills
    );
    draw_centered(font, &stats, MAP_Y + 280.0, 24, TEXT);
    draw_centered(
        font,
        "Нажми R, чтобы начать заново",
        MAP_Y + 330.0,
        22,
        PLAYER_COLOR,
    );
}

fn draw_centered(font: &Font, text: &str, y: f32, size: u16, color: Color) {
    let dims = measure_text(text, Some(font), size, 1.0);
    let x = (WINDOW_W as f32 - dims.width) / 2.0;
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size: size,
            color,
            ..Default::default()
        },
    );
}

fn draw_glyph(font: &Font, glyph: char, x: f32, y: f32, size: f32, color: Color) {
    let mut buffer = [0u8; 4];
    let text = glyph.encode_utf8(&mut buffer);
    let advance = measure_text("M", Some(font), size as u16, 1.0).width;
    let tx = x + (TILE - advance) / 2.0;
    let ty = y + TILE - 4.0;
    draw_text_ex(
        text,
        tx,
        ty,
        TextParams {
            font: Some(font),
            font_size: size as u16,
            color,
            ..Default::default()
        },
    );
}

fn tile_x(x: usize) -> f32 {
    MAP_X + x as f32 * TILE
}

fn tile_y(y: usize) -> f32 {
    MAP_Y + y as f32 * TILE
}
