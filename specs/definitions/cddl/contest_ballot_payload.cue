// CDDL Definitions
// 
// Contest Choice Payload V2 CDDL Specification
@extern(embed)

package cddl

cddlDefinitions: {
	"contest-ballot-payload": {
		requires: [
			"choices",
			"column-proof",
			"matrix-proof",
			"voter-choice",
		]
		def: """
			{
				+ uint => \(requires[0])
				? "\(requires[1])" : \(requires[1])
				? "\(requires[2])" : \(requires[2])
				? "\(requires[3])" : \(requires[3])
			}
			"""
		description: """
			Catalyst Vote Payload data object.

			A vote payload that can hold both encrypted or unencrypted votes.
			"""
		comment: """
			Catalyst Vote Payload data object.
			"""
		examples: [
			{
				title: "Example Encrypted Contest Ballot Payload."
				description: """
					Example Shows:
					
					* Three Proposals
					* Two Encrypted Choices
					* Row Proofs for each proposal.
					* `aes-ctr-encrypted-choices` which reflects the choices.

					The Contest Private Key was: 0x1234562343....
					The Contest Public Key was: 0x1324354235...
					The AES encryption key for the `aes-ctr-encrypted-choices` is 0x123456789...
					"""
				example: _ @embed(file=examples/contest_ballot_payload_encrypted.cbor,type=binary)
			},
			{
				title: "Example Clear Ballot Payload."
				description: """
					Example Shows:
					
					* Three Proposals
					* Two Choices
					"""
				example: _ @embed(file=examples/contest_ballot_payload_clear.cbor,type=binary)
			},
		]
	}

	choices: {
		requires: [
			"clear-choices",
			"elgamal-ristretto255-encrypted-choices",
		]
		def: """
			[ 0, \(requires[0]) ] /
			[ 1, \(requires[1]) ]
			"""
		description: """
			Choices are an array of encrypted  or unencrypted choices.
			"""
		comment: """
			Voters Choices.
			"""
	}

	"clear-choices": {
		requires: [
			"clear-choice",
		]
		def: """
			( +clear-choice )
			"""
		description: """
			A Choice Selection (clear/unencrypted).

			This can be a positive or negative integer, and is
			constrained by the parameters of the contest.
			"""
		comment: """
			Universal Unencrypted Set of Choices
			"""
	}

	"clear-choice": {
		requires: []
		def: """
			uint
			"""
		description: """
			An Choice Selection (clear/unencrypted).

			This is a positive integer, and is constrained
			by the parameters of the contest.
			"""
		comment: """
			Universal Unencrypted Choice
			"""
	}

	"elgamal-ristretto255-encrypted-choices": {
		requires: [
			"elgamal-ristretto255-encrypted-choice",
			"row-proof",
		]
		def: """
			[ 
			  [+ \(requires[0])]
			  ? \(requires[1]) 
			]
			"""
		description: """
			Encrypted Choices are a Vector (list) of encrypted items.
			The size of the vector will depend on the cryptography used, 
			and the number of choices.

			Typically, (but optionally) it has a proof attached which proves something
			about the encrypted choices, without disclosing their contents.

			For example, a ZKProof that there is only a single `1` in the choices, 
			and all the rest are `0`.

			The size/contents of the proof depend on what is being proved, and the 
			cryptography underlying the proof.
			"""
		comment: """
			elgamal/ristretto255 Encrypted Choices
			"""
	}

	"elgamal-ristretto255-encrypted-choice": {
		requires: [
			"elgamal-ristretto255-group-element",
		]
		def: """
			[
			  c1: \(requires[0])
			  c2: \(requires[0])
			]
			"""
		description: """
			The elgamal encrypted ciphertext `(c1, c2)`.
			"""
		comment: """
			\(description)
			"""
	}

	"elgamal-ristretto255-group-element": {
		requires: []
		def: """
			bytes .size 32
			"""
		description: """
			An individual Elgamal group element that composes the elgamal cipher text.
			"""
		comment: """
			\(description)
			"""
	}

	"row-proof": {
		requires: [
			"zkproof-elgamal-ristretto255-unit-vector-with-single-selection",
		]
		def: """
			[0, \(requires[0]) ]
			"""
		description: """
			A proof that the choices conform to a required set of properties.
			It is defined by the configured cryptography used for encrypted choices.
			This format is universal over all encrypted choice encoding.
			"""
		comment: """
			Universal Encrypted Row Proof
			"""
	}

	"zkproof-elgamal-ristretto255-unit-vector-with-single-selection": {
		requires: [
			"zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item",
			"zkproof-ed25519-scalar",
		]
		def: """
			[ +\(requires[0]), \(requires[1]) ]
			"""
		description: """
			Proof that the choices form a unit vector with a single selection.
			"""
	}

	"zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item": {
		requires: [
			"zkproof-elgamal-announcement",
			"elgamal-ristretto255-encrypted-choice",
			"zkproof-ed25519-r-response",
		]
		def: """
			( \(requires[0]), ~\(requires[1]), \(requires[2]) )
			"""
		description: """
			Proof that the row is a unit vector with a single selection.
			"""
		comment: """
			\(description)
			"""
	}

	"zkproof-elgamal-announcement": {
		requires: ["zkproof-elgamal-group-element"]
		def: """
			( \(requires[0]), \(requires[0]), \(requires[0]) )
			"""
		description: """
			ZK Proof Announcement values for Elgamal.
			"""
		comment: """
			\(description)
			"""
	}

	"zkproof-elgamal-group-element": {
		requires: []
		def: """
			bytes .size 32
			"""
		description: """
			An individual Elgamal group element used in ZK Proofs.
			"""
		comment: """
			\(description)
			"""
	}

	"zkproof-ed25519-r-response": {
		requires: ["zkproof-ed25519-scalar"]
		def: """
			( \(requires[0]), \(requires[0]), \(requires[0]) )
			"""
		description: """
			ZK Proof Response values for Ed25519.
			"""
		comment: """
			\(description)
			"""
	}

	"zkproof-ed25519-scalar": {
		requires: []
		def: """
			bytes .size 32
			"""
		description: """
			An individual Ed25519 scalar used in ZK Proofs.
			"""
		comment: """
			\(description)
			"""
	}

	"column-proof": {
		requires: []
		def: """
			[ uint, [ +undefined ] ]
			"""
		description: """
			Proof that values in a column have a required arrangement.
			This is similar to the `row-proof` but for all values in a 
			single column.
			It is an array that matches the length of `choices`.
			If it is a different length, then it is invalid.

			Currently there are no `column-proof` defined, this value is
			a placeholder for documentation purposes only.

			It is NOT to be implemented.
			`column-proof` should be assumed to be missing until such time
			as a concrete `column-proof` is defined.

			Similar to `row-proof` there can be multiple column-proofs defined which prove
			certain characteristics of the encrypted column values.
			They are identified by the unsigned integer starting the proof.
			"""
		comment: """
			Universal Encrypted Column Proof (Placeholder)
			"""
	}

	"matrix-proof": {
		requires: []
		def: """
			[ uint, undefined ]
			"""
		description: """
			Proof that values in the matrix of all columns and rows have a required arrangement.
			This is similar to the `row-proof` and `column-proof` but for all values in a 
			ballot taken together.

			There is a single `matrix-proof` but it may be chosen from a pre-defined
			known set of valid proofs.

			Currently there are no `matrix-proof` defined, this value is
			a placeholder for documentation purposes only.

			It is NOT to be implemented.
			`matrix-proof` should be assumed to be missing until such time
			as a concrete `matrix-proof` is defined.

			Similar to `row-proof` and `column-proof` there can be multiple matrix-proofs defined 
			which prove certain characteristics of the encrypted column values.
			They are identified by the unsigned integer starting the proof.
			"""
		comment: """
			Universal Encrypted Matrix Proof (Placeholder)
			"""
	}

	"voter-choice": {
		requires: ["aes-ctr-encrypted-choices"]
		def: """
			[ 0, \(requires[0]) ]
			"""
		description: """
			This is an encrypted payload that a voter, and ONLY the voter can decrypt.
			It allows the voter to recover their choices without needing to decrypt the
			encrypted votes used in the tally.

			There is no way to associate this data with the encrypted choices directly, but
			it is created by the voter from the same data used to create the choices.
			"""
		comment: """
			Encrypted Voter Choice Payload
			"""
	}

	"aes-ctr-encrypted-choices": {
		requires: ["aes-ctr-encrypted-block"]
		def: """
			+\(requires[0])
			"""
		description: """
			Choices are constructed as a CBOR multidimensional array of the form:
			`[ +[+choice] ]`
			reflecting the choices in the rows and columns as present 1:1 in the encrypted
			choices.

			This data is then compressed using `brotli` compression, and the result is encrypted 
			using AES-CTR and encoded as a sequence of blocks here.

			Data needs to be pre-compressed before encryption as encryption will make the data
			incompressible.

			The Encryption Key is to be derived from the Voters catalyst key-chain and not to be
			published.
			Derivation *MUST* include the contest Document ID and Version, so that the same
			encryption key is never used twice for different contests, but can still be re-derived
			by a voter that holds their catalyst key-chain recovery keys.
			"""
		comment: """
			Encrypted Voter Choices
			"""
	}

	"aes-ctr-encrypted-block": {
		requires: []
		def: """
			bytes .size 16
			"""
		description: """
			AES-CTR encrypted data.
			The Nonce/IV is the UUIDv7 `document_ver`.
			This is the correct size, and has the necessary randomness properties.
			The first block uses the `document_ver` the second `document_ver+1` and so on.
			The document_ver is interpreted as a Big Endian 128bit integer for the purpose
			of the addition.

			As the CTR is predictable, the blocks can be decrypted in parallel for maximum performance.
			"""
		comment: """
			AES-CTR Encrypted Data Block
			"""
	}

}
