use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{round_up, ONE};

pub struct MyShot {
    pos: Vec2I,
    dual: bool,
}

impl MyShot {
    pub fn new(pos: Vec2I, dual: bool) -> Self {
        Self {
            pos,
            dual,
        }
    }

    pub fn update(&mut self) -> bool {
        self.pos.y -= 8 * ONE;

        self.pos.y >= 0
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        let pos = self.pos();
        renderer.draw_sprite("myshot", pos + Vec2I::new(-2, -8))?;
        if self.dual {
            renderer.draw_sprite("myshot", pos + Vec2I::new(-2 + 16, -8))?;
        }

        Ok(())
    }

    pub fn get_collbox_for_dual(&self) -> Option<CollBox> {
        if self.dual {
            Some(CollBox {
                top_left: self.pos() + Vec2I::new(-1 + 16, -4),
                size: Vec2I::new(1, 8),
            })
        } else {
            None
        }
    }

    fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }
}

impl Collidable for MyShot {
    fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: self.pos() - Vec2I::new(1, 4),
            size: Vec2I::new(1, 8),
        }
    }
}
