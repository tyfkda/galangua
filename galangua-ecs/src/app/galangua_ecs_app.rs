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
use super::system::*;

pub struct GalanguaEcsApp {
    pressed_key: Option<VKey>,
    world: World,
    update_dispatcher: Dispatcher<'static, 'static>,
}

impl GalanguaEcsApp {
    pub fn new() -> Self {
        let mut world = World::new();
        // Components are registered automatically which used in dispatcher.
        //world.register::<Pos>();
        world.register::<SpriteDrawable>();

        let mut update_dispatcher = DispatcherBuilder::new()
            .with(SysPadUpdater, "pad_updater", &[])
            .with(SysPlayerMover, "player_mover", &["pad_updater"])
            .with(SysPlayerFirer, "player_firer", &["player_mover"])
            .with(SysMyShotMover, "myshot_mover", &["player_firer"])
            .with(SysFormationMover, "formation_mover", &[])
            .with(SysAppearanceManager, "appearance_manager", &[])
            .with(SysAttackManager, "attack_manager", &["appearance_manager"])
            .with(SysZakoMover, "zako_mover", &["formation_mover", "appearance_manager", "attack_manager"])
            .with(SysCollCheckMyShotEnemy, "collcheck_myshot_enemy", &["myshot_mover", "zako_mover"])
            .with(SysSequentialSpriteAnime, "sprite_anime", &[])
            .with(SysStarMover, "star_mover", &[])
            .build();
        update_dispatcher.setup(&mut world);

        world.create_entity()
            .with(Player)
            .with(Posture(Vec2I::new(CENTER_X, PLAYER_Y), 0))
            .with(SpriteDrawable {sprite_name: "rustacean", offset: Vec2I::new(-8, -8)})
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

        Self {
            pressed_key: None,
            world,
            update_dispatcher,
        }
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
        if self.pressed_key == Some(VKey::Escape) {
            return false;
        }

        self.update_dispatcher.dispatch(&mut self.world);
        self.world.maintain();

        self.pressed_key = None;
        true
    }

    fn draw(&mut self, renderer: &mut R) {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        let mut sys_drawer = SysDrawer(renderer);
        sys_drawer.run_now(&mut self.world);
    }
}
