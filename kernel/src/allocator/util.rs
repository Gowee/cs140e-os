/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    panic_if_not_power_of_two(align);
    (addr / align) * align
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    match align_down(addr, align) {
        aligned if aligned == addr => aligned,
        aligned => aligned + align
    }
}

fn is_power_of_two(number: usize) -> bool {
    number & (number - 1) == 0
}

fn panic_if_not_power_of_two(number: usize) {
    if !is_power_of_two(number) {
        panic!("Wrong alignment size: {}! It can only be a power of two.", number);
    }
}
