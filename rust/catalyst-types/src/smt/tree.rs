//! The wrapper for the Sparse Merkle Tree which extends it's functionality by providing
//! `coarse_height()` and `horizontal_slice_at()` methods.

use std::{collections::HashMap, marker::PhantomData};

use sparse_merkle_tree::{
    BranchKey, BranchNode, SparseMerkleTree,
    default_store::DefaultStore,
    merge::{self},
};
pub use sparse_merkle_tree::{H256, MerkleProof};

use crate::smt::{
    Error, Value, error,
    hasher::Hasher,
    utils::{self, MAX_COARSE_HEIGHT, ROOT_HEIGHT},
    value::ValueWrapper,
};

/// Sparse Merkle Tree
pub struct Tree<V> {
    /// Internal implementation of a Sparse Merkle Tree
    inner: SparseMerkleTree<Hasher, ValueWrapper, DefaultStore<ValueWrapper>>,
    /// Phantom data
    phantom: PhantomData<V>,
}

impl<V> Tree<V>
where V: Default + Value + Clone
{
    /// Creates new, empty tree
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: SparseMerkleTree::default(),
            phantom: PhantomData,
        }
    }

    /// Merkle root
    #[must_use]
    pub fn root(&self) -> &H256 {
        self.inner.root()
    }

    /// Check empty of the tree
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Insert value, returns its key.
    ///
    /// # Errors
    ///
    /// Errors coming from the internal implementation of the sparse merkle tree.
    pub fn insert(
        &mut self,
        value: &V,
    ) -> Result<H256, error::MerkleTreeError> {
        let key = Hasher::hash(&value.to_bytes());
        let value = ValueWrapper(value.to_bytes());
        let _ = self.inner.update(key, value)?;
        Ok(key)
    }

    /// Retrieves value from the tree
    ///
    /// # Errors
    ///
    /// Errors coming from the internal implementation of the sparse merkle tree.
    pub fn get(
        &self,
        key: &H256,
    ) -> Result<Option<V>, error::MerkleTreeError> {
        let value = self.inner.get(key)?;
        if value == <ValueWrapper as sparse_merkle_tree::traits::Value>::zero() {
            Ok(None)
        } else {
            Ok(Some(V::from_bytes(&value.0)))
        }
    }

    /// Generate merkle proof
    ///
    /// # Errors
    ///
    /// Errors coming from the internal implementation of the sparse merkle tree.
    pub fn merkle_proof(
        &self,
        keys: &[H256],
    ) -> Result<MerkleProof, error::MerkleTreeError> {
        let proof = self.inner.merkle_proof(keys.to_vec())?;
        Ok(proof)
    }

    /// Number of leaves
    #[must_use]
    pub fn count(&self) -> usize {
        self.inner.store().leaves_map().len()
    }

    /// Returns an appropriate coarse height for grouping Merkle tree nodes into batches.
    #[must_use]
    pub fn coarse_height(&self) -> u8 {
        let count = self.count();
        utils::coarse_height(count)
    }

    /// Returns an iterator over the horizontal slice of the tree at a specified height.
    ///
    /// This function retrieves all node positions (both materialized and virtual) at a
    /// given height in the Merkle tree. The iterator yields hashes for materialized
    /// nodes and None for virtual zero branches.
    ///
    /// # Height Convention
    ///
    /// Note: This function uses the standard tree convention where height 0 is the root
    /// and height increases toward the leaves.
    ///
    /// # Errors
    ///
    /// Errors related to calculating the horizontal slice.
    pub fn horizontal_slice_at(
        &self,
        height: u8,
    ) -> Result<impl Iterator<Item = Result<Option<H256>, Error>>, Error> {
        let store = self.inner.store();

        horizontal_slice::<_, Hasher>(store, height)
    }
}

