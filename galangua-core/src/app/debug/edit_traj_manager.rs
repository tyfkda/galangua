use crate::app::game::enemy::traj_command::TrajCommand;
use crate::app::game::enemy::FormationIndex;
use crate::app::game::game_manager::GameManager;
use crate::app::util::unsafe_util::peep;
use crate::framework::types::Vec2I;
use crate::framework::{RendererTrait, VKey};
use crate::util::math::ONE;

use std::fs::File;
use std::io::{prelude::*, BufReader};

pub struct EditTrajManager {
    fi: FormationIndex,
    no: u32,
    flip_x: bool,
    from_top: bool,
}

impl EditTrajManager {
    pub fn new() -> Self {
        Self {
            fi: FormationIndex(0, 5),
            no: 0,
            flip_x: false,
            from_top: false,
        }
    }

    pub fn update(&mut self, pressed_key: Option<VKey>, game_manager: &mut GameManager) {
        if pressed_key == Some(VKey::Left) && self.fi.0 > 0 {
            self.fi.0 -= 1;
        }
        if pressed_key == Some(VKey::Right) && self.fi.0 < 9 {
            self.fi.0 += 1;
        }
        if pressed_key == Some(VKey::Up) && self.fi.1 > 0 {
            self.fi.1 -= 1;
        }
        if pressed_key == Some(VKey::Down) && self.fi.1 < 5 {
            self.fi.1 += 1;
        }

        if pressed_key == Some(VKey::Num1) {
            self.set_traj_attack(game_manager, self.no, self.flip_x);
        }
        if pressed_key == Some(VKey::Num2) {
            self.set_attack(game_manager, false);
        }
        if pressed_key == Some(VKey::Num3) {
            self.set_attack(game_manager, true);
        }
        if pressed_key == Some(VKey::Num9) && self.no > 0 {
            self.no -= 1;
        }
        if pressed_key == Some(VKey::Num0) {
            self.no += 1;
        }
        if pressed_key == Some(VKey::F) {
            self.flip_x = !self.flip_x;
        }
        if pressed_key == Some(VKey::T) {
            self.from_top = !self.from_top;
        }
    }

    pub fn draw<R: RendererTrait>(&mut self, renderer: &mut R, game_manager: &mut GameManager) -> Result<(), String> {
        let enemy_manager = game_manager.enemy_manager_mut();
        let pos = &(&enemy_manager.get_formation_pos(&self.fi) / ONE) + &Vec2I::new(-8, -8);
        renderer.set_draw_color(255, 0, 255);
        renderer.fill_rect(Some([&pos, &Vec2I::new(16, 1)]))?;
        renderer.fill_rect(Some([&pos, &Vec2I::new(1, 16)]))?;
        renderer.fill_rect(Some([&(&pos + &Vec2I::new(0, 15)), &Vec2I::new(16, 1)]))?;
        renderer.fill_rect(Some([&(&pos + &Vec2I::new(15, 0)), &Vec2I::new(1, 16)]))?;

        renderer.set_texture_color_mod("font", 128, 128, 128);
        renderer.draw_str("font", 0 * 8, 0 * 8, "EDIT MODE")?;
        renderer.draw_str("font", 0 * 8, 1 * 8, &format!("NO={}", self.no))?;
        renderer.draw_str("font", 0 * 8, 2 * 8, &format!("F)LIP={}", self.flip_x.to_string().to_uppercase()))?;
        renderer.draw_str("font", 0 * 8, 3 * 8, &format!("T)OP={}", self.from_top.to_string().to_uppercase()))?;
        Ok(())
    }

    fn set_attack(&mut self, game_manager: &mut GameManager, capture_attack: bool) {
        let accessor = unsafe { peep(game_manager) };
        let enemy_manager = game_manager.enemy_manager_mut();
        if let Some(enemy) = enemy_manager.get_enemy_at_mut(&self.fi) {
            enemy.set_attack(capture_attack, accessor);
        }
    }

