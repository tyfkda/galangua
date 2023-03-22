use super::enemy_manager::EnemyManager;

use crate::app::game::enemy::enemy::{create_appearance_enemy, Enemy};
use crate::app::game::enemy::Accessor;
use crate::app::game::manager::CaptureEventType;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::Accessor as AccessorForAppearance;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::attack_manager::Accessor as AttackManagerAccessor;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::{CaptureState, FormationIndex};
use galangua_common::app::util::collision::CollBox;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::unsafe_util::peep;

#[cfg(debug_assertions)]
use galangua_common::app::game::appearance_table::{ENEMY_TYPE_TABLE, ORDER};

#[cfg(debug_assertions)]
use crate::app::game::enemy::enemy::create_enemy;

#[derive(Clone, Copy, PartialEq)]
enum StageState {
    Appearance,
    Normal,
    Rush,
    Cleared,
}

pub struct StageManager {
    enemy_manager: EnemyManager,
    formation: Formation,
    appearance_manager: AppearanceManager,
    attack_manager: AttackManager,
    stage_state: StageState,
}

impl StageManager {
    pub fn new() -> Self {
        Self {
            enemy_manager: EnemyManager::new(),
            formation: Formation::default(),
            appearance_manager: AppearanceManager::default(),
            attack_manager: AttackManager::default(),
            stage_state: StageState::Appearance,
        }
    }

    pub fn start_next_stage(&mut self, stage: u16, captured_fighter: Option<FormationIndex>) {
        self.enemy_manager.start_next_stage();
        self.appearance_manager.restart(stage, captured_fighter);
        self.formation.restart();
        self.attack_manager.restart(stage);
        self.stage_state = StageState::Appearance;
    }

    pub fn all_destroyed(&self) -> bool {
        self.stage_state == StageState::Cleared &&
            self.enemy_manager.all_destroyed()
    }

    pub fn update(&mut self, accessor: &mut impl Accessor) {
        self.update_appearance();
        self.update_formation();
        self.update_attackers(accessor);
        self.enemy_manager.update(accessor);
        self.check_stage_state();
    }

    pub fn draw(&self, renderer: &mut impl RendererTrait) {
        self.enemy_manager.draw(renderer);
    }

    pub fn check_collision(
        &mut self, target: &CollBox, power: u32, accessor: &mut impl Accessor,
    ) -> bool {
        self.enemy_manager.check_collision(target, power, accessor)
    }

    pub fn check_shot_collision(&mut self, target: &CollBox) -> bool {
        self.enemy_manager.check_shot_collision(target)
    }

    fn update_appearance(&mut self) {
        let prev_done = self.appearance_manager.done;
        let accessor = unsafe { peep(self) };
        if let Some(new_borns) = self.appearance_manager.update(accessor) {
            for e in new_borns {
                self.enemy_manager.spawn(create_appearance_enemy(
                    e.enemy_type, &e.pos, e.angle, e.speed,
                    &e.fi, e.traj,
                ));
            }
        }
        if !prev_done && self.appearance_manager.done {
            self.stage_state = StageState::Normal;
            self.formation.done_appearance();
            self.attack_manager.set_enable(true);
        }
    }

    pub fn spawn_captured_fighter(&mut self, pos: &Vec2I, fi: &FormationIndex) -> bool {
        self.enemy_manager.spawn_captured_fighter(pos, fi)
    }

    pub fn remove_enemy(&mut self, formation_index: &FormationIndex) -> bool {
        self.enemy_manager.remove_enemy(formation_index)
    }

    fn update_formation(&mut self) {
        self.formation.update();
    }

    fn update_attackers(&mut self, accessor: &mut impl Accessor) {
        let acc = AttackManagerAccessorImpl(accessor);
        let result = self.attack_manager.update(&acc);
        if let Some((fi, capture_attack)) = result {
            self.attack_manager.put_attacker(&fi);
            let enemy = {
                let accessor = unsafe { peep(accessor) };
                accessor.get_enemy_at_mut(&fi).unwrap()
            };
            enemy.start_attack(capture_attack, accessor);
            if capture_attack {
                accessor.capture_event(CaptureEventType::StartCaptureAttack(fi));
            }
        }
    }

    fn check_stage_state(&mut self) {
        if self.stage_state == StageState::Appearance {
            return;
        }

        let new_state = match self.enemy_manager.alive_enemy_count {
            n if n == 0               => StageState::Cleared,
            n if n <= RUSH_THRESHOLD  => StageState::Rush,
            _                         => self.stage_state,
        };
        if new_state != self.stage_state {
            self.stage_state = new_state;
        }
    }

    pub fn pause_enemy_shot(&mut self, wait: u32) {
        self.enemy_manager.pause_enemy_shot(wait);
    }

    pub fn spawn_shot(&mut self, pos: &Vec2I, target_pos: &[Option<Vec2I>], speed: i32) {
        self.enemy_manager.spawn_shot(pos, target_pos, speed);
    }

    pub fn pause_attack(&mut self, value: bool) {
        self.attack_manager.pause(value);
        self.appearance_manager.pause(value);
    }

    pub fn is_no_attacker(&self) -> bool {
        self.attack_manager.is_no_attacker()
    }

    pub fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&dyn Enemy> {
        self.enemy_manager.get_enemy_at(formation_index)
    }

    pub fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Box<dyn Enemy>> {
        self.enemy_manager.get_enemy_at_mut(formation_index)
    }

    pub fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.formation.pos(formation_index)
    }

    pub fn is_rush(&self) -> bool {
        self.stage_state == StageState::Rush
    }

    // Debug

    #[cfg(debug_assertions)]
    pub fn reset_stable(&mut self) {
        self.enemy_manager.reset_stable();

        let stage = 0;
        self.appearance_manager.restart(stage, None);
        self.appearance_manager.done = true;
        self.formation.restart();
        self.formation.done_appearance();
        self.attack_manager.restart(stage);
        self.attack_manager.set_enable(false);
        self.stage_state = StageState::Normal;

        for unit in 0..5 {
            for i in 0..8 {
                let index = ORDER[unit * 8 + i];
                let enemy_type = ENEMY_TYPE_TABLE[unit * 2 + (i / 4)];
                let pos = self.formation.pos(&index);
                let mut enemy = create_enemy(enemy_type, &pos, 0, 0, &index);
                enemy.set_to_formation();
                self.enemy_manager.spawn(enemy);
            }
        }
    }
}

impl AccessorForAppearance for StageManager {
    fn is_stationary(&self) -> bool {
        self.enemy_manager.is_stationary()
    }
}

struct AttackManagerAccessorImpl<'a, A: Accessor>(&'a A);
impl<'a, A: Accessor> AttackManagerAccessor for AttackManagerAccessorImpl<'a, A> {
    fn can_capture_attack(&self) -> bool { self.0.capture_state() == CaptureState::NoCapture }
    fn captured_fighter_index(&self) -> Option<FormationIndex> { self.0.captured_fighter_index() }
    fn is_enemy_live_at(&self, formation_index: &FormationIndex) -> bool { self.0.get_enemy_at(formation_index).is_some() }
    fn is_enemy_formation_at(&self, formation_index: &FormationIndex) -> bool {
        if let Some(enemy) = self.0.get_enemy_at(formation_index) {
            enemy.is_formation()
        } else {
            false
        }
    }
}
