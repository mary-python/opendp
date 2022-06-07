#[cfg(feature = "ffi")]
mod ffi;

mod int;
pub use int::*;

mod float;
pub use float::*;

use crate::core::Transformation;
use crate::dist::{AbsoluteDistance, InsertDeleteDistance, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{CheckNull, TotalOrd};
use crate::trans::make_cast_metric;

// make_(sized_)?bounded_sum
// make_(sized_)?bounded_int_(ordered|split|checked|modular)_sum
// make_(sized_)?bounded_float_(sequential|pairwise|kahan|rtz)_sum

const DEFAULT_SIZE_LIMIT: usize = 65536;

type BoundedSumTrans<T> = Transformation<
    VectorDomain<BoundedDomain<T>>,
    AllDomain<T>,
    SymmetricDistance,
    AbsoluteDistance<T>,
>;

pub trait MakeBoundedSum: Sized + CheckNull + Clone + TotalOrd {
    fn make_bounded_sum(bounds: (Self, Self)) -> Fallible<BoundedSumTrans<Self>>;
}

pub fn make_bounded_sum<T: MakeBoundedSum>(bounds: (T, T)) -> Fallible<BoundedSumTrans<T>> {
    T::make_bounded_sum(bounds)
}

macro_rules! impl_make_bounded_sum_int {
    ($($ty:ty)+) => ($(impl MakeBoundedSum for $ty {
        fn make_bounded_sum(bounds: (Self, Self)) -> Fallible<BoundedSumTrans<Self>> {
            make_bounded_int_split_sum(bounds)
        }
    })+);
}
impl_make_bounded_sum_int! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }

macro_rules! impl_make_bounded_sum_float {
    ($($ty:ty)+) => ($(impl MakeBoundedSum for $ty {
        fn make_bounded_sum(bounds: (Self, Self)) -> Fallible<BoundedSumTrans<Self>> {
            let domain = VectorDomain::new(BoundedDomain::new_closed(bounds.clone())?);
            (
                make_cast_metric::<_, SymmetricDistance, InsertDeleteDistance>(domain)? >>
                make_bounded_float_sequential_sum(DEFAULT_SIZE_LIMIT, bounds)?
            )
        }
    })+);
}
impl_make_bounded_sum_float! { f32 f64 }

type SizedBoundedSumTrans<T> = Transformation<
    SizedDomain<VectorDomain<BoundedDomain<T>>>,
    AllDomain<T>,
    SymmetricDistance,
    AbsoluteDistance<T>,
>;
pub trait MakeSizedBoundedSum: Sized + CheckNull + Clone + TotalOrd {
    fn make_sized_bounded_sum(
        size: usize,
        bounds: (Self, Self),
    ) -> Fallible<SizedBoundedSumTrans<Self>>;
}

macro_rules! impl_make_sized_bounded_sum_int {
    ($($ty:ty)+) => ($(impl MakeSizedBoundedSum for $ty {
        fn make_sized_bounded_sum(size: usize, bounds: (Self, Self)) -> Fallible<SizedBoundedSumTrans<Self>> {
            make_sized_bounded_int_split_sum(size, bounds)
        }
    })+);
}
impl_make_sized_bounded_sum_int! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }
macro_rules! impl_make_sized_bounded_sum_float {
    ($($ty:ty)+) => ($(impl MakeSizedBoundedSum for $ty {
        fn make_sized_bounded_sum(size: usize, bounds: (Self, Self)) -> Fallible<SizedBoundedSumTrans<Self>> {
            let domain = SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds.clone())?), size);
            make_cast_metric(domain)? >> make_sized_bounded_float_sequential_sum(size, bounds)?
        }
    })+);
}
impl_make_sized_bounded_sum_float! { f32 f64 }

pub fn make_sized_bounded_sum<T: MakeSizedBoundedSum>(
    size: usize,
    bounds: (T, T),
) -> Fallible<SizedBoundedSumTrans<T>> {
    T::make_sized_bounded_sum(size, bounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_bounded_sum_l1() {
        let transformation = make_bounded_sum::<i32>((0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l2() {
        let transformation = make_bounded_sum::<i32>((0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_n() {
        let transformation = make_sized_bounded_sum::<i32>(5, (0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }
}