    fn set_traj_attack(&mut self, game_manager: &mut GameManager, no: u32, flip_x: bool) {
        let filename = format!("debug/debug_traj{}.txt", no);
        if let Some(traj_command_vec) = load_traj_command_file(&filename) {
            let enemy_manager = game_manager.enemy_manager_mut();
            if let Some(enemy) = enemy_manager.get_enemy_at_mut(&self.fi) {
                if self.from_top {
                    let pos = *enemy.raw_pos();
                    enemy.set_pos(&Vec2I::new(pos.x, -16 * ONE));
                }
                enemy.set_table_attack(traj_command_vec, flip_x);
            }
        } else {
            eprintln!("{} load failed", &filename);
        }
    }
}

fn load_traj_command_file(filename: &str) -> Option<Vec<TrajCommand>> {
    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut vec = Vec::new();
            let mut err = false;
            let mut lineno = 0;
            let one = ONE as f64;
            for line in reader.lines() {
                lineno += 1;
                if let Ok(line) = line {
                    if line.len() > 0 && line.starts_with("#") {
                        continue;
                    }

                    let words: Vec<&str> = line.split_whitespace().collect();
                    if words.len() >= 1 {
                        match words[0] {
                            "Pos" if words.len() >= 3 => {
                                if let (Ok(x), Ok(y)) = (&words[1].parse::<f64>(), &words[2].parse::<f64>()) {
                                    let ix = (x * one).round() as i32;
                                    let iy = (y * one).round() as i32;
                                    vec.push(TrajCommand::Pos(ix, iy));
                                    continue;
                                }
                                println!("Line {}: number expected", lineno);
                                err = true;
                            }
                            "Speed" if words.len() >= 2 => {
                                if let Ok(speed) = &words[1].parse::<f64>() {
                                    let ispeed = (speed * one).round() as i32;
                                    vec.push(TrajCommand::Speed(ispeed));
                                } else {
                                    println!("Line {}: number expected", lineno);
                                    err = true;
                                }
                            }
                            "Angle" if words.len() >= 2 => {
                                if let Ok(angle) = &words[1].parse::<f64>() {
                                    let iangle = (angle * one).round() as i32;
                                    vec.push(TrajCommand::Angle(iangle));
                                } else {
                                    println!("Line {}: number expected", lineno);
                                    err = true;
                                }
                            }
                            "VAngle" if words.len() >= 2 => {
                                if let Ok(vangle) = &words[1].parse::<f64>() {
                                    let ivangle = (vangle * one).round() as i32;
                                    vec.push(TrajCommand::VAngle(ivangle));
                                } else {
                                    println!("Line {}: number expected", lineno);
                                    err = true;
                                }
                            }
                            "Delay" if words.len() >= 2 => {
                                if let &Ok(delay) = &words[1].parse::<u32>() {
                                    vec.push(TrajCommand::Delay(delay));
                                } else {
                                    println!("Line {}: number expected", lineno);
                                    err = true;
                                }
                            }
                            "DestAngle" if words.len() >= 3 => {
                                if let (Ok(angle), Ok(radius)) = (&words[1].parse::<f64>(), &words[2].parse::<f64>()) {
                                    let iangle = (angle * one).round() as i32;
                                    let iradius = (radius * one).round() as i32;
                                    vec.push(TrajCommand::DestAngle(iangle, iradius));
                                    continue;
                                }
                                println!("Line {}: number expected", lineno);
                                err = true;
                            }
                            "WaitYG" if words.len() >= 2 => {
                                if let Ok(value) = &words[1].parse::<f64>() {
                                    let ivalue = (value * one).round() as i32;
                                    vec.push(TrajCommand::WaitYG(ivalue));
                                } else {
                                    println!("Line {}: number expected", lineno);
                                    err = true;
                                }
                            }
                            "AddPos" if words.len() >= 3 => {
                                if let (Ok(x), Ok(y)) = (&words[1].parse::<f64>(), &words[2].parse::<f64>()) {
                                    let ix = (x * one).round() as i32;
                                    let iy = (y * one).round() as i32;
                                    vec.push(TrajCommand::AddPos(ix, iy));
                                    continue;
                                }
                                println!("Line {}: number expected", lineno);
                                err = true;
                            }
                            "CopyFormationX" => {
                                vec.push(TrajCommand::CopyFormationX);
                            }
                            _ => {
                                println!("Line {}: Unhandled, {:?}", lineno, words);
                                err = true;
                            }
                        }
                    }
                }
            }
            if err {
                None
            } else {
                Some(vec)
            }
        }
        Err(_err) => None,
    }
}
