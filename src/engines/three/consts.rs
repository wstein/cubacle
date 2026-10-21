pub(super) const INVERSE_SOLUTION: u8 = 0x01;
pub(super) const USE_SEPARATOR: u8 = 0x02;
pub(super) const APPEND_LENGTH: u8 = 0x04;
pub(super) const MAX_PREMV_LEN: i8 = 20;
pub(super) const MIN_P1PRE_LEN: i8 = 7;

pub(super) const N_FLIP: usize = 2048;
pub(super) const N_FLIP_SYM: usize = 336;
pub(super) const N_TWST: usize = 2187;
pub(super) const N_TWST_SYM: usize = 324;
pub(super) const N_SLICE: usize = 495;
pub(super) const N_PERM: usize = 40320;
pub(super) const N_PERM_SYM: usize = 2768;
pub(super) const N_MPERM: usize = 24;
pub(super) const N_CCOMB: usize = 70;

pub(super) const N_MOVES_P1: usize = 18;
pub(super) const N_MOVES_P2: usize = 10;
pub(super) const MAX_DEPTH2: usize = 13;
pub(super) const SEARCH_TIMEOUT: i8 = i8::MIN;
pub(super) const SEARCH_COMPLETE: i8 = i8::MIN + 1;
pub(super) const PREMOVE_TIMEOUT: i32 = i32::MIN;

pub(super) static MOVE2STR: [&str; 18] = [
    "U ", "U2", "U'", "R ", "R2", "R'", "F ", "F2", "F'", "D ", "D2", "D'", "L ", "L2", "L'", "B ",
    "B2", "B'",
];
pub(super) static URF_MOVE: [[u8; 18]; 6] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
    [6, 7, 8, 0, 1, 2, 3, 4, 5, 15, 16, 17, 9, 10, 11, 12, 13, 14],
    [3, 4, 5, 6, 7, 8, 0, 1, 2, 12, 13, 14, 15, 16, 17, 9, 10, 11],
    [2, 1, 0, 5, 4, 3, 8, 7, 6, 11, 10, 9, 14, 13, 12, 17, 16, 15],
    [8, 7, 6, 2, 1, 0, 5, 4, 3, 17, 16, 15, 11, 10, 9, 14, 13, 12],
    [5, 4, 3, 8, 7, 6, 2, 1, 0, 14, 13, 12, 17, 16, 15, 11, 10, 9],
];

pub(super) static P2MOVES: [u8; 18] =
    [0, 1, 2, 4, 7, 9, 10, 11, 13, 16, 3, 5, 6, 8, 12, 14, 15, 17];
