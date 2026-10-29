use super::geometry::{axis, dot, position_rank, Rotation};
use super::{face_axis, invalid, moves, FrameError, Print, State, Sticker};

impl Print {
    /// Validates all identities, body bindings, facelets and artwork frames.
    pub fn new(mut stickers: Vec<Sticker>) -> Result<Print, FrameError> {
        if stickers.len() != 24 {
            return Err(invalid("a print needs exactly 24 stickers"));
        }
        stickers.sort_by_key(|s| s.id);
        let mut positions = [[0; 3]; 8];
        let mut counts = [0; 8];
        let mut facelets = [false; 24];
        for (id, s) in stickers.iter().enumerate() {
            if s.id as usize != id || s.cubie >= 8 {
                return Err(invalid("sticker ids must be unique 0..23 and bodies 0..7"));
            }
            let rank = position_rank(s.p).ok_or_else(|| invalid("invalid corner position"))?;
            let normal = axis(s.n).ok_or_else(|| invalid("normal must be a signed axis"))?;
            if dot(s.p, s.n) != 1 || facelets[rank as usize * 3 + normal] {
                return Err(invalid(
                    "print facelets must cover each outward corner facelet once",
                ));
            }
            facelets[rank as usize * 3 + normal] = true;
            if axis(s.up).is_none() || dot(s.n, s.up) != 0 {
                return Err(invalid(
                    "artwork up must be a signed axis orthogonal to normal",
                ));
            }
            if s.image
                .as_ref()
                .is_some_and(|image| image.chars().count() > 256)
            {
                return Err(invalid("image exceeds 256 characters"));
            }
            let b = s.cubie as usize;
            if counts[b] != 0 && positions[b] != s.p {
                return Err(invalid("a physical body cannot span corner positions"));
            }
            positions[b] = s.p;
            counts[b] += 1;
        }
        if counts != [3; 8] {
            return Err(invalid("every body needs exactly three stickers"));
        }
        let mut occupied = [false; 8];
        for p in positions {
            let rank = position_rank(p).unwrap() as usize;
            if occupied[rank] {
                return Err(invalid("two bodies cannot share a reference position"));
            }
            occupied[rank] = true;
        }
        let all = Rotation::all();
        let rotations = std::array::from_fn(|body| {
            let mut indexed = [all[0]; 24];
            for r in &all {
                let rank = position_rank(r.apply(positions[body])).unwrap();
                indexed[(3 * rank + r.twist_digit()) as usize] = *r;
            }
            indexed
        });
        let move_table = std::array::from_fn(|body| {
            std::array::from_fn(|pose| {
                std::array::from_fn(|m| {
                    let (axis, sign) = face_axis(moves()[m].face);
                    let mut r = rotations[body][pose];
                    if r.apply(positions[body])[axis] == sign {
                        for _ in 0..(m % 3 + 1) {
                            r = r.quarter(axis, -sign);
                        }
                    }
                    3 * position_rank(r.apply(positions[body])).unwrap() + r.twist_digit()
                })
            })
        });
        Ok(Print {
            stickers: stickers.try_into().unwrap(),
            positions,
            rotations,
            move_table,
        })
    }

    pub fn printing(&self) -> &[Sticker; 24] {
        &self.stickers
    }

    pub fn reference(&self) -> State {
        State {
            poses: self.positions.map(|p| 3 * position_rank(p).unwrap()),
        }
    }
}
