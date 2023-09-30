use std::slice::{
    from_raw_parts,
    from_raw_parts_mut,
};

pub fn split<T, const N: usize>(bytes: &[T]) -> Option<(&[T; N], &[T])> {
    match bytes.len() >= N {
        true => {
            let (array, remainder) = bytes.split_at(N);
            Option::Some((unsafe { &*(array.as_ptr().cast()) }, remainder))
        },
        false => Option::None,
    }
}

pub fn split_mut<T, const N: usize>(bytes: &mut [T]) -> Option<(&mut [T; N], &mut [T])> {
    match bytes.len() >= N {
        true => {
            let (array, remainder) = bytes.split_at_mut(N);
            Option::Some((unsafe { &mut *(array.as_mut_ptr().cast()) }, remainder))
        },
        false => Option::None,
    }
}

pub fn as_array<T, const N: usize>(bytes: &[T]) -> Option<&[T; N]> {
    match bytes.len() == N {
        true => Option::Some(unsafe { &*(bytes.as_ptr().cast()) }),
        false => Option::None,
    }
}

pub fn as_array_mut<T, const N: usize>(bytes: &mut [T]) -> Option<&mut [T; N]> {
    match bytes.len() == N {
        true => Option::Some(unsafe { &mut *(bytes.as_mut_ptr().cast()) }),
        false => Option::None,
    }
}

pub fn as_chunks<T, const N: usize>(bytes: &[T], count: usize) -> Option<&[[T; N]]> {
    match bytes.len() == N * count {
        true => Option::Some(unsafe { from_raw_parts(bytes.as_ptr().cast(), count) }),
        false => Option::None,
    }
}

pub fn as_chunks_mut<T, const N: usize>(bytes: &mut [T], count: usize) -> Option<&mut [[T; N]]> {
    match bytes.len() == N * count {
        true => Option::Some(unsafe { from_raw_parts_mut(bytes.as_mut_ptr().cast(), count) }),
        false => Option::None,
    }
}
