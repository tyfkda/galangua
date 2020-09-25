use counted_array::counted_array;

use crate::app::game::enemy::traj_command::TrajCommand;
use crate::app::game::enemy::traj_command_table::*;
use crate::app::game::enemy::{EnemyType, FormationIndex};

const fn p(x: u8, y: u8) -> FormationIndex { FormationIndex(x, y) }

pub(super) const ORDER: [FormationIndex; 40] = [
    p(4, 2), p(5, 2), p(4, 3), p(5, 3),
    p(4, 4), p(5, 4), p(4, 5), p(5, 5),

    p(3, 1), p(4, 1), p(5, 1), p(6, 1),
    p(3, 2), p(6, 2), p(3, 3), p(6, 3),

    p(8, 2), p(7, 2), p(8, 3), p(7, 3),
    p(1, 2), p(2, 2), p(1, 3), p(2, 3),

    p(7, 4), p(6, 4), p(7, 5), p(6, 5),
    p(3, 4), p(2, 4), p(3, 5), p(2, 5),

    p(9, 4), p(8, 4), p(9, 5), p(8, 5),
    p(0, 4), p(1, 4), p(0, 5), p(1, 5),
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
