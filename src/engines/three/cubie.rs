#[derive(Clone, Copy)]
pub(super) struct Cubie {
    pub(super) ca: [u8; 8],
    pub(super) ea: [u8; 12],
}

pub(super) const U: u8 = 0;
pub(super) const R: u8 = 9;
pub(super) const F: u8 = 18;
pub(super) const D: u8 = 27;
pub(super) const L: u8 = 36;
pub(super) const B: u8 = 45;

pub(super) static CORNER_FACELET: [[u8; 3]; 8] = [
    [U + 8, R, F + 2],
    [U + 6, F, L + 2],
    [U, L, B + 2],
    [U + 2, B, R + 2],
    [D + 2, F + 8, R + 6],
    [D, L + 8, F + 6],
    [D + 6, B + 8, L + 6],
    [D + 8, R + 8, B + 6],
];

pub(super) static EDGE_FACELET: [[u8; 2]; 12] = [
    [U + 5, R + 1],
    [U + 7, F + 1],
    [U + 3, L + 1],
    [U + 1, B + 1],
    [D + 5, R + 7],
    [D + 1, F + 7],
    [D + 3, L + 7],
    [D + 7, B + 7],
    [F + 5, R + 3],
    [F + 3, L + 5],
    [B + 5, L + 3],
    [B + 3, R + 5],
];

#[allow(clippy::needless_range_loop)]
impl Cubie {
    pub(super) fn new() -> Self {
        let mut cc = Cubie {
            ca: [0; 8],
            ea: [0; 12],
        };
        cc.reset();
        cc
    }

    pub(super) fn reset(&mut self) {
        for i in 0..8 {
            self.ca[i] = i as u8;
        }
        for i in 0..12 {
            self.ea[i] = i as u8 * 2;
        }
    }

    pub(super) fn cmp(&self, other: &Cubie) -> i8 {
        for i in 0..8 {
            if self.ca[i] != other.ca[i] {
                return (self.ca[i] as i8) - (other.ca[i] as i8);
            }
        }
        for i in 0..12 {
            if self.ea[i] != other.ea[i] {
                return (self.ea[i] as i8) - (other.ea[i] as i8);
            }
        }
        0
    }

    pub(super) fn corn_mult(a: &Cubie, b: &Cubie, prod: &mut Cubie) {
        for cn in 0..8 {
            let ori_a = a.ca[(b.ca[cn] & 0x7) as usize] >> 3;
            let ori_b = b.ca[cn] >> 3;
            let mut ori = ori_a + if ori_a < 3 { ori_b } else { 6 - ori_b };
            ori = ori % 3 + if (ori_a < 3) == (ori_b < 3) { 0 } else { 3 };
            prod.ca[cn] = (a.ca[(b.ca[cn] & 0x7) as usize] & 0x7) | (ori << 3);
        }
    }

    pub(super) fn edge_mult(a: &Cubie, b: &Cubie, prod: &mut Cubie) {
        for ed in 0..12 {
            prod.ea[ed] = a.ea[(b.ea[ed] >> 1) as usize] ^ (b.ea[ed] & 1);
        }
    }

    pub(super) fn inv(src: &Cubie, inv: &mut Cubie) {
        for ed in 0..12 {
            inv.ea[(src.ea[ed] >> 1) as usize] = (ed as u8 * 2) | (src.ea[ed] & 0x1);
        }
        for cn in 0..8 {
            inv.ca[(src.ca[cn] & 0x7) as usize] =
                cn as u8 | (((0x20 >> (src.ca[cn] >> 3)) & 0x18) as u8);
        }
    }
}

pub(super) fn get_nparity(mut idx: i32, n: i32) -> i32 {
    let mut p = 0;
    let mut i = n - 2;
    while i >= 0 {
        p ^= idx % (n - i);
        idx /= n - i;
        i -= 1;
    }
    p & 1
}

pub(super) fn get_nperm(arr: &[u8], n: i32) -> i32 {
    let mut idx = 0;
    let mut val = 0x76543210;
    for i in 0..(n - 1) {
        let v = arr[i as usize] << 2;
        idx = (n - i) * idx + ((val >> v) & 0xf);
        val -= 0x11111110 << v;
    }
    idx
}

pub(super) fn set_nperm(arr: &mut [u8], mut idx: u16, n: u16) {
    let mut extract = 0;
    let mut val = 0x76543210;
    for i in 2..=n {
        extract = (extract << 4) | (idx % i) as u32;
        idx /= i;
    }
    for i in 0..(n - 1) {
        let v = (extract & 0xf) << 2;
        extract >>= 4;
        arr[i as usize] = ((val >> v) & 0xf) as u8;
        let m = (1 << v) - 1;
        val = (val & m) | ((val >> 4) & !m);
    }
    arr[(n - 1) as usize] = (val & 0xf) as u8;
}

