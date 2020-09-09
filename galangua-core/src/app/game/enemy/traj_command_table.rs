use counted_array::counted_array;

use super::traj_command::TrajCommand;
use super::traj_command::TrajCommand::*;

use crate::app::consts::*;
use crate::util::math::{ANGLE, ONE};

counted_array!(pub const COMMAND_TABLE1: [TrajCommand; _] = [
    Pos((WIDTH / 2 + 24) * ONE, -8 * ONE),
    Speed(3 * ONE),
    Angle((ANGLE / 2) * ONE),
    VAngle(0),
    Delay(5),
    Shot(0),
    VAngle(2 * ONE),
    Delay(17),
    VAngle(0),
    Delay(30),
    DestAngle((ANGLE / 8) * ONE, 30 * ONE),
    VAngle(0),
]);

counted_array!(pub const COMMAND_TABLE2: [TrajCommand; _] = [
    Pos(-8 * ONE, 244 * ONE),
    Speed(3 * ONE),
    Angle((ANGLE / 4) * ONE),
    VAngle(-2 * ONE),
    Delay(16),
    VAngle(0),
    Delay(10),
    VAngle(-2 * ONE),
    Delay(17),
    Shot(8),
    DestAngle((-ANGLE + (ANGLE / 8)) * ONE, 20 * ONE),
    VAngle(0),
]);

counted_array!(pub const BEE_ATTACK_TABLE: [TrajCommand; _] = [
    Speed(2 * ONE),
    Angle(0),
    VAngle(-4 * ONE),
    Delay(41),

    VAngle(0),
    WaitYG(210 * ONE),

    VAngle(2 * ONE),
    Delay(80),
]);

counted_array!(pub const BEE_ATTACK_RUSH_CONT_TABLE: [TrajCommand; _] = [
    Speed(2 * ONE),
    VAngle(2 * ONE),
    Delay(65),

    VAngle(0),
    WaitYG(304 * ONE),
    AddPos(0 * ONE, -320 * ONE),
    CopyFormationX,
    Angle((ANGLE / 2) * ONE),
]);

counted_array!(pub const BUTTERFLY_ATTACK_TABLE: [TrajCommand; _] = [
    Speed(2 * ONE),
    Angle(0 * ONE),
    VAngle(-4 * ONE),
    Delay(40),

    VAngle(0 * ONE),
    WaitYG(160 * ONE),

    VAngle(4 * ONE),
    Delay(18),

    VAngle(0 * ONE),
    Delay(10),

    VAngle(-4 * ONE),
    Delay(18),

    VAngle(0 * ONE),
    WaitYG(304 * ONE),
    AddPos(0 * ONE, -320 * ONE),
    CopyFormationX,
    Angle((ANGLE / 2) * ONE),
]);

counted_array!(pub const OWL_ATTACK_TABLE: [TrajCommand; _] = [
    Speed(2 * ONE),
    Angle(0 * ONE),
    VAngle(-4 * ONE),
    Delay(32),

    VAngle(0 * ONE),
    WaitYG(110 * ONE),

    VAngle(-3 * ONE),
    Delay(94),

    VAngle(0 * ONE),
    WaitYG(200 * ONE),

    VAngle(1 * ONE),
    Delay(40),

    VAngle(0 * ONE),
    WaitYG(304 * ONE),
    AddPos(0 * ONE, -320 * ONE),
    CopyFormationX,
    Angle((ANGLE / 2) * ONE),
]);

counted_array!(pub const BEE_RUSH_ATTACK_TABLE: [TrajCommand; _] = [
    Speed(25 * ONE / 10),
    Angle((ANGLE / 2) * ONE),
    VAngle(0),
    WaitYG(8 * ONE),

    VAngle(3 * ONE),
    Delay(10),

    VAngle(0),
    Delay(20),

    VAngle(-3 * ONE),
    Delay(23),

    VAngle(0),
    Delay(20),

    VAngle(1 * ONE / 2),
    Delay(20),

    VAngle(0),
    WaitYG(220 * ONE),

    VAngle(25 * ONE / 10),
    Delay(113),

    VAngle(0),
    WaitYG(304 * ONE),
    AddPos(0 * ONE, -320 * ONE),
    CopyFormationX,
    Angle((ANGLE / 2) * ONE),
]);

counted_array!(pub const BUTTERFLY_RUSH_ATTACK_TABLE: [TrajCommand; _] = [
    Speed(25 * ONE / 10),
    Angle((ANGLE / 2) * ONE),
    VAngle(0),
    WaitYG(8 * ONE),

    VAngle(3 * ONE),
    Delay(10),

    VAngle(0),
    Delay(20),

    VAngle(-3 * ONE),
    Delay(23),

    VAngle(0),
    Delay(37),

    VAngle(3 * ONE),
    Delay(23),

    VAngle(0),
    WaitYG(304 * ONE),
    AddPos(0 * ONE, -320 * ONE),
    CopyFormationX,
    Angle((ANGLE / 2) * ONE),
]);

counted_array!(pub const OWL_RUSH_ATTACK_TABLE: [TrajCommand; _] = [
    Speed(25 * ONE / 10),
    Angle((ANGLE / 2) * ONE),
    VAngle(0),
    WaitYG(110 * ONE),

    VAngle(-4 * ONE),
    Delay(72),

    VAngle(0),
    WaitYG(200 * ONE),

    VAngle(15 * ONE / 10),
    Delay(25),

    VAngle(1 * ONE),
    WaitYG(304 * ONE),
    AddPos(0 * ONE, -320 * ONE),
    CopyFormationX,
    Angle((ANGLE / 2) * ONE),
]);
