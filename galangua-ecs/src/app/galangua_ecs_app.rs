use specs::prelude::*;
use std::marker::PhantomData;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::stage_indicator::StageIndicator;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::score_holder::ScoreHolder;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::{AppTrait, RendererTrait, SystemTrait, VKey};
use galangua_common::util::pad::{Pad, PadBit};

use super::components::*;
use super::resources::*;
use super::system::*;
use super::system::system_player::*;

enum AppState {
    Title(Title),
    Game(Game),
}

pub struct GalanguaEcsApp<S: SystemTrait> {
    pressed_key: Option<VKey>,
    state: AppState,
    pad: Pad,
    score_holder: ScoreHolder,

    #[cfg(debug_assertions)]
    paused: bool,

    phantom_data: PhantomData<S>,
}

impl<S: SystemTrait> GalanguaEcsApp<S> {
    pub fn new(system: S) -> Self {
        let high_score = system.get_u32(&KEY_HIGH_SCORE)
                .or(Some(DEFAULT_HIGH_SCORE))
                .unwrap();

        Self {
            pressed_key: None,
            state: AppState::Title(Title::new()),
            pad: Pad::default(),
            score_holder: ScoreHolder::new(high_score),

            #[cfg(debug_assertions)]
            paused: false,

            phantom_data: PhantomData,
        }
    }

    fn start_game(&mut self) {
        self.state = AppState::Game(Game::new(self.score_holder.high_score));
    }

    fn back_to_title(&mut self) {
        if let AppState::Game(game_state) = &mut self.state {
            if let Some(sh) = game_state.get_score_holder() {
                self.score_holder = sh.clone();
            }
            self.state = AppState::Title(Title::new());

            #[cfg(debug_assertions)]
            { self.paused = false; }
        }
    }
}

impl<R: RendererTrait, S: SystemTrait> AppTrait<R> for GalanguaEcsApp<S> {
    fn on_key(&mut self, vkey: VKey, down: bool) {
        self.pad.on_key(vkey, down);
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
        self.pad.update();

        if self.pressed_key == Some(VKey::Escape) {
            match &self.state {
                AppState::Title(_title) => { return false; }
                _ => self.back_to_title(),
            }
        }

        #[cfg(debug_assertions)]
        {
            if self.pressed_key == Some(VKey::Return) {
                self.paused = !self.paused;
            }
            if self.paused && self.pressed_key != Some(VKey::S) {
                self.pressed_key = None;
                return true;
            }
        }

        match &mut self.state {
            AppState::Title(title) => {
                if let Some(value) = title.update(&self.pad) {
                    if value {
                        self.start_game();
                    } else {
                        return false;
                    }
                }
            }
            AppState::Game(game) => {
                if !game.update(&self.pad) {
                    self.back_to_title();
                }
            }
        };
        self.pressed_key = None;
        true
    }

    fn draw(&mut self, renderer: &mut R) {
        match &self.state {
            AppState::Title(title) => title.draw(&self.score_holder, renderer),
            AppState::Game(game) => game.draw(renderer),
        };
    }
}

struct Title {
    frame_count: u32,
}

impl Title {
    fn new() -> Self {
        Self {
            frame_count: 0,
        }
    }

    fn update(&mut self, pad: &Pad) -> Option<bool> {
        self.frame_count = self.frame_count.wrapping_add(1);

        if pad.is_trigger(PadBit::A) {
            return Some(true);
        }
        None
    }

    fn draw<R: RendererTrait>(&self, score_holder: &ScoreHolder, renderer: &mut R) {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        renderer.set_texture_color_mod("font", 255, 255, 255);
        renderer.draw_str("font", 10 * 8, 8 * 8, "GALANGUA");

        if self.frame_count & 32 == 0 {
            renderer.draw_str("font", 2 * 8, 25 * 8, "PRESS SPACE KEY TO START");
        }
        score_holder.draw(renderer, true)
    }
}

struct Game {
    world: World,
    update_dispatcher: Dispatcher<'static, 'static>,
}

impl Game {
    fn new(high_score: u32) -> Self {
        let mut world = World::new();
        // Components are registered automatically which used in dispatcher.
        //world.register::<Pos>();
        //world.register::<SpriteDrawable>();
        //world.register::<TextureColor>();

        let mut update_dispatcher = DispatcherBuilder::new()
            .with(SysGameController, "game_controller", &[])
            .with(SysPlayerMover, "player_mover", &["game_controller"])
            .with(SysPlayerFirer, "player_firer", &["player_mover"])
            .with(SysMyShotMover, "myshot_mover", &["player_firer"])
            .with(SysFormationMover, "formation_mover", &["game_controller"])
            .with(SysAppearanceManager, "appearance_manager", &["game_controller"])
            .with(SysAttackManager, "attack_manager", &["appearance_manager"])
            .with(SysZakoMover, "zako_mover", &["formation_mover", "appearance_manager", "attack_manager"])
            .with(SysOwlMover, "owl_mover", &["formation_mover", "appearance_manager", "attack_manager"])
            .with(SysTroopsMover, "troops_mover", &["owl_mover"])
            .with(SysTractorBeamMover, "tractor_beam_mover", &["player_mover", "owl_mover"])
            .with(SysEneShotSpawner, "eneshot_spawner", &["zako_mover", "owl_mover"])
            .with(SysEneShotMover, "eneshot_mover", &["zako_mover", "owl_mover", "eneshot_spawner"])
            .with(SysCollCheckMyShotEnemy, "collcheck_myshot_enemy", &["myshot_mover", "zako_mover", "owl_mover"])
            .with(SysCollCheckPlayerEnemy, "collcheck_player_enemy", &["player_mover", "zako_mover", "owl_mover", "tractor_beam_mover", "collcheck_myshot_enemy"])
            .with(SysCollCheckPlayerEneShot, "collcheck_player_eneshot", &["player_mover", "eneshot_mover", "collcheck_myshot_enemy"])
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

        world.insert(StarManager::default());
        world.insert(StageIndicator::default());
        world.insert(Formation::default());
        world.insert(AppearanceManager::default());
        world.insert(AttackManager::default());
        world.insert(GameInfo::new(high_score));
        world.insert(EneShotSpawner::default());

        Self {
            world,
            update_dispatcher,
        }
    }

    fn update(&mut self, pad: &Pad) -> bool {
        self.world.insert(pad.clone());

        self.update_dispatcher.dispatch(&mut self.world);
        self.world.maintain();

        self.world.get_mut::<GameInfo>()
            .map_or(true, |game_info| game_info.game_state != GameState::Finished)
    }

    fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        let mut sys_drawer = SysDrawer(renderer);
        sys_drawer.run_now(&self.world);
    }

    fn get_score_holder<'a>(&'a mut self) -> Option<&'a ScoreHolder> {
        self.world.get_mut::<GameInfo>()
            .map(|game_info| &game_info.score_holder)
    }
}
