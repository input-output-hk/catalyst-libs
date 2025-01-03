//! An implementation of the voting protocol described in this [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/)
//!
//! ```rust
//! use catalyst_voting::vote_protocol::{
//!     committee::ElectionSecretKey,
//!     tally::{
//!         decrypt_tally,
//!         proof::{generate_tally_proof_with_default_rng, verify_tally_proof},
//!         tally, DecryptionTallySetup,
//!     },
//!     voter::{
//!         encrypt_vote_with_default_rng,
//!         proof::{
//!             generate_voter_proof_with_default_rng, verify_voter_proof, VoterProofCommitment,
//!         },
//!         Vote,
//!     },
//! };
//!
//! struct Voter {
//!     voting_power: u64,
//!     choice: usize,
//! }
//!
//! // Initial setup
//! let voting_options = 3;
//! let election_secret_key = ElectionSecretKey::random_with_default_rng();
//! let election_public_key = election_secret_key.public_key();
//! let voter_proof_commitment = VoterProofCommitment::random_with_default_rng();
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
//! // Generating votes
//! let vote_1 = Vote::new(voter_1.choice, voting_options).unwrap();
//! let vote_2 = Vote::new(voter_2.choice, voting_options).unwrap();
//! let vote_3 = Vote::new(voter_3.choice, voting_options).unwrap();
//!
//! let (encrypted_vote_1, voter_randomness_1) =
//!     encrypt_vote_with_default_rng(&vote_1, &election_public_key);
//! let (encrypted_vote_2, voter_randomness_2) =
//!     encrypt_vote_with_default_rng(&vote_2, &election_public_key);
//! let (encrypted_vote_3, voter_randomness_3) =
//!     encrypt_vote_with_default_rng(&vote_3, &election_public_key);
//!
//! // Verify encrypted votes
//! {
//!     let voter_proof_1 = generate_voter_proof_with_default_rng(
//!         &vote_1,
//!         encrypted_vote_1.clone(),
//!         voter_randomness_1,
//!         &election_public_key,
//!         &voter_proof_commitment,
//!     )
//!     .unwrap();
//!     assert!(verify_voter_proof(
//!         encrypted_vote_1.clone(),
//!         &election_public_key,
//!         &voter_proof_commitment,
//!         &voter_proof_1
//!     ));
//!
//!     let voter_proof_2 = generate_voter_proof_with_default_rng(
//!         &vote_2,
//!         encrypted_vote_2.clone(),
//!         voter_randomness_2,
//!         &election_public_key,
//!         &voter_proof_commitment,
//!     )
//!     .unwrap();
//!     assert!(verify_voter_proof(
//!         encrypted_vote_2.clone(),
//!         &election_public_key,
//!         &voter_proof_commitment,
//!         &voter_proof_2
//!     ));
//!
//!     let voter_proof_3 = generate_voter_proof_with_default_rng(
//!         &vote_3,
//!         encrypted_vote_3.clone(),
//!         voter_randomness_3,
//!         &election_public_key,
//!         &voter_proof_commitment,
//!     )
//!     .unwrap();
//!     assert!(verify_voter_proof(
//!         encrypted_vote_3.clone(),
//!         &election_public_key,
//!         &voter_proof_commitment,
//!         &voter_proof_3
//!     ));
//! }
//!
//! // Tally step
//! let encrypted_votes = [encrypted_vote_1, encrypted_vote_2, encrypted_vote_3];
//! let voting_powers = [
//!     voter_1.voting_power,
//!     voter_2.voting_power,
//!     voter_3.voting_power,
//! ];
//! let encrypted_tallies: Vec<_> = (0..voting_options)
//!     .map(|voting_option| tally(voting_option, &encrypted_votes, &voting_powers).unwrap())
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
//! // Verify tallies
//! {
//!     let tally_proofs: Vec<_> = encrypted_tallies
//!         .iter()
//!         .map(|t| generate_tally_proof_with_default_rng(t, &election_secret_key))
//!         .collect();
//!
//!     let is_ok = tally_proofs
//!         .iter()
//!         .zip(encrypted_tallies.iter())
//!         .zip(decrypted_tallies.iter())
//!         .all(|((p, enc_t), t)| verify_tally_proof(enc_t, *t, &election_public_key, p));
//!     assert!(is_ok);
//! }
//!
//! assert_eq!(decrypted_tallies, vec![
//!     voter_1.voting_power,
//!     voter_2.voting_power,
//!     voter_3.voting_power
//! ]);
//! ```

pub mod committee;
pub mod tally;
pub mod voter;
