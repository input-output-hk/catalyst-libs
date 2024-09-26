//! Utulity functions and entities

/// A convinient macro to autogenerate a `std::ops` traits implmentation as `Add`, `Sub`
/// etc.
///
/// It is required to provide a basic implementation of the `std::ops` for reference
/// types, and then you can call `std_ops_gen` to provide a the remaining implementation
/// E.g.
/// ```
/// struct YourType;
/// impl Mul<&YourType> for &YourType {
/// type Output = YourType;

/// fn mul(self, other: &YourType) -> YourType {
///    unimplemented!()
/// }
/// }
///
/// std_ops_gen!(Add, add, YourType, YourType, YourType);
/// ```
macro_rules! std_ops_gen {
    ($class: ident,  $f: ident, $rty: ident, $lty: ident, $out: ident) => {
        impl $class<$rty> for &$lty {
            type Output = $out;

            fn $f(self, other: $rty) -> Self::Output {
                self.$f(&other)
            }
        }

        impl $class<&$rty> for $lty {
            type Output = $out;

            fn $f(self, other: &$rty) -> Self::Output {
                (&self).$f(other)
            }
        }

        impl $class<$rty> for $lty {
            type Output = $out;

            fn $f(self, other: $rty) -> Self::Output {
                (&self).$f(&other)
            }
        }
    };
}

pub(crate) use std_ops_gen;
