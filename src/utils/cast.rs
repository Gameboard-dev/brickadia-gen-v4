use std::any::type_name;
use num_traits::{NumCast, ToPrimitive};

/// Converts a numeric value between types. <br>
pub fn cast<S, T>(v: S) -> T 
where
    S: ToPrimitive,
    T: NumCast,
{
    NumCast::from(v).unwrap_or_else(|| {
        panic!(
            "Conversion Failed: {} -> {}",
            type_name::<S>(),
            type_name::<T>()
        )
    })
}

