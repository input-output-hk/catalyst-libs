//! Voting primitives which are used among Catalyst ecosystem.
//!
//! ```rust
//! use catalyst_voting::{
//!     decrypt_tally, encrypt_vote, tally, DecryptionTallySetup, EncryptionRandomness, SecretKey,
//!     Vote,
//! };
//!
//! struct Voter {
//!     voting_power: u64,
//!     choice: usize,
//! }
//!
//! let mut rng = rand_core::OsRng;
//! let voting_options = 3;
//! let election_secret_key = SecretKey::generate(&mut rng);
//! let election_public_key = election_secret_key.public_key();
//!
//! let voter_1 = Voter {
//!     voting_power: 10,
//!     choice: 0,
//! };
//!
//! let voter_2 = Voter {
//!     voting_power: 20,
//!     choice: 1,
//! };
//!
//! let voter_3 = Voter {
//!     voting_power: 30,
//!     choice: 2,
//! };
//!
//! let vote_1 = Vote::new(voter_1.choice, voting_options).unwrap();
//! let vote_2 = Vote::new(voter_2.choice, voting_options).unwrap();
//! let vote_3 = Vote::new(voter_3.choice, voting_options).unwrap();
//!
//! let voter_1_randomness = EncryptionRandomness::generate(&mut rng, voting_options);
//! let voter_2_randomness = EncryptionRandomness::generate(&mut rng, voting_options);
//! let voter_3_randomness = EncryptionRandomness::generate(&mut rng, voting_options);
//!
//! let encrypted_vote_1 =
//!     encrypt_vote(&vote_1, &election_public_key, &voter_1_randomness).unwrap();
//! let encrypted_vote_2 =
//!     encrypt_vote(&vote_2, &election_public_key, &voter_2_randomness).unwrap();
//! let encrypted_vote_3 =
//!     encrypt_vote(&vote_3, &election_public_key, &voter_3_randomness).unwrap();
//! let encrypted_votes = vec![encrypted_vote_1, encrypted_vote_2, encrypted_vote_3];
//!
//! let encrypted_tallies: Vec<_> = (0..voting_options)
//!     .map(|voting_option| {
//!         tally(voting_option, &encrypted_votes, &[
//!             voter_1.voting_power,
//!             voter_2.voting_power,
//!             voter_3.voting_power,
//!         ])
//!         .unwrap()
//!     })
//!     .collect();
//!
//! let decryption_tally_setup = DecryptionTallySetup::new(
//!     voter_1.voting_power + voter_2.voting_power + voter_3.voting_power,
//! )
//! .unwrap();
//! let decrypted_tallies: Vec<_> = encrypted_tallies
//!     .iter()
//!     .map(|t| decrypt_tally(t, &election_secret_key, &decryption_tally_setup).unwrap())
//!     .collect();
//!
//! assert_eq!(decrypted_tallies, vec![
//!     voter_1.voting_power,
//!     voter_2.voting_power,
//!     voter_3.voting_power
//! ]);
//! ```

mod crypto;
pub mod tally;
pub mod voter;

pub use crypto::elgamal::{PublicKey, SecretKey};
pub use tally::{decrypt_tally, tally, DecryptionTallySetup};
pub use voter::{encrypt_vote, EncryptionRandomness, Vote};