pub(super) fn get_comb(arr: &[u8], n: i32, mask: i32) -> i32 {
    let mut idx_c = 0;
    let mut r = 4;
    let mut cnk = if n == 12 { 330 } else { 35 };
    for i in (0..n).rev() {
        if (arr[i as usize] & 0xc) == mask as u8 {
            idx_c += cnk;
            cnk = cnk * r / std::cmp::max(1, i - r + 1);
            r -= 1;
        }
        cnk = cnk * (i - r) / std::cmp::max(1, i);
    }
    idx_c
}

pub(super) fn set_comb(arr: &mut [u8], mut idx_c: i32, n: i32, mask: i32) {
    let mut r = 4;
    let mut fill = n - 1;
    let mut cnk = if n == 12 { 330 } else { 35 };
    for i in (0..n).rev() {
        if idx_c >= cnk {
            idx_c -= cnk;
            cnk = cnk * r / std::cmp::max(1, i - r + 1);
            r -= 1;
            arr[i as usize] = (r | mask) as u8;
        } else {
            if (fill & 0xc) == mask {
                fill -= 4;
            }
            arr[i as usize] = fill as u8;
            fill -= 1;
        }
        cnk = cnk * (i - r) / std::cmp::max(1, i);
    }
}

#[allow(clippy::needless_range_loop)]
impl Cubie {
    pub(super) fn get_flip(&self) -> i32 {
        let mut idx = 0;
        for i in 0..11 {
            idx = (idx << 1) | (self.ea[i] & 1) as i32;
        }
        idx
    }

    pub(super) fn set_flip(&mut self, mut idx: u16) {
        let mut parity = 0;
        for i in (0..11).rev() {
            let val = idx & 1;
            idx >>= 1;
            parity ^= val;
            self.ea[i] = (self.ea[i] & !1) | (val as u8);
        }
        self.ea[11] = (self.ea[11] & !1) | (parity as u8);
    }

    pub(super) fn get_twst(&self) -> i32 {
        let mut idx = 0;
        for i in 0..7 {
            idx += (idx << 1) + (self.ca[i] >> 3) as i32;
        }
        idx
    }

    pub(super) fn set_twst(&mut self, mut idx: u16) {
        let mut twst = 15;
        for i in (0..7).rev() {
            let val = idx % 3;
            idx /= 3;
            twst -= val;
            self.ca[i] = (self.ca[i] & 0x7) | (val << 3) as u8;
        }
        self.ca[7] = (self.ca[7] & 0x7) | ((twst % 3) << 3) as u8;
    }

    pub(super) fn get_slice(&self) -> u16 {
        let mut arr = [0u8; 12];
        for i in 0..12 {
            arr[i] = self.ea[i] >> 1;
        }
        494 - get_comb(&arr, 12, 8) as u16
    }

    pub(super) fn set_slice(&mut self, idx: i32) {
        let mut arr = [0u8; 12];
        set_comb(&mut arr, 494 - idx, 12, 8);
        for i in 0..12 {
            self.ea[i] = (self.ea[i] & 1) | (arr[i] << 1);
        }
    }

    pub(super) fn get_cperm(&self) -> i32 {
        let mut arr = [0u8; 8];
        for i in 0..8 {
            arr[i] = self.ca[i] & 0x7;
        }
        get_nperm(&arr, 8)
    }

    pub(super) fn set_cperm(&mut self, idx: u16) {
        let mut arr = [0u8; 8];
        set_nperm(&mut arr, idx, 8);
        for i in 0..8 {
            self.ca[i] = (self.ca[i] & !0x7) | arr[i];
        }
    }

    pub(super) fn get_eperm(&self) -> i32 {
        let mut arr = [0u8; 8];
        for i in 0..8 {
            arr[i] = self.ea[i] >> 1;
        }
        get_nperm(&arr, 8)
    }

    pub(super) fn set_eperm(&mut self, idx: u16) {
        let mut arr = [0u8; 8];
        set_nperm(&mut arr, idx, 8);
        for i in 0..8 {
            self.ea[i] = (self.ea[i] & 1) | (arr[i] << 1);
        }
    }

    pub(super) fn get_mperm(&self) -> i32 {
        let mut arr = [0u8; 4];
        for i in 8..12 {
            arr[i - 8] = (self.ea[i] >> 1) & 0x3;
        }
        get_nperm(&arr, 4)
    }

    pub(super) fn set_mperm(&mut self, idx: u16) {
        let mut arr = [0u8; 4];
        set_nperm(&mut arr, idx, 4);
        for i in 8..12 {
            self.ea[i] = (self.ea[i] & 1) | ((arr[i - 8] + 8) << 1);
        }
    }

