use counted_array::counted_array;

use super::enemy::EnemyType;
use super::traj_command::TrajCommand;
use super::traj_command_table::*;
use super::FormationIndex;

const fn pos(x: u8, y: u8) -> FormationIndex { FormationIndex(x, y) }

pub(super) const ORDER: [FormationIndex; 40] = [
    pos(4, 2), pos(5, 2), pos(4, 3), pos(5, 3),
    pos(4, 4), pos(5, 4), pos(4, 5), pos(5, 5),

    pos(3, 1), pos(4, 1), pos(5, 1), pos(6, 1),
    pos(3, 2), pos(6, 2), pos(3, 3), pos(6, 3),

    pos(8, 2), pos(7, 2), pos(8, 3), pos(7, 3),
    pos(1, 2), pos(2, 2), pos(1, 3), pos(2, 3),

    pos(7, 4), pos(6, 4), pos(7, 5), pos(6, 5),
    pos(3, 4), pos(2, 4), pos(3, 5), pos(2, 5),

    pos(9, 4), pos(8, 4), pos(9, 5), pos(8, 5),
    pos(0, 4), pos(1, 4), pos(0, 5), pos(1, 5),
];

pub(super) const ENEMY_TYPE_TABLE: [EnemyType; 2 * 5] = [
    EnemyType::Butterfly, EnemyType::Bee,
    EnemyType::Owl, EnemyType::Butterfly,
    EnemyType::Butterfly, EnemyType::Butterfly,
    EnemyType::Bee, EnemyType::Bee,
    EnemyType::Bee, EnemyType::Bee,
];

pub(super) struct UnitTableEntry<'a> {
    pub(super) pat: usize,
    pub(super) table: &'a [TrajCommand],
    pub(super) flip_x: bool,
}

counted_array!(pub(super) const UNIT_TABLE: [[UnitTableEntry; 5]; _] = [
    [
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 1, table: &COMMAND_TABLE2, flip_x: false },
        UnitTableEntry { pat: 1, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 2, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 2, table: &COMMAND_TABLE1, flip_x: true },
    ],
    [
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE3, flip_x: true },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE2, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE1, flip_x: true },
    ],
    [
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE2, flip_x: false },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
    ],
    [
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE3, flip_x: true },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE2, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE3, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE3, flip_x: true },
    ],
]);

counted_array!(pub(super) const ASSAULT_TABLE: [[u32; 5]; _] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [1, 0, 0, 1, 1],
    [1, 0, 0, 1, 1],
    [1, 0, 0, 1, 1],
    [1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1],
    [2, 1, 1, 2, 2],
    [2, 1, 1, 2, 2],
    [2, 1, 1, 2, 2],
    [2, 2, 2, 2, 2],
]);

counted_array!(pub(super) const SHOT_ENABLE_TABLE: [[u32; 5]; _] = [
    [0, 0, 0, 0, 0],
    [3, 4, 4, 4, 4],
    [4, 4, 4, 4, 4],
    [4, 4, 4, 4, 4],
    [5, 5, 5, 5, 5],
]);
