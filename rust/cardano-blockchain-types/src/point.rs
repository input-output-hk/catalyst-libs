//! A Cardano Point on the Blockchain.
//!
//! Wrapped version of the Pallas primitive.
//! We only use this version unless talking to Pallas.

use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
};

use pallas::crypto::hash::Hash;

use crate::{
    hashes::{Blake2b256Hash, Blake2bHash},
    Slot,
};

/// A specific point in the blockchain. It can be used to
/// identify a particular location within the blockchain, such as the tip (the
/// most recent block) or any other block. It has special kinds of `Point`,
/// available as constants: `TIP_POINT`, and `ORIGIN_POINT`.
///
/// # Attributes
///
/// * `Point` - The inner type is a `Point` from the `pallas::network::miniprotocols`
///   module. This inner `Point` type encapsulates the specific details required to
///   identify a point in the blockchain.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Point(pallas::network::miniprotocols::Point);

impl Point {
    /// The origin of the blockchain. It is used when the
    /// interest is in the very first point of the blockchain, regardless of its
    /// specific details.
    ///
    /// # Usage
    ///
    /// `ORIGIN_POINT` can be used in scenarios where the starting point of the
    /// blockchain is required. It signifies the genesis block or the initial state
    /// of the blockchain.
    ///
    /// The inner `Point` is created with the `Origin` variant from
    /// `pallas::network::miniprotocols::Point`, indicating that this is a marker
    /// for the blockchain's origin.
    pub const ORIGIN: Point = Point(pallas::network::miniprotocols::Point::Origin);
    /// The tip of the blockchain at the current moment.
    /// It is used when the specific point in the blockchain is not known, but the
    /// interest is in the most recent block (the tip). The tip is the point where
    /// new blocks are being added.
    ///
    /// # Usage
    ///
    /// `TIP_POINT` can be used in scenarios where the most up-to-date point in the
    /// blockchain is required. It signifies that the exact point is not important
    /// as long as it is the latest available point in the chain.
    ///
    /// The inner `Point` is created with `u64::MAX` and an empty `Vec<u8>`, indicating
    /// that this is a special marker rather than a specific point in the blockchain.
    pub const TIP: Point = Point(pallas::network::miniprotocols::Point::Specific(
        u64::MAX,
        Vec::new(),
    ));
    /// A truly unknown point in the blockchain. It is used
    /// when the previous point is completely unknown and does not correspond to the
    /// origin of the blockchain.
    ///
    /// # Usage
    ///
    /// `UNKNOWN_POINT` can be used in scenarios where the previous point in the
    /// blockchain is not known and should not be assumed to be the origin. It serves
    /// as a marker for an indeterminate or unspecified point.
    ///
    /// The inner `Point` is created with `u64::MIN` and an empty `Vec<u8>`, indicating
    /// that this is a special marker for an unknown point, rather than a specific
    /// point in the blockchain.
    pub const UNKNOWN: Point = Point(pallas::network::miniprotocols::Point::Specific(
        u64::MIN,
        Vec::new(),
    ));

    /// Creates a new `Point` instance representing a specific
    /// point in the blockchain, identified by a given slot and hash.
    ///
    /// # Parameters
    ///
    /// * `slot` - A `Slot` representing the slot number in the blockchain.
    /// * `hash` - A `Blake2b256Hash` , block hash at the specified slot.
    ///
    /// # Returns
    ///
    /// A new `Point` instance encapsulating the given slot and hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Point;
    ///
    /// let slot = 42;
    /// let hash = [0; 32];
    /// let point = Point::new(slot.into(), hash.into());
    /// ```
    #[must_use]
    pub fn new(slot: Slot, hash: Blake2b256Hash) -> Self {
        Self(pallas::network::miniprotocols::Point::Specific(
            slot.into(),
            hash.into(),
        ))
    }

    /// Creates a new `Point` instance representing a specific
    /// point in the blockchain, identified by a given slot, but with an
    /// unknown hash. This can be useful in scenarios where the slot is known
    /// but the hash is either unavailable or irrelevant.
    ///
    /// # Parameters
    ///
    /// * `slot` - A `Slot` representing the slot number in the blockchain.
    ///
    /// # Returns
    ///
    /// A new `Point` instance encapsulating the given slot with an empty hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Point;
    ///
    /// let slot = 42;
    /// let point = Point::fuzzy(slot.into());
    /// ```
    #[must_use]
    pub fn fuzzy(slot: Slot) -> Self {
        Self(pallas::network::miniprotocols::Point::Specific(
            slot.into(),
            Vec::new(),
        ))
    }

