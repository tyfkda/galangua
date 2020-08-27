use counted_array::counted_array;

use super::traj_command::TrajCommand;

use crate::app::consts::*;
use crate::util::math::{ANGLE, ONE};

counted_array!(pub const COMMAND_TABLE1: [TrajCommand; _] = [
    TrajCommand::Pos((WIDTH / 2 + 24) * ONE, -8 * ONE),
    TrajCommand::Speed(3 * ONE),
    TrajCommand::Angle((ANGLE / 2) * ONE),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(5),
    TrajCommand::VAngle(2 * ONE),
    TrajCommand::Delay(17),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(30),
    TrajCommand::DestAngle((ANGLE / 8) * ONE, 30 * ONE),
    TrajCommand::VAngle(0),
]);

counted_array!(pub const COMMAND_TABLE2: [TrajCommand; _] = [
    TrajCommand::Pos(-8 * ONE, 244 * ONE),
    TrajCommand::Speed(3 * ONE),
    TrajCommand::Angle((ANGLE / 4) * ONE),
    TrajCommand::VAngle(-2 * ONE),
    TrajCommand::Delay(16),
    TrajCommand::VAngle(0),
    TrajCommand::Delay(10),
    TrajCommand::VAngle(-2 * ONE),
    TrajCommand::Delay(17),
    TrajCommand::DestAngle((-ANGLE + (ANGLE / 8)) * ONE, 20 * ONE),
    TrajCommand::VAngle(0),
]);

counted_array!(pub const BEE_ATTACK_TABLE: [TrajCommand; _] = [
    TrajCommand::Speed(2 * ONE),
    TrajCommand::Angle(0),
    TrajCommand::VAngle(-4 * ONE),
    TrajCommand::Delay(41),

    TrajCommand::VAngle(0),
    TrajCommand::WaitYG(210 * ONE),

    TrajCommand::VAngle(2 * ONE),
    TrajCommand::Delay(80),
]);

counted_array!(pub const BEE_ATTACK_RUSH_CONT_TABLE: [TrajCommand; _] = [
    TrajCommand::Speed(2 * ONE),
    TrajCommand::VAngle(2 * ONE),
    TrajCommand::Delay(65),

    TrajCommand::VAngle(0),
    TrajCommand::WaitYG(304 * ONE),
    TrajCommand::AddPos(0 * ONE, -320 * ONE),
    TrajCommand::CopyFormationX,
    TrajCommand::Angle(128 * ONE),
]);

counted_array!(pub const BUTTERFLY_ATTACK_TABLE: [TrajCommand; _] = [
    TrajCommand::Speed(2 * ONE),
    TrajCommand::Angle(0 * ONE),
    TrajCommand::VAngle(-4 * ONE),
    TrajCommand::Delay(40),

    TrajCommand::VAngle(0 * ONE),
    TrajCommand::WaitYG(160 * ONE),

    TrajCommand::VAngle(4 * ONE),
    TrajCommand::Delay(18),

    TrajCommand::VAngle(0 * ONE),
    TrajCommand::Delay(10),

    TrajCommand::VAngle(-4 * ONE),
    TrajCommand::Delay(18),

    TrajCommand::VAngle(0 * ONE),
    TrajCommand::WaitYG(304 * ONE),
    TrajCommand::AddPos(0 * ONE, -320 * ONE),
    TrajCommand::CopyFormationX,
    TrajCommand::Angle(128 * ONE),
]);

counted_array!(pub const OWL_ATTACK_TABLE: [TrajCommand; _] = [
    TrajCommand::Speed(2 * ONE),
    TrajCommand::Angle(0 * ONE),
    TrajCommand::VAngle(-4 * ONE),
    TrajCommand::Delay(32),

    TrajCommand::VAngle(0 * ONE),
    TrajCommand::WaitYG(110 * ONE),

    TrajCommand::VAngle(-3 * ONE),
    TrajCommand::Delay(94),

    TrajCommand::VAngle(0 * ONE),
    TrajCommand::WaitYG(200 * ONE),

    TrajCommand::VAngle(1 * ONE),
    TrajCommand::Delay(32),

    TrajCommand::VAngle(0 * ONE),
    TrajCommand::WaitYG(304 * ONE),
    TrajCommand::AddPos(0 * ONE, -320 * ONE),
    TrajCommand::CopyFormationX,
    TrajCommand::Angle(128 * ONE),
]);
