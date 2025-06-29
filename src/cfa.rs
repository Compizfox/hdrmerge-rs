/// Returns the indices in flat pixel array of the i-th 2x2 (CFA) block.
///
/// The indices are numbered left-to-right, top-to-bottom, like:
/// ```
/// 1 2
/// 3 4
/// ```
///
/// * `width`: Row width of the pixel array
/// * `i`: Block index
pub fn block_to_indices(width: usize, i: usize) -> [usize; 4] {
    let row_floored = 2 * i / width;
    let row_i = row_floored * width;
    [
        row_i         + 2 * i, row_i         + 2 * i + 1,
        row_i + width + 2 * i, row_i + width + 2 * i + 1,
    ]
}

/// Returns true if one of the pixels in the block is saturated.
///
/// * `data`: Pixel array
/// * `block`: Array of indices
/// * `wl`: White level indicating saturation
pub fn is_saturated(data: &Vec<u16>, block: [usize; 4], wl: u32) -> bool {
    block
        .iter()
        .map(|&i| data[i])
        .any(|x| x as u32 >= wl)
}

#[cfg(test)]
mod tests {
    #[test]
    fn block_to_indices() {
        let array: [usize; 6 * 4] = core::array::from_fn(|i| i);
        let width = 6;

        assert_eq!(crate::cfa::block_to_indices(width, 0), [0, 1, 6, 7]);
        assert_eq!(crate::cfa::block_to_indices(width, 1), [2, 3, 8, 9]);
        assert_eq!(crate::cfa::block_to_indices(width, 3), [12, 13, 18, 19]);
    }
}