    pub(super) fn get_ccomb(&self) -> i32 {
        let mut arr = [0u8; 8];
        for i in 0..8 {
            arr[i] = self.ca[i] & 0x7;
        }
        get_comb(&arr, 8, 0)
    }

    pub(super) fn set_ccomb(&mut self, idx: i32) {
        let mut arr = [0u8; 8];
        set_comb(&mut arr, idx, 8, 0);
        for i in 0..8 {
            self.ca[i] = (self.ca[i] & !0x7) | arr[i];
        }
    }
}

#[allow(clippy::needless_range_loop)]
impl Cubie {
    pub fn verify(&self) -> i32 {
        let mut sum = 0;
        let mut edge_mask = 0;
        for e in 0..12 {
            edge_mask |= 1 << (self.ea[e] >> 1);
            sum ^= self.ea[e] & 1;
        }
        if edge_mask != 0xfff {
            return -2;
        } else if sum != 0 {
            return -3;
        }
        let mut corn_mask = 0;
        for c in 0..8 {
            corn_mask |= 1 << (self.ca[c] & 7);
            sum += self.ca[c] >> 3;
        }
        if corn_mask != 0xff {
            return -4;
        } else if sum % 3 != 0 {
            return -5;
        }
        let mut parity = get_nparity(self.get_cperm(), 8);
        let mut ea: [u8; 12] = self.ea;
        for i in 0..12 {
            while ((ea[i] as usize) >> 1) != i {
                let j = (ea[i] as usize) >> 1;
                ea.swap(i, j);
                parity ^= 1;
            }
        }
        if parity != 0 {
            return -6;
        }
        0
    }

    pub(super) fn set_from_facelet(&mut self, facelet: &str) -> i32 {
        if facelet.len() < 54 {
            return -1;
        }
        let fstr: &[u8] = facelet.as_bytes();
        let mut f: [u8; 54] = [0; 54];
        let colors: [u8; 6] = [fstr[4], fstr[13], fstr[22], fstr[31], fstr[40], fstr[49]];
        let mut count: i32 = 0;
        for i in 0..54 {
            if let Some(fidx) = colors.iter().position(|&s| s == fstr[i]) {
                f[i] = fidx as u8;
                count += 1 << (fidx * 4);
            } else {
                return -1;
            }
        }
        if count != 0x999999 {
            return -1;
        }
        self.reset();
        let mut ori: usize;
        for i in 0..8 {
            ori = 0;
            while ori < 3 {
                if f[CORNER_FACELET[i][ori] as usize] == 0
                    || f[CORNER_FACELET[i][ori] as usize] == 3
                {
                    break;
                }
                ori += 1
            }
            let col1 = f[CORNER_FACELET[i][(ori + 1) % 3] as usize];
            let col2 = f[CORNER_FACELET[i][(ori + 2) % 3] as usize];
            for j in 0..8 {
                if col1 == CORNER_FACELET[j][1] / 9 && col2 == CORNER_FACELET[j][2] / 9 {
                    self.ca[i] = (ori as u8 % 3) << 3 | j as u8;
                    break;
                }
            }
        }
        for i in 0..12 {
            for j in 0..12 {
                if f[EDGE_FACELET[i][0] as usize] == EDGE_FACELET[j][0] / 9
                    && f[EDGE_FACELET[i][1] as usize] == EDGE_FACELET[j][1] / 9
                {
                    self.ea[i] = (j as u8) << 1;
                    break;
                }
                if f[EDGE_FACELET[i][0] as usize] == EDGE_FACELET[j][1] / 9
                    && f[EDGE_FACELET[i][1] as usize] == EDGE_FACELET[j][0] / 9
                {
                    self.ea[i] = ((j as u8) << 1) | 1;
                    break;
                }
            }
        }
        0
    }

    #[cfg(test)]
    pub(super) fn as_facelet(&self) -> String {
        let colors: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];
        let mut f: [u8; 54] = [0; 54];
        for i in 0..54 {
            f[i] = (i as u8) / 9;
        }
        for c in 0..8 {
            let j = (self.ca[c] & 0x7) as usize;
            let ori = (self.ca[c] >> 3) as usize;
            for n in 0..3 {
                f[CORNER_FACELET[c][(n + ori) % 3] as usize] = CORNER_FACELET[j][n] / 9;
            }
        }
        for e in 0..12 {
            let j = (self.ea[e] >> 1) as usize;
            let ori = (self.ea[e] & 1) as usize;
            for n in 0..2 {
                f[EDGE_FACELET[e][(n + ori) % 2] as usize] = EDGE_FACELET[j][n] / 9;
            }
        }
        let mut buf = String::new();
        for i in 0..54 {
            buf.push(colors[f[i] as usize]);
        }
        buf
    }
}
