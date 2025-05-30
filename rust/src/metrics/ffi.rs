use std::{ffi::c_char, fmt::Debug, marker::PhantomData};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Metric},
    error::Fallible,
    ffi::{
        any::{AnyMetric, Downcast},
        util::{self, ExtrinsicObject, Type, c_bool, into_c_char_p, to_str},
    },
    metrics::{AbsoluteDistance, L1Distance, L2Distance},
    traits::{InfAdd, Number},
};

use super::{
    ChangeOneDistance, DiscreteDistance, HammingDistance, InsertDeleteDistance, L0PInfDistance,
    L01InfDistance, L02InfDistance, LInfDistance, SymmetricDistance,
};

#[bootstrap(
    name = "_metric_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___metric_free(this: *mut AnyMetric) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "_metric_equal",
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check whether two metrics are equal.
///
/// # Arguments
/// * `left` - Metric to compare.
/// * `right` - Metric to compare.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___metric_equal(
    left: *mut AnyMetric,
    right: *const AnyMetric,
) -> FfiResult<*mut c_bool> {
    let status = try_as_ref!(left) == try_as_ref!(right);
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "metric_debug",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Debug a `metric`.
///
/// # Arguments
/// * `this` - The metric to debug (stringify).
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__metric_debug(this: *mut AnyMetric) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(format!("{:?}", this))))
}

#[bootstrap(
    name = "metric_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the type of a `metric`.
///
/// # Arguments
/// * `this` - The metric to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__metric_type(this: *mut AnyMetric) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.type_.descriptor.to_string())))
}

#[bootstrap(
    name = "metric_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the distance type of a `metric`.
///
/// # Arguments
/// * `this` - The metric to retrieve the distance type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__metric_distance_type(
    this: *mut AnyMetric,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "symmetric_distance",
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `SymmetricDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__symmetric_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(SymmetricDistance)))
}

#[bootstrap(
    name = "insert_delete_distance",
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `InsertDeleteDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__insert_delete_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(InsertDeleteDistance)))
}

#[bootstrap(
    name = "change_one_distance",
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `ChangeOneDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__change_one_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(ChangeOneDistance::default())))
}

#[bootstrap(name = "hamming_distance", returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `HammingDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__hamming_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(HammingDistance)))
}

