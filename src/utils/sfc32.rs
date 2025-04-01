/// SFC32 generates a pseudo-random number generator (PRNG) using Small Fast Counter (sfc32).
/// It's portable and can be translated easily into other languages.
/// It operates on mutable u32 seeds (`a`, `b`, `c`, `d`) and returns a closure.
/// The closure returns pseudo-random numbers in the range `[0, 1)` on each call.
pub fn sfc32(mut a: u32, mut b: u32, mut c: u32, mut d: u32) -> impl FnMut() -> f64 {
    // `move ||` creates a closure that captures and takes ownership of variables by value 
    move || {
        // Addition, wrapping around on integer overflow.
        let t = a.wrapping_add(b).wrapping_add(d);
        d = d.wrapping_add(1);
        // `b` is bitwise right-shifted, equivalent to performing integer division with 2^9
        // The bitwise XOR operation with a ^= b is equivalent to the following:
        // For pairs of bits: return 1 if only one is 1, 0 if both are 0 or both are 1
        a ^= b >> 9;
        // The operation (b << 9) shifts the bits of b left by 9 positions. Bits that "fall off" the left end are discarded.
        // This is equivalent to performing integer division between `b` and 2^9.
        // ^= performs XOR between `a` and (b << 9).
        // The result in each position is 1 if only one of the two bits is 1, but will be 0 if both are 0 or 1
        b = b.wrapping_add(c).wrapping_add(c << 3);
        // The operation (c << 21) shifts the bits of c left by 21 positions. Bits that "fall off" the left end are discarded.
        // The operation (c >> 11) shifts the bits of c right by 11 positions. Bits that "fall off" the right end are discarded.
        // To achieve a rotation, the bits that "fall off" one end are wrapped around to the other end with `|`.
        c = (c << 21) | (c >> 11);
        // Addition, wrapping around on integer overflow.
        c = c.wrapping_add(t);
        // The number 4294967296.0 is chosen for normalization because it is equal to 2^32, which is the number of possible values of u32bits.
        // The result is a pseudo-random number in the range [0, 1).
        // SFC32 returns a closure which returns a different pseudo-random number on each call.
        (t as f64) / 4294967296.0
    }
}


/// Generates a random integer in the range `[min, max)` using a PRNG closure.
pub fn random_range<F>(mut sfc: F, min: f32, max: f32) -> usize
where
    F: FnMut() -> f64,
{
    let range = max - min;
    let rand_value = sfc() as f32 * range;
    (min + rand_value) as usize
}
