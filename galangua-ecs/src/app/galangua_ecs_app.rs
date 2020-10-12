use legion::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_table::{ENEMY_TYPE_TABLE, ORDER};
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::formation_table::{BASE_X_TABLE, BASE_Y_TABLE};
use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::traj_command_table::{BUTTERFLY_ATTACK_TABLE};
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::EnemyType;
use galangua_common::framework::{AppTrait, RendererTrait, VKey};
use galangua_common::framework::types::{Vec2I, ZERO_VEC};
use galangua_common::util::math::ONE;
use galangua_common::util::pad::Pad;

use super::components::*;
use super::system::*;

pub struct GalanguaEcsApp {
    pressed_key: Option<VKey>,
    world: World,
    resources: Resources,
    schedule: Schedule,
}

impl GalanguaEcsApp {
    pub fn new() -> Self {
        let schedule = Schedule::builder()
            .add_system(update_pad_system())
            .add_system(move_player_system())
            .add_system(fire_myshot_system())
            .add_system(move_myshot_system())
            .add_system(move_formation_system())
            .add_system(move_zako_system())
            .add_system(coll_check_myshot_enemy_system())
            .add_system(move_star_system())
            .build();

        Self {
            pressed_key: None,
            world: World::default(),
            resources: Resources::default(),
            schedule,
        }
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
        {
            let mut formation = Formation::default();
            formation.done_appearance();
            self.resources.insert(formation);
        }

        self.world.extend(vec![
            (Player, Posture(Vec2I::new(CENTER_X, PLAYER_Y), 0), SpriteDrawable {sprite_name: "rustacean", offset: Vec2I::new(-8, -8)}),
        ]);

        let enemies = (0..ORDER.len()).map(|i| {
            let fi = ORDER[i];
            let enemy_type = ENEMY_TYPE_TABLE[i / 4];
            let pos = &Vec2I::new(BASE_X_TABLE[fi.0 as usize], BASE_Y_TABLE[fi.1 as usize]) * ONE;
            let sprite_name = match enemy_type {
                EnemyType::Bee => "gopher1",
                EnemyType::Butterfly => "dman1",
                EnemyType::Owl => "cpp11",
                EnemyType::CapturedFighter => "rustacean_captured",
            };
            let zako = if i == 0 {
                let mut traj = Traj::new(&BUTTERFLY_ATTACK_TABLE, &ZERO_VEC, false, fi);
                traj.set_pos(&pos);
                Zako { state: ZakoState::Attack, traj: Some(traj) }
            } else {
                Zako { state: ZakoState::Formation, traj: None }
            };
            (
                zako,
                Enemy { enemy_type, formation_index: fi },
                Posture(pos, 0),
                Speed(0, 0),
                CollRect { offset: Vec2I::new(-6, -6), size: Vec2I::new(12, 12) },
                SpriteDrawable {sprite_name, offset: Vec2I::new(-8, -8)},
            )
        });
        self.world.extend(enemies);
    }

    fn update(&mut self) -> bool {
        if self.pressed_key == Some(VKey::Escape) {
            return false;
        }

        self.schedule.execute(&mut self.world, &mut self.resources);

        self.pressed_key = None;
        true
    }

    fn draw(&mut self, renderer: &mut R) {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        draw_system(&self.world, &self.resources, renderer);
    }
}
