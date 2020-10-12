use legion::*;

use galangua_common::framework::{AppTrait, RendererTrait, VKey};

use galangua_common::framework::types::Vec2I;
use galangua_common::util::math::ONE;

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
            .add_system(mover_system())
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

        self.world.extend(vec![
            (Pos(Vec2I::new(0 * ONE, 100 * ONE)), Vel(Vec2I::new(1 * ONE, 0)), SpriteDrawable),
        ]);
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

        draw_system(&self.world, renderer);
    }
}
