use super::traj_command::TrajCommand;
use super::super::consts::*;

pub const COMMAND_TABLE1: [TrajCommand; 11] = [
    TrajCommand::Pos((WIDTH / 2 + 16) * ONE, -8 * ONE),
    TrajCommand::Speed(3 * ONE),
    TrajCommand::Angle((ANGLE / 2) * ONE),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(10),
    TrajCommand::VAngle(2 * ONE),
    TrajCommand::Delay(17),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(30),
    TrajCommand::DestAngle((ANGLE / 8) * ONE, 30 * ONE),
    TrajCommand::VAngle(0),
];

pub const COMMAND_TABLE2: [TrajCommand; 11] = [
    TrajCommand::Pos(-8 * ONE, 208 * ONE),
    TrajCommand::Speed(3 * ONE),
    TrajCommand::Angle((ANGLE / 4) * ONE),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(5),
    TrajCommand::VAngle(-1 * ONE),
    TrajCommand::Delay(10),
    TrajCommand::VAngle(-2 * ONE),
    TrajCommand::Delay(17),
    TrajCommand::DestAngle((-ANGLE + (ANGLE / 8)) * ONE, 20 * ONE),
    TrajCommand::VAngle(0),
];
