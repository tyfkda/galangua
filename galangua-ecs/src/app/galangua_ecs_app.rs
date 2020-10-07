use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::{AppTrait, RendererTrait, VKey};
use galangua_common::util::pad::Pad;

use super::components::*;
use super::resources::*;
use super::system::*;
use super::system::system_player::*;

pub struct GalanguaEcsApp {
    pressed_key: Option<VKey>,
    world: World,
    update_dispatcher: Dispatcher<'static, 'static>,

    #[cfg(debug_assertions)]
    paused: bool,
}

impl GalanguaEcsApp {
    pub fn new() -> Self {
        let mut world = World::new();
        // Components are registered automatically which used in dispatcher.
        //world.register::<Pos>();
        world.register::<SpriteDrawable>();

        let mut update_dispatcher = DispatcherBuilder::new()
            .with(SysGameController, "game_controller", &[])
            .with(SysPadUpdater, "pad_updater", &[])
            .with(SysPlayerMover, "player_mover", &["pad_updater", "game_controller"])
            .with(SysPlayerFirer, "player_firer", &["player_mover"])
            .with(SysMyShotMover, "myshot_mover", &["player_firer"])
            .with(SysFormationMover, "formation_mover", &["game_controller"])
            .with(SysAppearanceManager, "appearance_manager", &["game_controller"])
            .with(SysAttackManager, "attack_manager", &["appearance_manager"])
            .with(SysZakoMover, "zako_mover", &["formation_mover", "appearance_manager", "attack_manager"])
            .with(SysOwlMover, "owl_mover", &["formation_mover", "appearance_manager", "attack_manager"])
            .with(SysTroopsMover, "troops_mover", &["owl_mover"])
            .with(SysTractorBeamMover, "tractor_beam_mover", &["player_mover", "owl_mover"])
            .with(SysCollCheckMyShotEnemy, "collcheck_myshot_enemy", &["myshot_mover", "zako_mover", "owl_mover"])
            .with(SysCollCheckPlayerEnemy, "collcheck_player_enemy", &["player_mover", "zako_mover", "owl_mover", "tractor_beam_mover", "collcheck_myshot_enemy"])
            .with(SysRecaptureFighter, "recapture_fighter", &["collcheck_myshot_enemy", "collcheck_player_enemy"])
            .with(SysSequentialSpriteAnime, "sprite_anime", &[])
            .with(SysStarMover, "star_mover", &["game_controller", "tractor_beam_mover", "collcheck_player_enemy"])
            .build();
        update_dispatcher.setup(&mut world);

        world.create_entity()
            .with(new_player())
            .with(Posture(Vec2I::new(CENTER_X, PLAYER_Y), 0))
            .with(player_coll_rect())
            .with(player_sprite())
            .build();

        world.insert(Pad::default());
        world.insert(StarManager::default());
        {
            let mut appearance_manager = AppearanceManager::default();
            appearance_manager.restart(0, None);
            world.insert(appearance_manager);
        }
        world.insert(Formation::default());
        world.insert(AttackManager::default());
        world.insert(GameInfo::new());

        Self {
            pressed_key: None,
            world,
            update_dispatcher,

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

        self.update_dispatcher.dispatch(&mut self.world);
        self.world.maintain();

        true
    }
}

impl<R: RendererTrait> AppTrait<R> for GalanguaEcsApp {
    fn on_key(&mut self, vkey: VKey, down: bool) {
        let mut pad = self.world.fetch_mut::<Pad>();
        pad.on_key(vkey, down);
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
    }

    fn update(&mut self) -> bool {
        let result = self.update_main();
        self.pressed_key = None;
        result
    }

    fn draw(&mut self, renderer: &mut R) {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        let mut sys_drawer = SysDrawer(renderer);
        sys_drawer.run_now(&mut self.world);
    }
}
