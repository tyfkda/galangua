use specs::prelude::*;

use galangua_common::framework::types::Vec2I;
use galangua_common::framework::{AppTrait, RendererTrait, VKey};
use galangua_common::util::math::ONE;

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
        //world.register::<Vel>();
        world.register::<SpriteDrawable>();

        let mut update_dispatcher = DispatcherBuilder::new().with(SysMover, "sys_mover", &[]).build();
        update_dispatcher.setup(&mut world);

        world.create_entity()
            .with(Pos(Vec2I::new(0 * ONE, 100 * ONE)))
            .with(Vel(Vec2I::new(1 * ONE, 0)))
            .with(SpriteDrawable)
            .build();

        Self {
            pressed_key: None,
            world,
            update_dispatcher,
        }
    }
}

impl<R: RendererTrait> AppTrait<R> for GalanguaEcsApp {
    fn on_key(&mut self, vkey: VKey, down: bool) {
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
