use super::enemy_command::EnemyCommand;

pub const COMMAND_TABLE1: [EnemyCommand; 12] = [
    EnemyCommand::Pos(224 / 2 * 256, -8 * 256),
    EnemyCommand::Speed(3 * 256),
    EnemyCommand::Angle(128 * 256),
    EnemyCommand::VAngle(0),
    EnemyCommand::Delay(20),
    EnemyCommand::VAngle(5 * 256),
    EnemyCommand::Delay(5),
    EnemyCommand::VAngle(0),
    EnemyCommand::Delay(30),
    EnemyCommand::VAngle(-3 * 256),
    EnemyCommand::Delay(45),
    EnemyCommand::VAngle(0),
];
