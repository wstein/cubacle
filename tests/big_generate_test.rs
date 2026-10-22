//! Random 6x6x6 and 7x7x7 states must be cubes: Orbit64 refuses impossible ones.

use cubacle::generate::random_state;
use cubacle::notation::orbit64;
use cubacle::puzzle::PuzzleKind;

#[test]
fn random_large_cubes_encode_and_decode() {
    for kind in [PuzzleKind::Cube6, PuzzleKind::Cube7] {
        for _ in 0..50 {
            let cube = random_state(kind).expect("a generator");
            let token = orbit64::encode(&cube).unwrap_or_else(|e| panic!("{kind:?}: {e}"));
            assert_eq!(orbit64::decode(&token).unwrap(), cube);
        }
    }
}