    /// Creates a new Fuzzy `Point` from a concrete point.
    ///
    /// Will not alter either TIP or ORIGIN points.
    #[must_use]
    pub fn as_fuzzy(&self) -> Self {
        if *self == Self::TIP {
            Self::TIP
        } else {
            match self.0 {
                pallas::network::miniprotocols::Point::Specific(slot, _) => {
                    Self::fuzzy(slot.into())
                },
                pallas::network::miniprotocols::Point::Origin => Self::ORIGIN,
            }
        }
    }

    /// Check if a Point is Fuzzy.
    ///
    /// Even though we don't know the hash for TIP or Origin, neither of these points
    /// are considered to be fuzzy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Point;
    ///
    /// let slot = 42;
    /// let point = Point::fuzzy(slot.into());
    ///
    /// assert!(point.is_fuzzy());
    /// ```
    #[must_use]
    pub fn is_fuzzy(&self) -> bool {
        if *self == Self::TIP {
            false
        } else {
            match self.0 {
                pallas::network::miniprotocols::Point::Specific(_, ref hash) => hash.is_empty(),
                pallas::network::miniprotocols::Point::Origin => false,
            }
        }
    }

    /// Check if a Point is the origin.
    ///
    /// Origin is the synthetic Origin point, and ALSO any point thats at slot zero with a
    /// hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Point;
    ///
    /// let slot = 42;
    /// let point = Point::fuzzy(slot.into());
    ///
    /// assert!(!point.is_origin());
    /// ```
    #[must_use]
    pub fn is_origin(&self) -> bool {
        match self.0 {
            pallas::network::miniprotocols::Point::Specific(slot, ref hash) => {
                slot == 0 && !hash.is_empty()
            },
            pallas::network::miniprotocols::Point::Origin => true,
        }
    }

    /// Check if a Point is actually unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Point;
    ///
    /// let point = Point::fuzzy(0.into());
    ///
    /// assert!(point.is_unknown());
    /// ```
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        match self.0 {
            pallas::network::miniprotocols::Point::Specific(slot, ref hash) => {
                slot == 0 && hash.is_empty()
            },
            pallas::network::miniprotocols::Point::Origin => false,
        }
    }

    /// Check if a Point is the tip.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Point;
    ///
    /// let point = Point::fuzzy(0.into());
    ///
    /// assert!(!point.is_tip());
    /// ```
    #[must_use]
    pub fn is_tip(&self) -> bool {
        match self.0 {
            pallas::network::miniprotocols::Point::Specific(slot, ref hash) => {
                slot == u64::MAX && hash.is_empty()
            },
            pallas::network::miniprotocols::Point::Origin => false,
        }
    }

    /// Retrieves the slot number from the `Point`. If the `Point`
    /// is the origin, it returns a default slot number.
    ///
    /// # Returns
    ///
    /// A `u64` representing the slot number. If the `Point` is the origin,
    /// it returns a default slot value, typically `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Point;
    ///
    /// let specific_point = Point::new(42.into(), [0; 32].into());
    /// assert_eq!(specific_point.slot_or_default(), 42.into());
    ///
    /// let origin_point = Point::ORIGIN;
    /// assert_eq!(origin_point.slot_or_default(), 0.into()); // assuming 0 is the default
    /// ```
    #[must_use]
    pub fn slot_or_default(&self) -> Slot {
        self.0.slot_or_default().into()
    }

    /// Retrieves the hash from the `Point`. If the `Point` is
    /// the origin, it returns `None`.
    ///
    /// # Returns
    ///
    /// A `Blake2b256Hash` representing the hash. If the `Point` is the `Origin`, it
    /// returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::{Point, hashes::Blake2bHash};
    ///
    /// let specific_point = Point::new(42.into(), [0; 32].into());
    /// assert_eq!(specific_point.hash_or_default(), Some(Blake2bHash::new(&[0; 32])));
    ///
    /// let origin_point = Point::ORIGIN;
    /// assert_eq!(origin_point.hash_or_default(), None);
    /// ```
    #[must_use]
    pub fn hash_or_default(&self) -> Option<Blake2b256Hash> {
        match &self.0 {
            pallas::network::miniprotocols::Point::Specific(_, hash) => {
                Some(Blake2bHash::new(hash))
            },
            // Origin has empty hash, so set it to None
            pallas::network::miniprotocols::Point::Origin => None,
        }
    }

    /// Checks if two `Point` instances are strictly equal.
    /// Strict equality means both the slot number and hash must be identical.
    ///
    /// # Parameters
    ///
    /// * `b` - Another `Point` instance to compare against.
    ///
    /// # Returns
    ///
    /// A `bool` indicating whether the two `Point` instances are strictly equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Point;
    ///
    /// let point1 = Point::new(42.into(), [0; 32].into());
    /// let point2 = Point::new(42.into(), [0; 32].into());
    /// assert!(point1.strict_eq(&point2));
    ///
    /// let point3 = Point::new(42.into(), [0; 32].into());
    /// let point4 = Point::new(43.into(), [0; 32].into());
    /// assert!(!point3.strict_eq(&point4));
    /// ```
    #[must_use]
    pub fn strict_eq(&self, b: &Self) -> bool {
        self.0 == b.0
    }
}

