use legion::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::stage_indicator::StageIndicator;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::framework::{AppTrait, RendererTrait, VKey};
use galangua_common::framework::types::Vec2I;
use galangua_common::util::pad::Pad;

use super::components::*;
use super::resources::*;
use super::system::*;
use super::system::system_player::*;

pub struct GalanguaEcsApp {
    pressed_key: Option<VKey>,
    world: World,
    resources: Resources,
    schedule: Schedule,

    #[cfg(debug_assertions)]
    paused: bool,
}

impl GalanguaEcsApp {
    pub fn new() -> Self {
        let schedule = Schedule::builder()
            .add_system(update_game_controller_system())
            .add_system(move_star_system())
            .add_system(update_pad_system())
            .add_system(move_player_system())
            .add_system(fire_myshot_system())
            .add_system(move_myshot_system())
            .add_system(move_formation_system())
            .add_system(run_appearance_manager_system())
            .flush()
            .add_system(run_attack_manager_system())
            .add_system(move_zako_system())
            .add_system(move_owl_system())
            .add_system(move_troops_system())
            .add_system(move_tractor_beam_system())
            .add_system(coll_check_myshot_enemy_system())
            .add_system(coll_check_player_enemy_system())
            .add_system(recapture_fighter_system())
            .add_system(move_sequential_anime_system())
            .build();

        Self {
            pressed_key: None,
            world: World::default(),
            resources: Resources::default(),
            schedule,

            #[cfg(debug_assertions)]
            paused: false,
        }
    }

    fn update_main(&mut self) -> bool {
        if self.pressed_key == Some(VKey::Escape) {
            return false;
        }

        #[cfg(debug_assertions)]
        {
            if self.pressed_key == Some(VKey::Return) {
                self.paused = !self.paused;
            }
            if self.paused && self.pressed_key != Some(VKey::S) {
                return true;
            }
        }

        self.schedule.execute(&mut self.world, &mut self.resources);

        true
    }
}

impl<R: RendererTrait> AppTrait<R> for GalanguaEcsApp {
    fn on_key(&mut self, vkey: VKey, down: bool) {
        if let Some(mut pad) = self.resources.get_mut::<Pad>() {
            pad.on_key(vkey, down);
        }

        if down {
            self.pressed_key = Some(vkey);
        }
    }

    fn on_joystick_axis(&mut self, _axis_index: u8, _dir: i8) {
    }

    fn on_joystick_button(&mut self, _button_index: u8, _down: bool) {
    }

    fn init(&mut self, renderer: &mut R) {
        renderer.load_textures("assets", &["chr.png", "font.png"]);
        renderer.load_sprite_sheet("assets/chr.json");

        self.resources.insert(Pad::default());
        self.resources.insert(StarManager::default());
        self.resources.insert(StageIndicator::default());
        self.resources.insert(Formation::default());
        self.resources.insert(AppearanceManager::default());
        self.resources.insert(AttackManager::default());
        self.resources.insert(GameInfo::new());

        self.world.push((
            new_player(),
            Posture(Vec2I::new(CENTER_X, PLAYER_Y), 0),
            player_coll_rect(),
            player_sprite(),
        ));
    }

    fn update(&mut self) -> bool {
        let result = self.update_main();
        self.pressed_key = None;
        result
    }

    fn draw(&mut self, renderer: &mut R) {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        draw_system(&self.world, &self.resources, renderer);
    }
}
