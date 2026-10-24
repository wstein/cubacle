use super::*;

fn layout(spec: &[(bool, &[usize])]) -> Layout {
    let mut at = 0;
    Layout {
        blocks: spec
            .iter()
            .map(|(outer, sizes)| BlockRange {
                outer: *outer,
                units: sizes
                    .iter()
                    .map(|n| {
                        at += n;
                        at - n..at
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn cut(rows: &[Vec<RowBlock>]) -> Vec<Vec<Vec<Range<usize>>>> {
    rows.iter()
        .map(|r| {
            r.iter()
                .map(|b| b.pieces.iter().map(|p| p.moves.clone()).collect())
                .collect()
        })
        .collect()
}

// an icon of 100 px: 1 icon 100, 2 icons 202, 3 icons 304
fn fit(width: u32) -> Fit {
    Fit {
        icon: 100,
        width,
        icons: usize::MAX,
    }
}

#[test]
fn rows_break_between_groups_only() {
    let l = layout(&[(false, &[2]), (false, &[2]), (false, &[1])]);
    // 202 + 28 + 202 = 432 > 500? no: fits two groups, then the third (+28+100)
    let rows = pack(&l, fit(440));
    assert_eq!(
        cut(&rows),
        vec![vec![vec![0..2], vec![2..4]], vec![vec![4..5]]]
    );
}

#[test]
fn a_block_stays_together_when_it_fits() {
    let l = layout(&[(false, &[1]), (true, &[1, 1]), (false, &[1])]);
    // block = 100 + 16 + 100 = 216; after the first group 100 + 28 + 216 = 344 > 300
    let rows = pack(&l, fit(300));
    assert_eq!(
        cut(&rows),
        vec![vec![vec![0..1]], vec![vec![1..2, 2..3]], vec![vec![3..4]]]
    );
}

#[test]
fn a_block_wider_than_a_row_breaks_between_its_groups() {
    let l = layout(&[(true, &[1, 1, 1])]);
    let rows = pack(&l, fit(250));
    assert_eq!(cut(&rows), vec![vec![vec![0..1, 1..2]], vec![vec![2..3]]]);
    assert_eq!(rows[0][0].id, rows[1][0].id);
}

#[test]
fn a_group_wider_than_a_row_is_cut_into_icons() {
    let l = layout(&[(false, &[5]), (false, &[1])]);
    let rows = pack(&l, fit(310));
    // 3 icons = 304 per row
    assert_eq!(
        cut(&rows),
        vec![vec![vec![0..3]], vec![vec![3..5]], vec![vec![5..6]]]
    );
    assert!(rows[0][0].pieces[0].first);
    assert!(!rows[1][0].pieces[0].first);
    assert_eq!(heading(&rows[1], 2), "group 1 of 2 (continued)");
}

#[test]
fn columns_limit_the_icons_per_row() {
    let l = layout(&[(false, &[1]), (false, &[1]), (false, &[1])]);
    let rows = pack(
        &l,
        Fit {
            icon: 100,
            width: 5000,
            icons: 2,
        },
    );
    assert_eq!(
        cut(&rows),
        vec![vec![vec![0..1], vec![1..2]], vec![vec![2..3]]]
    );
}

#[test]
fn headings_count_groups() {
    let l = layout(&[(false, &[1]), (false, &[1]), (false, &[1])]);
    let rows = pack(&l, fit(5000));
    assert_eq!(heading(&rows[0], 3), "groups 1-3 of 3");
    let one = pack(&layout(&[(false, &[2])]), fit(5000));
    assert_eq!(heading(&one[0], 9), "group 1 of 9");
}