impl<V> Default for Tree<V>
where V: Default + Value + Clone
{
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a complete horizontal slice of the Sparse Merkle Tree at a specified height.
///
/// This function creates an iterator representing all possible node positions at a given
/// height, whether those nodes are materialized in storage or not.
///
/// # Sparse Merkle Tree Structure
///
/// In an SMT, each height has a theoretical maximum number of nodes:
/// - Height 0 (root): 1 node (2^0)
/// - Height 1: 2 nodes (2^1)
/// - Height 2: 4 nodes (2^2)
/// - Height h: 2^h nodes
///
/// However, only nodes on paths to actual leaf values are materialized. This function
/// reconstructs the complete horizontal slice by:
/// 1. Computing all theoretical node positions at the target height
/// 2. Looking up which positions have materialized nodes in storage
/// 3. For materialized nodes: computing their hash by merging left/right children
/// 4. For non-materialized nodes: yielding `None` (representing implicit zero branches)
///
/// # Algorithm
///
/// 1. Calculate `key_prefix_length` = path length from root to target height
/// 2. Calculate `width` = `2^key_prefix_length` (total possible nodes at this height)
/// 3. Retrieve all materialized nodes at target height from store
/// 4. Lazily iterate through each horizontal position (0 to width-1):
///    - Generate the `node_key` for that position
///    - If node is materialized: compute its hash from left/right children
///    - If node is not materialized: yield None (virtual zero branch)
///
/// # Returns
///
/// An iterator yielding `Option<H256>` for each position in the horizontal slice:
/// - `Some(hash)` indicates a materialized node with the computed hash
/// - `None` indicates a virtual zero branch (non-materialized)
///
/// The iterator will yield `2^(ROOT_HEIGHT - target_height)` items.
///
/// # Performance Note
///
/// Returns an iterator to avoid allocating `2^key_prefix_length` elements upfront.
/// For heights near the leaves (high values), the theoretical width can be astronomically
/// large. The iterator allows processing nodes on-demand or early termination.
fn horizontal_slice<V, H>(
    store: &DefaultStore<V>,
    target_height: u8,
) -> Result<impl Iterator<Item = Result<Option<H256>, Error>>, Error>
where
    V: Clone,
    H: sparse_merkle_tree::traits::Hasher + Default,
{
    if target_height > MAX_COARSE_HEIGHT {
        return Err(Error::SliceHeightTooLarge {
            allowed_max: MAX_COARSE_HEIGHT,
        });
    }

    // In `sparse_merkle_tree` crate the heights are inverted (root is located at height 255).
    let inverted_height =
        ROOT_HEIGHT
            .checked_sub(target_height)
            .ok_or(Error::SliceHeightTooLarge {
                allowed_max: MAX_COARSE_HEIGHT,
            })?;

    // Total number of nodes at this height.
    let width = 2_u32.pow(u32::from(target_height));

    // Retrieve all materialized nodes at this height.
    // SMT nodes are materialized lazily: only nodes on paths to non-empty leaves.
    // Thus at any height, stored nodes <= theoretical maximum for that level.
    let materialized_nodes: HashMap<_, _> =
        materialized_nodes_at_height(store, inverted_height).collect();

    // For a given height we need to iterate through `width` number of positions,
    // even if some nodes are not materialized.
    Ok((0..width).map(move |horizontal_position| {
        let node_key = utils::node_key(target_height, horizontal_position)?;
        let key = BranchKey::new(inverted_height, node_key);

        Ok(materialized_nodes.get(&node_key).map(|node| {
            // Recreate the node identity using the provided hasher.
            let mv = merge::merge::<H>(key.height, &key.node_key, &node.left, &node.right);
            mv.hash::<H>()
        }))
    }))
}

/// Returns an iterator over all materialized nodes at a specific height in the Sparse
/// Merkle Tree.
///
/// # Sparse Merkle Tree Materialization
///
/// In a Sparse Merkle Tree (SMT), not all nodes are explicitly stored (materialized) in
/// memory. Most nodes in the tree are implicitly zero/empty, and only nodes that are on
/// the path to actual leaf values are materialized and stored. This sparse representation
/// is what makes SMTs practical for large key spaces - a full binary tree of height 256
/// would require 2^256 nodes, which is impossible to store.
///
/// For example, in a tree with only 3 leaves:
/// - Height 0 (root): 1 materialized node (the root)
/// - Height 1: 2 materialized nodes (left and right children of root)
/// - Height 2: May have 2-3 materialized nodes (only branches leading to leaves)
/// - Heights 3-255: Sparse - only nodes on paths to the 3 leaves are materialized
///
/// This function filters the stored branches to find only those at the requested height,
/// allowing to examine a horizontal slice of the actually-stored tree structure.
fn materialized_nodes_at_height<V>(
    store: &DefaultStore<V>,
    target_height: u8,
) -> impl Iterator<Item = (&H256, &BranchNode)> {
    store
        .branches_map()
        .iter()
        .filter_map(move |(branch_key, node)| {
            (branch_key.height == target_height).then_some((&branch_key.node_key, node))
        })
}

#[cfg(test)]
mod tests {
    use sparse_merkle_tree::H256;
    use test_case::test_case;

    use crate::smt::{Tree, Value};

    #[derive(Default, Debug, Clone, PartialEq)]
    struct IntValue(i32);

    impl Value for IntValue {
        fn to_bytes(&self) -> Vec<u8> {
            self.0.to_be_bytes().into_iter().collect()
        }

        fn from_bytes(bytes: &[u8]) -> Self {
            IntValue(i32::from_be_bytes(bytes.try_into().unwrap()))
        }
    }

    #[test]
    fn update_with_distinct_values_changes_root_hash() {
        let mut smt = Tree::new();

        let mut root_hashes = vec![];

        for i in 0..=10 {
            let current_root = *smt.root();
            assert!(!root_hashes.contains(&current_root));
            root_hashes.push(current_root);
            smt.insert(&IntValue(i)).expect("should insert");
        }
    }

    #[test]
    fn update_with_same_value_does_not_change_root_hash() {
        let mut smt = Tree::new();
        smt.insert(&IntValue(1)).expect("should insert");
        let prev_root = *smt.root();
        smt.insert(&IntValue(1)).expect("should insert");
        let current_root = *smt.root();
        assert_eq!(prev_root, current_root);
    }

    #[test]
    fn can_retrieve_values() {
        let mut smt = Tree::new();
        let key = smt.insert(&IntValue(1)).expect("should insert");

        let existing = smt
            .get(&key)
            .expect("should retrieve")
            .expect("should exist");

        assert_eq!(existing, IntValue(1));

        let non_existing_key = H256::zero();
        let non_existing = smt.get(&non_existing_key).expect("should retrieve");
        assert!(non_existing.is_none());
    }

    #[test_case(0 => 1)]
    #[test_case(1 => 2)]
    #[test_case(2 => 4)]
    #[test_case(7 => 128)]
    #[test_case(10 => 1024)]
    fn horizontal_slice_has_expected_length(height: u8) -> usize {
        let smt = Tree::<IntValue>::new();
        let hashes = smt
            .horizontal_slice_at(height)
            .expect("should get a slice")
            .collect::<Result<Vec<_>, _>>()
            .expect("should get a slice");
        hashes.len()
    }

    #[test]
    fn horizontal_slice_at_0_equals_to_root() {
        let mut smt = Tree::new();
        let _key = smt.insert(&IntValue(1)).expect("should insert");

        let root = smt.root();
        let node_at_0 = smt
            .horizontal_slice_at(0)
            .expect("should get a slice")
            .next()
            .expect("should have at least one value")
            .expect("should retrieve the value")
            .expect("value should exist");

        assert_eq!(*root, node_at_0);
    }
}
