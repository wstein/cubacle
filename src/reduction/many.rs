//! Growing the list of reductions until enough finished solutions are
//! distinct.
//!
//! New code. A list of results asks the last reduction phase for its first
//! `quota` results, in the serial order, so a longer request starts with the
//! shorter one. Two reductions can end in the same normalized solution; the
//! caller counts the distinct ones and the quota grows until there are enough,
//! a longer request brings nothing new, or the growth limit is reached.

/// The largest quota asked for, as a multiple of the number of results wanted
/// plus a margin: duplicates are the exception, so a long search for more
/// is not worth its time.
const GROWTH: usize = 4;
const MARGIN: usize = 16;
/// The first quota asked for. A long list starts smaller and doubles: the lists
/// share their start, so a clock that runs out later still leaves the results of the
/// earlier requests.
const START: usize = 100;

/// The reductions for `wanted` results: `list(quota)` gives the first `quota`
/// of them (fewer when there are no more), `enough` is told the whole list and
/// says whether it holds `wanted` distinct finished solutions. An error of a
/// later request ends the growth with the list so far.
pub(crate) fn grow<T, E>(
    wanted: usize,
    mut list: impl FnMut(usize) -> Result<Vec<T>, E>,
    mut enough: impl FnMut(&[T]) -> bool,
) -> Result<Vec<T>, E> {
    let limit = wanted.saturating_mul(GROWTH).saturating_add(MARGIN);
    let mut quota = wanted.min(START);
    let mut have = list(quota)?;
    loop {
        if enough(&have) || quota >= limit {
            return Ok(have);
        }
        quota = quota.saturating_mul(2).min(limit);
        match list(quota) {
            // a phase may return fewer than its quota while it has more: the
            // end of the list is a request that brings nothing new
            Ok(more) if more.len() > have.len() => have = more,
            Ok(_) | Err(_) => return Ok(have),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_until_enough_and_stops_at_the_end_of_the_list() {
        let source: Vec<u32> = (0..10).collect();
        let take = |q: usize| Ok::<_, ()>(source.iter().copied().take(q).collect::<Vec<_>>());
        // enough at once
        assert_eq!(grow(3, take, |l| l.len() >= 3).unwrap().len(), 3);
        // needs more than asked first: distinct count below the list length
        let got = grow(3, take, |l| l.iter().filter(|&&x| x % 2 == 0).count() >= 3).unwrap();
        assert_eq!(got.len(), 6);
        // never enough: ends with all there is, after the request that brought nothing new
        let mut calls = 0;
        let counted = |q: usize| {
            calls += 1;
            take(q)
        };
        assert_eq!(grow(3, counted, |_| false).unwrap().len(), 10);
        assert_eq!(calls, 4);
    }

    #[test]
    fn a_long_list_starts_with_a_small_request_and_doubles() {
        let mut asked = Vec::new();
        let got = grow(
            1000,
            |q| {
                asked.push(q);
                Ok::<_, ()>(vec![0u8; q])
            },
            |l| l.len() >= 400,
        );
        assert_eq!(got.unwrap().len(), 400);
        assert_eq!(asked, [100, 200, 400]);
    }

    #[test]
    fn a_later_error_keeps_the_list_so_far_and_the_first_one_fails() {
        let mut calls = 0;
        let got = grow(
            2,
            |q| {
                calls += 1;
                if calls == 1 {
                    Ok(vec![0; q])
                } else {
                    Err("late")
                }
            },
            |_| false,
        );
        assert_eq!(got, Ok(vec![0, 0]));
        assert_eq!(grow::<u8, _>(2, |_| Err("first"), |_| false), Err("first"));
    }
}
