use core::ops::Range;

pub fn split_range(a: Range<usize>) -> (Range<usize>, Range<usize>) {
    let Range { start, end } = a;
    let mid = (end - start + 1) / 2 + start;
    (start..mid, mid..end)
}

pub fn split_slice<T>(s: &[T]) -> (&[T], &[T]) {
    s.split_at((s.len() + 1) / 2)
}

pub fn split_tuple_as_range(start: usize, end: usize) -> ((usize, usize), (usize, usize)) {
    let (ra, rb) = split_range(Range { start, end });
    ((ra.start, ra.end), (rb.start, rb.end))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn splitters() {
        let tocheck: &[&[u8]] = &[
            &[0][..],
            &[0, 1][..],
            &[0, 1, 2][..],
            &[0, 1, 2, 3][..],
            &[0, 1, 2, 3, 4][..],
            &[0, 1, 2, 3, 4, 5][..],
        ];
        let expected: &[(&[u8], &[u8])] = &[
            (&[0][..], &[][..]),
            (&[0][..], &[1][..]),
            (&[0, 1][..], &[2][..]),
            (&[0, 1][..], &[2, 3][..]),
            (&[0, 1, 2][..], &[3, 4][..]),
            (&[0, 1, 2][..], &[3, 4, 5][..]),
        ];

        for t in tocheck {
            let (ra, rb) = split_range(0..t.len());
            let sr = (&t[ra], &t[rb]);
            let ss = split_slice(t);
            assert_eq!(sr, ss);
        }

        for t in tocheck {
            let (ra, rb) = split_range(0..t.len());
            let (ta, tb) = split_tuple_as_range(0, t.len());
            assert_eq!((ra, rb), (ta.0..ta.1, tb.0..tb.1));
        }

        assert_eq!(tocheck.len(), expected.len());
        for (t, e) in tocheck.iter().zip(expected) {
            assert_eq!(&split_slice(t), e);
        }
    }
}