#[bootstrap(returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `AbsoluteDistance` metric.
///
/// # Arguments
/// * `T` - The type of the distance.
fn absolute_distance<T>() -> AbsoluteDistance<T> {
    AbsoluteDistance::default()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__absolute_distance(T: *const c_char) -> FfiResult<*mut AnyMetric> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMetric> {
        Ok(AnyMetric::new(absolute_distance::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `L1Distance` metric.
///
/// # Arguments
/// * `T` - The type of the distance.
fn l1_distance<T>() -> L1Distance<T> {
    L1Distance::default()
}
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__l1_distance(T: *const c_char) -> FfiResult<*mut AnyMetric> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMetric> {
        Ok(AnyMetric::new(l1_distance::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `L2Distance` metric.
///
/// # Arguments
/// * `T` - The type of the distance.
fn l2_distance<T>() -> L2Distance<T> {
    L2Distance::default()
}
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__l2_distance(T: *const c_char) -> FfiResult<*mut AnyMetric> {
    fn monomorphize<T: 'static>() -> FfiResult<*mut AnyMetric> {
        Ok(AnyMetric::new(l2_distance::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], ())
}

#[bootstrap(name = "discrete_distance", returns(c_type = "FfiResult<AnyMetric *>"))]
/// Construct an instance of the `DiscreteDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__discrete_distance() -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(DiscreteDistance)))
}

#[bootstrap(
    arguments(metric(c_type = "AnyMetric *", rust_type = b"null")),
    generics(M(suppress)),
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `L01InfDistance` metric.
///
/// # Arguments
/// * `metric` - The metric used to compute distance between partitions.
fn l01inf_distance<M: Metric>(metric: M) -> L01InfDistance<M> {
    L0PInfDistance(metric)
}
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__l01inf_distance(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyMetric> {
    let metric = try_as_ref!(metric);
    let M = metric.type_.clone();
    if M == Type::of::<SymmetricDistance>() {
        let metric = try_!(metric.downcast_ref::<SymmetricDistance>()).clone();
        return Ok(AnyMetric::new(l01inf_distance(metric))).into();
    }
    fn monomorphize<Q: Number>(metric: &AnyMetric) -> Fallible<AnyMetric> {
        let metric = metric.downcast_ref::<AbsoluteDistance<Q>>()?.clone();
        Ok(AnyMetric::new(l01inf_distance(metric))).into()
    }
    let Q = try_!(M.get_atom());
    dispatch!(monomorphize, [(Q, @numbers)], (metric)).into()
}

#[bootstrap(
    arguments(metric(c_type = "AnyMetric *", rust_type = b"null")),
    generics(M(suppress)),
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `L02InfDistance` metric.
///
/// # Arguments
/// * `metric` - The metric used to compute distance between partitions.
fn l02inf_distance<M: Metric>(metric: M) -> L02InfDistance<M> {
    L0PInfDistance(metric)
}
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__l02inf_distance(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyMetric> {
    let metric = try_as_ref!(metric);
    let M = metric.type_.clone();
    fn monomorphize<Q: Number>(metric: &AnyMetric) -> Fallible<AnyMetric> {
        let metric = metric.downcast_ref::<AbsoluteDistance<Q>>()?.clone();
        Ok(AnyMetric::new(l02inf_distance(metric))).into()
    }
    let Q = try_!(M.get_atom());
    dispatch!(monomorphize, [(Q, @numbers)], (metric)).into()
}

#[bootstrap(
    arguments(monotonic(default = false)),
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Construct an instance of the `LInfDistance` metric.
///
/// # Arguments
/// * `monotonic` - set to true if non-monotonicity implies infinite distance
///
/// # Generics
/// * `T` - The type of the distance.
fn linf_distance<T: InfAdd>(monotonic: bool) -> LInfDistance<T> {
    LInfDistance::new(monotonic)
}
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__linf_distance(
    monotonic: c_bool,
    T: *const c_char,
) -> FfiResult<*mut AnyMetric> {
    let monotonic = util::to_bool(monotonic);
    fn monomorphize<T: 'static + InfAdd>(monotonic: bool) -> FfiResult<*mut AnyMetric> {
        Ok(AnyMetric::new(linf_distance::<T>(monotonic))).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(
        monomorphize,
        [(T, [u32, u64, i32, i64, usize, f32, f64])],
        (monotonic)
    )
}
#[derive(Clone, Default)]
pub struct ExtrinsicDistance {
    pub descriptor: String,
}

impl std::fmt::Debug for ExtrinsicDistance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserDistance({:?})", self.descriptor)
    }
}

impl PartialEq for ExtrinsicDistance {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor
    }
}

impl Metric for ExtrinsicDistance {
    type Distance = ExtrinsicObject;
}

#[bootstrap(
    name = "user_distance",
    features("honest-but-curious"),
    arguments(descriptor(rust_type = "String"))
)]
/// Construct a new UserDistance.
/// Any two instances of an UserDistance are equal if their string descriptors are equal.
///
/// # Arguments
/// * `descriptor` - A string description of the metric.
///
/// # Why honest-but-curious?
/// Your definition of `d` must satisfy the requirements of a pseudo-metric:
///
/// 1. for any $x$, $d(x, x) = 0$
/// 2. for any $x, y$, $d(x, y) \ge 0$ (non-negativity)
/// 3. for any $x, y$, $d(x, y) = d(y, x)$ (symmetry)
/// 4. for any $x, y, z$, $d(x, z) \le d(x, y) + d(y, z)$ (triangle inequality)
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__user_distance(
    descriptor: *mut c_char,
) -> FfiResult<*mut AnyMetric> {
    let descriptor = try_!(to_str(descriptor)).to_string();
    Ok(AnyMetric::new(ExtrinsicDistance { descriptor })).into()
}

pub struct TypedMetric<Q> {
    metric: AnyMetric,
    marker: PhantomData<fn() -> Q>,
}

impl<Q: 'static> TypedMetric<Q> {
    pub fn new(metric: AnyMetric) -> Fallible<TypedMetric<Q>> {
        if metric.distance_type != Type::of::<Q>() {
            return fallible!(
                FFI,
                "unexpected distance type in metric. Expected {}, got {}",
                Type::of::<Q>().to_string(),
                metric.distance_type.to_string()
            );
        }

        Ok(TypedMetric {
            metric,
            marker: PhantomData,
        })
    }
}

impl<Q> PartialEq for TypedMetric<Q> {
    fn eq(&self, other: &Self) -> bool {
        self.metric == other.metric
    }
}

impl<Q> Clone for TypedMetric<Q> {
    fn clone(&self) -> Self {
        Self {
            metric: self.metric.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<Q> Debug for TypedMetric<Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.metric)
    }
}
impl<Q> Default for TypedMetric<Q> {
    fn default() -> Self {
        panic!()
    }
}

impl<Q> Metric for TypedMetric<Q> {
    type Distance = Q;
}
