// CDDL Definitions
// 
// Contest Choice Payload V2 CDDL Specification
@extern(embed)

package cddl

cddlDefinitions: {
	"contest-ballot-checkpoint": {
		requires: [
			"smt-root",
			"smt-entries",
		]
		def: """
			{
				"\(requires[0])" : \(requires[0]),
				"\(requires[1])" : \(requires[1]),
			}
			"""
		description: """
			Catalyst Ballot Checkpoint data object.

			This serves as a checkpoint that collects new `contest-ballot-payload` documents
			that have been observed by a bulleting board.

			It will be created periodically during the voting period to allow proofs of inclusion
			to be firmly anchored and repeatably verifiable, and to allow voters or auditors to confirm
			a bulletin board acted honestly and included all valid ballots it detected.

			At another interval (which may be the same or different), a roll-up of the latest
			checkpoint is submitted to a blockchain to provide an immutable anchor of the 
			ballots collected by a bulletin board up to that point in time.
			"""
		comment: """
			Catalyst Ballot Checkpoint Payload data object.
			"""
		examples: [
			{
				title: "Example Catalyst Ballot Checkpoint Genesis Payload."
				description: """
					Example Shows:
					
					* First Block in a chain
					* No proposals collected in the interval.
					* The root hash is of an empty SMT.

					NOTE: This example is of the entire Signed Document structure including the Payload.
					"""
				example: _ @embed(file=examples/contest_ballot_checkpoint_genesis.cbor,type=binary)
			},
			{
				title: "Example Catalyst Ballot Checkpoint Intermediate Payload."
				description: """
					Example Shows:
					
					* Intermediate Block in a chain
					* Six proposals collected in the interval.
					* The root hash is the SMT including these new documents.

					NOTE: This example is of the entire Signed Document structure including the Payload.
					"""
				example: _ @embed(file=examples/contest_ballot_checkpoint_intermediate.cbor,type=binary)
			},
			{
				title: "Example Catalyst Ballot Checkpoint Final Payload."
				description: """
					Example Shows:
					
					* Final Block in a chain
					* Two proposals collected in the interval.
					* The root hash is the SMT including these new documents.

					NOTE: This example is of the entire Signed Document structure including the Payload.
					"""
				example: _ @embed(file=examples/contest_ballot_checkpoint_final.cbor,type=binary)
			},
		]
	}

	"smt-root": {
		requires: [
			"blake3",
		]
		def: "\(requires[0])"
		description: """
			The SMT Root hash is a Blake 3 256bit digest Hash.
			"""
		comment: """
			\(description)
			"""
	}

	"smt-entries": {
		def: "uint"
		description: """
			The Count of all Documents held by the SMT.
			"""
		comment: """
			\(description)
			"""
	}
	rejections: {
		def: "uint"
		description: """
			The Count of all Documents held by the SMT.
			"""
		comment: """
			\(description)
			"""
	}
	encrypted_tally: {
		def: "uint"
		description: """
			The Count of all Documents held by the SMT.
			"""
		comment: """
			\(description)
			"""
	}

}
