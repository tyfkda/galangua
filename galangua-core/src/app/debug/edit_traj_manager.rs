use crate::app::game::enemy::FormationIndex;
use crate::app::game::game_manager::GameManager;
use crate::app::util::unsafe_util::peep;
use crate::util::math::ONE;
use crate::framework::{RendererTrait, VKey};
use crate::framework::types::Vec2I;

pub struct EditTrajManager {
    pub fi: FormationIndex,
}

impl EditTrajManager {
    pub fn new() -> Self {
        Self {
            fi: FormationIndex(0, 5),
        }
    }

    pub fn update(&mut self, pressed_key: Option<VKey>, game_manager: &mut GameManager) {
        if pressed_key == Some(VKey::Left) && self.fi.0 > 0 {
            self.fi.0 -= 1;
        }
        if pressed_key == Some(VKey::Right) && self.fi.0 < 9 {
            self.fi.0 += 1;
        }
        if pressed_key == Some(VKey::Up) && self.fi.1 > 0 {
            self.fi.1 -= 1;
        }
        if pressed_key == Some(VKey::Down) && self.fi.1 < 5 {
            self.fi.1 += 1;
        }

        if pressed_key == Some(VKey::Num1) {
            self.set_attack(game_manager, false);
        }
        if pressed_key == Some(VKey::Num2) {
            self.set_attack(game_manager, true);
        }
    }

    pub fn draw<R: RendererTrait>(&mut self, renderer: &mut R, game_manager: &mut GameManager) -> Result<(), String> {
        let enemy_manager = game_manager.enemy_manager_mut();
        let pos = &(&enemy_manager.get_formation_pos(&self.fi) / ONE) + &Vec2I::new(-8, -8);
        renderer.set_draw_color(255, 0, 255);
        renderer.fill_rect(Some([&pos, &Vec2I::new(16, 1)]))?;
        renderer.fill_rect(Some([&pos, &Vec2I::new(1, 16)]))?;
        renderer.fill_rect(Some([&(&pos + &Vec2I::new(0, 15)), &Vec2I::new(16, 1)]))?;
        renderer.fill_rect(Some([&(&pos + &Vec2I::new(15, 0)), &Vec2I::new(1, 16)]))?;

        renderer.set_texture_color_mod("font", 128, 128, 128);
        renderer.draw_str("font", 2 * 8, 0 * 8, "EDIT MODE")?;
        Ok(())
    }

    fn set_attack(&mut self, game_manager: &mut GameManager, capture_attack: bool) {
        let accessor = unsafe { peep(game_manager) };
        let enemy_manager = game_manager.enemy_manager_mut();
        if let Some(enemy) = enemy_manager.get_enemy_at_mut(&self.fi) {
            enemy.set_attack(capture_attack, accessor);
        }
    }
}
