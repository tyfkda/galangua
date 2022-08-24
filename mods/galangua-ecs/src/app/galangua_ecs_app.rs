use atomic_refcell::AtomicRef;
use legion::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::stage_indicator::StageIndicator;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::score_holder::ScoreHolder;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::{AppTrait, RendererTrait, SystemTrait, VKey};
use galangua_common::util::fps_calc::{FpsCalc, TimerTrait};
use galangua_common::util::pad::{Pad, PadBit};

use super::components::*;
use super::resources::*;
use super::system::system_player::*;
use super::system::*;

enum AppState {
    Title(Title),
    Game(Game),
}

pub struct GalanguaEcsApp<T: TimerTrait, S: SystemTrait> {
    system: S,
    pressed_key: Option<VKey>,
    state: AppState,
    pad: Pad,
    star_manager: StarManager,
    score_holder: ScoreHolder,
    fps_calc: FpsCalc<T>,

    #[cfg(debug_assertions)]
    paused: bool,
}

impl<T: TimerTrait, S: SystemTrait> GalanguaEcsApp<T, S> {
    pub fn new(timer: T, system: S) -> Self {
        let high_score = system.get_u32(&KEY_HIGH_SCORE)
                .or(Some(DEFAULT_HIGH_SCORE))
                .unwrap();

        Self {
            system,
            pressed_key: None,
            state: AppState::Title(Title::new()),
            pad: Pad::default(),
            star_manager: StarManager::default(),
            score_holder: ScoreHolder::new(high_score),
            fps_calc: FpsCalc::new(timer),

            #[cfg(debug_assertions)]
            paused: false,
        }
    }

    fn start_game(&mut self) {
        self.state = AppState::Game(Game::new(&self.star_manager, self.score_holder.high_score));
    }

    fn back_to_title(&mut self) {
        let mut high_score_updated = false;
        if let AppState::Game(game_state) = &mut self.state {
            if let Some(score_holder) = game_state.get_score_holder() {
                let prev_high_score = self.score_holder.high_score;
                self.score_holder = score_holder;
                high_score_updated = self.score_holder.high_score > prev_high_score;
            }
            if let Some(star_manager) = game_state.get_star_manager() {
                self.star_manager = star_manager.clone();  // Write back.
            }
            self.star_manager.set_stop(false);
            self.state = AppState::Title(Title::new());

            #[cfg(debug_assertions)]
            { self.paused = false; }
        }

        if high_score_updated {
            self.on_high_score_updated();
        }
    }

    fn on_high_score_updated(&mut self) {
        self.system.set_u32(KEY_HIGH_SCORE, self.score_holder.high_score);
    }
}

impl<R: RendererTrait, T: TimerTrait, S: SystemTrait> AppTrait<R> for GalanguaEcsApp<T, S> {
    fn on_key(&mut self, vkey: VKey, down: bool) {
        self.pad.on_key(vkey, down);
        if down {
            self.pressed_key = Some(vkey);
        }
    }

    fn on_joystick_axis(&mut self, axis_index: u8, dir: i8) {
        self.pad.on_joystick_axis(axis_index, dir);
    }

    fn on_joystick_button(&mut self, button_index: u8, down: bool) {
        self.pad.on_joystick_button(button_index, down);
    }

    fn init(&mut self, renderer: &mut R) {
        renderer.load_textures("assets", &["chr.png", "font.png"]);
        renderer.load_sprite_sheet("assets/chr.json");
    }

    fn update(&mut self) -> bool {
        self.pad.update();

        if self.pressed_key == Some(VKey::Escape) {
            match &self.state {
                AppState::Title(_title) => {
                    self.pressed_key = None;
                    return false;
                }
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
                if let Some(value) = title.update(&self.pad, &mut self.star_manager) {
                    if value {
                        self.start_game();
                    } else {
                        return false;
                    }
                }
            }
            AppState::Game(game) => {
                if !game.update(&self.pad, &mut self.system) {
                    self.back_to_title();
                }
            }
        };
        self.pressed_key = None;
        true
    }

