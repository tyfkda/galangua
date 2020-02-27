use super::traj_command::TrajCommand;

pub const COMMAND_TABLE1: [TrajCommand; 11] = [
    TrajCommand::Pos((224 / 2 + 16) * 256, -8 * 256),
    TrajCommand::Speed(3 * 256),
    TrajCommand::Angle(128 * 256),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(10),
    TrajCommand::VAngle(2 * 256),
    TrajCommand::Delay(17),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(30),
    TrajCommand::DestAngle(32 * 256, 30 * 256),
    TrajCommand::VAngle(0),
];

pub const COMMAND_TABLE2: [TrajCommand; 11] = [
    TrajCommand::Pos(-8 * 256, 208 * 256),
    TrajCommand::Speed(3 * 256),
    TrajCommand::Angle(64 * 256),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(5),
    TrajCommand::VAngle(-1 * 256),
    TrajCommand::Delay(10),
    TrajCommand::VAngle(-2 * 256),
    TrajCommand::Delay(17),
    TrajCommand::DestAngle((-256 + 32) * 256, 20 * 256),
    TrajCommand::VAngle(0),
];
