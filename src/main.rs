mod entity;
mod game;
mod map;
mod ui;

use std::time::{SystemTime, UNIX_EPOCH};

use game::{Action, Game, Phase};
use macroquad::prelude::*;
use ui::window_conf;

fn new_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0xC0FF_EE00_1234_5678)
}

#[macroquad::main(window_conf)]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/DejaVuSansMono.ttf"))
        .expect("не удалось загрузить шрифт DejaVu Sans Mono");

    let mut game = Game::new(new_seed());

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if is_key_pressed(KeyCode::R) {
            game = Game::new(new_seed());
        } else if game.phase == Phase::Playing {
            if let Some(action) = read_action() {
                game.player_action(action);
            }
        }

        ui::draw(&game, &font);
        next_frame().await;
    }
}

fn read_action() -> Option<Action> {
    use KeyCode::*;

    let shift = is_key_down(LeftShift) || is_key_down(RightShift);

    if is_key_pressed(W) || is_key_pressed(Up) || is_key_pressed(K) {
        Some(Action::Move(0, -1))
    } else if is_key_pressed(S) || is_key_pressed(Down) || is_key_pressed(J) {
        Some(Action::Move(0, 1))
    } else if is_key_pressed(A) || is_key_pressed(Left) || is_key_pressed(H) {
        Some(Action::Move(-1, 0))
    } else if is_key_pressed(D) || is_key_pressed(Right) || is_key_pressed(L) {
        Some(Action::Move(1, 0))
    } else if is_key_pressed(Y) {
        Some(Action::Move(-1, -1))
    } else if is_key_pressed(U) {
        Some(Action::Move(1, -1))
    } else if is_key_pressed(B) {
        Some(Action::Move(-1, 1))
    } else if is_key_pressed(N) {
        Some(Action::Move(1, 1))
    } else if is_key_pressed(G) {
        Some(Action::Pickup)
    } else if is_key_pressed(Q) {
        Some(Action::Quaff)
    } else if is_key_pressed(Enter) {
        Some(Action::Descend)
    } else if is_key_pressed(Period) {
        if shift {
            Some(Action::Descend)
        } else {
            Some(Action::Wait)
        }
    } else if is_key_pressed(Space) {
        Some(Action::Wait)
    } else {
        None
    }
}