impl PartialEq<Option<Blake2b256Hash>> for Point {
    /// Compares the hash stored in the `Point` with a `Blake2b256Hash`.
    /// It returns `true` if the hashes match and `false` otherwise. If the
    /// provided hash is `None`, the function checks if the `Point` has an
    /// empty hash.
    fn eq(&self, other: &Option<Blake2b256Hash>) -> bool {
        match other {
            Some(cmp_hash) => {
                match self.0 {
                    pallas::network::miniprotocols::Point::Specific(_, ref hash) => {
                        // Compare vec to vec
                        *hash == <Blake2b256Hash as Into<Vec<u8>>>::into(*cmp_hash)
                    },
                    pallas::network::miniprotocols::Point::Origin => false,
                }
            },
            None => {
                match self.0 {
                    pallas::network::miniprotocols::Point::Specific(_, ref hash) => hash.is_empty(),
                    pallas::network::miniprotocols::Point::Origin => true,
                }
            },
        }
    }
}

impl PartialEq<Option<Hash<32>>> for Point {
    /// Compares the hash stored in the `Point` with a Pallas `Hash`.
    /// It returns `true` if the hashes match and `false` otherwise. If the
    /// provided hash is `None`, the function checks if the `Point` has an
    /// empty hash.
    fn eq(&self, other: &Option<Hash<32>>) -> bool {
        match other {
            Some(cmp_hash) => {
                match self.0 {
                    pallas::network::miniprotocols::Point::Specific(_, ref hash) => {
                        **hash == **cmp_hash
                    },
                    pallas::network::miniprotocols::Point::Origin => false,
                }
            },
            None => {
                match self.0 {
                    pallas::network::miniprotocols::Point::Specific(_, ref hash) => hash.is_empty(),
                    pallas::network::miniprotocols::Point::Origin => true,
                }
            },
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if *self == Self::ORIGIN {
            return write!(f, "Point @ Origin");
        } else if *self == Self::TIP {
            return write!(f, "Point @ Tip");
        } else if *self == Self::UNKNOWN {
            return write!(f, "Point @ Unknown");
        }

        let slot = self.slot_or_default();
        let hash = self.hash_or_default();
        match hash {
            Some(hash) => {
                write!(
                    f,
                    "Point @ {slot:?}:{}",
                    hex::encode(<Blake2b256Hash as Into<Vec<u8>>>::into(hash))
                )
            },
            None => write!(f, "Point @ {slot:?}"),
        }
    }
}

impl From<pallas::network::miniprotocols::Point> for Point {
    fn from(point: pallas::network::miniprotocols::Point) -> Self {
        Self(point)
    }
}

impl From<Point> for pallas::network::miniprotocols::Point {
    fn from(point: Point) -> pallas::network::miniprotocols::Point {
        point.0
    }
}

impl PartialOrd for Point {
    /// Implements a partial ordering based on the slot number
    /// of two `Point` instances. It only checks the slot number for ordering.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    /// Implements a total ordering based on the slot number
    /// of two `Point` instances. It only checks the slot number for ordering.
    fn cmp(&self, other: &Self) -> Ordering {
        cmp_point(&self.0, &other.0)
    }
}

impl PartialEq<u64> for Point {
    fn eq(&self, other: &u64) -> bool {
        self.0.slot_or_default() == *other
    }
}

impl PartialOrd<u64> for Point {
    /// Allows to compare a `Point` against a `u64`
    fn partial_cmp(&self, other: &u64) -> Option<Ordering> {
        self.0.slot_or_default().partial_cmp(other)
    }
}

impl PartialEq<Option<Point>> for Point {
    /// Allows for direct comparison between a `Point` and an `Option<Point>`,
    /// returning `true` only if the `Option` contains a `Point` that is equal to the
    /// `self` instance.
    fn eq(&self, other: &Option<Point>) -> bool {
        if let Some(other) = other {
            *self == *other
        } else {
            false
        }
    }
}

impl PartialOrd<Option<Point>> for Point {
    /// Allows comparing a `Point` with an `Option<Point>`, where a `Point` is always
    /// considered greater than `None`.
    fn partial_cmp(&self, other: &Option<Point>) -> Option<Ordering> {
        if let Some(other) = other {
            self.partial_cmp(other)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl Default for Point {
    /// Returns the default value for `Point`, which is `UNKNOWN_POINT`.
    fn default() -> Self {
        Self::UNKNOWN
    }
}

/// Compare Points, because Pallas does not impl `Ord` for Point.
fn cmp_point(
    a: &pallas::network::miniprotocols::Point, b: &pallas::network::miniprotocols::Point,
) -> Ordering {
    match a {
        pallas::network::miniprotocols::Point::Origin => {
            match b {
                pallas::network::miniprotocols::Point::Origin => Ordering::Equal,
                pallas::network::miniprotocols::Point::Specific(..) => Ordering::Less,
            }
        },
        pallas::network::miniprotocols::Point::Specific(slot, _) => {
            match b {
                pallas::network::miniprotocols::Point::Origin => Ordering::Greater,
                pallas::network::miniprotocols::Point::Specific(other_slot, _) => {
                    slot.cmp(other_slot)
                },
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::point::*;

    #[test]
    fn test_cmp_hash_simple() {
        let origin1 = Point::ORIGIN;
        let point1 = Point::new(100u64.into(), [8; 32].into());

        assert!(origin1 != Some(Blake2bHash::<32>::new(&[0; 32])));
        assert!(origin1 == None::<Blake2b256Hash>);
        assert!(point1 == Some(Hash::<32>::new([8; 32])));
        assert!(point1 != None::<Hash<32>>);
    }

    #[test]
    fn test_get_hash_simple() {
        let point1 = Point::new(100u64.into(), [8; 32].into());

        assert_eq!(point1.hash_or_default(), Some([8; 32].into()));
    }

    #[test]
    fn test_identical_compare() {
        let point1 = Point::new(100u64.into(), [8; 32].into());
        let point2 = Point::new(100u64.into(), [8; 32].into());
        let point3 = Point::new(999u64.into(), [8; 32].into());

        assert!(point1.strict_eq(&point2));
        assert!(!point1.strict_eq(&point3));
    }

    #[test]
    fn test_comparisons() {
        let origin1 = Point::ORIGIN;
        let origin2 = Point::ORIGIN;
        let tip1 = Point::TIP;
        let tip2 = Point::TIP;
        let early_block = Point::new(100u64.into(), Blake2bHash::new(&[]));
        let late_block1 = Point::new(5000u64.into(), Blake2bHash::new(&[]));
        let late_block2 = Point::new(5000u64.into(), Blake2bHash::new(&[]));

        assert!(origin1 == origin2);
        assert!(origin1 < early_block);
        assert!(origin1 <= early_block);
        assert!(origin1 != early_block);
        assert!(origin1 < late_block1);
        assert!(origin1 <= late_block1);
        assert!(origin1 != late_block1);
        assert!(origin1 < tip1);
        assert!(origin1 <= tip1);
        assert!(origin1 != tip1);

        assert!(tip1 > origin1);
        assert!(tip1 >= origin1);
        assert!(tip1 != origin1);
        assert!(tip1 > early_block);
        assert!(tip1 >= late_block1);
        assert!(tip1 != late_block1);
        assert!(tip1 == tip2);

        assert!(early_block > origin1);
        assert!(early_block >= origin1);
        assert!(early_block != origin1);
        assert!(early_block < late_block1);
        assert!(early_block <= late_block1);
        assert!(early_block != late_block1);
        assert!(early_block < tip1);
        assert!(early_block <= tip1);
        assert!(early_block != tip1);

        assert!(late_block1 == late_block2);
        assert!(late_block1 > origin1);
        assert!(late_block1 >= origin1);
        assert!(late_block1 != origin1);
        assert!(late_block1 > early_block);
        assert!(late_block1 >= early_block);
        assert!(late_block1 != early_block);
        assert!(late_block1 < tip1);
        assert!(late_block1 <= tip1);
        assert!(late_block1 != tip1);
    }

    #[test]
    fn test_create_point_and_fuzzy_not_equal() {
        let point1 = Point::new(100u64.into(), Blake2bHash::new(&[]));
        let fuzzy1 = Point::fuzzy(100u64.into());

        assert!(point1 != fuzzy1);
    }
}