    fn draw(&mut self, renderer: &mut R) {
        match &self.state {
            AppState::Title(title) => title.draw(&self.star_manager, &self.score_holder, self.system.is_touch_device(), renderer),
            AppState::Game(game) => game.draw(renderer),
        }

        self.fps_calc.update();

        #[cfg(debug_assertions)]
        {
            renderer.set_texture_color_mod("font", 128, 128, 128);
            renderer.draw_str("font", 23 * 8, 0 * 8, &format!("FPS{:2}", self.fps_calc.fps()));
        }
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

    fn update(&mut self, pad: &Pad, star_manager: &mut StarManager) -> Option<bool> {
        self.frame_count = self.frame_count.wrapping_add(1);

        star_manager.update();

        if pad.is_trigger(PadBit::A) {
            return Some(true);
        }
        None
    }

    fn draw(&self, star_manager: &StarManager, score_holder: &ScoreHolder, is_touch_device: bool, renderer: &mut impl RendererTrait) {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        star_manager.draw(renderer);

        renderer.set_texture_color_mod("font", 255, 255, 255);
        renderer.draw_str("font", 10 * 8, 8 * 8, "GALANGUA");

        if self.frame_count & 32 == 0 {
            let msg = if is_touch_device {
                "PRESS \"SHOT\" TO START"
            } else {
                "PRESS SPACE KEY TO START"
            };
            renderer.draw_str("font", (28 - msg.len() as i32) / 2 * 8, 25 * 8, msg);
        }
        score_holder.draw(renderer, true);

        renderer.set_texture_color_mod("font", 128, 128, 128);
        renderer.draw_str("font", WIDTH - (VERSION.len() as i32) * 8, HEIGHT - 1 * 8, VERSION);
    }
}

struct Game {
    world: World,
    resources: Resources,
    schedule: Schedule,
}

impl Game {
    fn new(star_manager: &StarManager, high_score: u32) -> Self {
        let schedule = Schedule::builder()
            .add_system(update_game_controller_system())
            .add_system(move_star_system())
            .add_system(move_player_system())
            .add_system(fire_myshot_system())
            .add_system(move_myshot_system())
            .add_system(move_formation_system())
            .add_system(run_appearance_manager_system())
            .flush()
            .add_system(run_attack_manager_system())
            .add_system(move_zako_system())
            .add_system(animate_zako_system())
            .add_system(move_owl_system())
            .add_system(animate_owl_system())
            .add_system(move_troops_system())
            .add_system(move_tractor_beam_system())
            .add_system(spawn_eneshot_system())
            .add_system(move_eneshot_system())
            .add_system(coll_check_myshot_enemy_system())
            .add_system(coll_check_player_enemy_system())
            .add_system(coll_check_player_eneshot_system())
            .add_system(recapture_fighter_system())
            .add_system(move_sequential_anime_system())
            .build();

        let mut resources = Resources::default();
        resources.insert(star_manager.clone());
        resources.insert(StageIndicator::default());
        resources.insert(Formation::default());
        resources.insert(AppearanceManager::default());
        resources.insert(AttackManager::default());
        resources.insert(EneShotSpawner::default());
        resources.insert(GameInfo::new(high_score));
        resources.insert(SoundQueue::new());

        let mut world = World::default();
        world.push((
            new_player(),
            Posture(Vec2I::new(CENTER_X, PLAYER_Y), 0),
            player_coll_rect(),
            player_sprite(),
        ));

        Self {
            world,
            resources,
            schedule,
        }
    }

    fn update(&mut self, pad: &Pad, system: &mut impl SystemTrait) -> bool {
        self.resources.insert(pad.clone());

        self.schedule.execute(&mut self.world, &mut self.resources);

        {
            let mut sound_queue = self.resources.get_mut::<SoundQueue>().unwrap();
            sound_queue.flush(system);
        }

        self.resources.get::<GameInfo>()
            .map_or(true, |game_info| game_info.game_state != GameState::Finished)
    }

    fn draw(&self, renderer: &mut impl RendererTrait) {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        draw_system(&self.world, &self.resources, renderer);
    }

    fn get_score_holder(&self) -> Option<ScoreHolder> {
        self.resources.get::<GameInfo>()
            .map(|game_info| game_info.score_holder.clone())
    }

    fn get_star_manager<'a>(&'a self) -> Option<AtomicRef<StarManager>> {
        self.resources.get::<StarManager>()
    }
}
