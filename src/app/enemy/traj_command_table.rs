use super::traj_command::TrajCommand;

pub const COMMAND_TABLE1: [TrajCommand; 12] = [
    TrajCommand::Pos(224 / 2 * 256, -8 * 256),
    TrajCommand::Speed(3 * 256),
    TrajCommand::Angle(128 * 256),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(10),
    TrajCommand::VAngle(2 * 256),
    TrajCommand::Delay(15),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(30),
    TrajCommand::DestAngle(0 * 256, 30 * 256),
    TrajCommand::VAngle(0),
    TrajCommand::Angle(0 * 256),
];
