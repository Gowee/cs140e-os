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

fn panic_if_not_power_of_two(number: usize) {
    if !number.is_power_of_two() {
        panic!("Wrong alignment size: {}! It can only be a power of two.", number);
    }
}

pub fn log2(number: usize) -> usize {
    if number == 0 {
        panic!("attempt to calculate logarithm with zero");
    }
    let mut number = number;
    let mut log = 0;
    while {
        number = number >> 1;
        number > 0
    } {
        log += 1;
    }
    log
}

pub fn log2_up(number: usize) -> usize {
    let log = log2(number);
    if number.is_power_of_two() {
        log
    }
    else {
        log + 1 // round up
    }
}
