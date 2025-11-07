// CDDL Definitions
// 
// Contest Choice Payload V2 CDDL Specification
@extern(embed)

package cddl

cddlDefinitions: {
	"contest-ballot-checkpoint": {
		requires: [
			"stage",
			"smt-root",
			"smt-entries",
			"rejections",
			"encrypted-tally",
			"tally",
			"drep-encryption-key",
		]
		def: """
			{
				"\(requires[0])" : \(requires[0])
				"\(requires[1])" : \(requires[1])
				"\(requires[2])" : \(requires[2])
				? "\(requires[3])" : \(requires[3])
				? "\(requires[4])" : \(requires[4])
				? "\(requires[5])" : \(requires[5])
				? "\(requires[6])" : \(requires[6])
			}
			"""
		description: """
			Catalyst Ballot Checkpoint data object.

			This serves as a checkpoint that collects new `contest-ballot-payload` documents
			that have been observed by a bulletin board.

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
	stage: {
		requires: []
		def: """
			(
				"bulletin-board" /
				"tally" /
				"audit"
			)
			"""
		description: """
			What stage in the ballot processing does this checkpoint represent.
			"""
		comment: """
			\(description)
			"""
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
		requires: [
			"rejection-reason",
			"document_ref",
		]
		def: """
			{
				+ \(requires[0]) => [ + \(requires[1]) ]
			}
			"""
		description: """
			For any documents that were rejected for a defined reason,
			the list of document references which were rejected.
			"""
		comment: """
			List of documents rejected at this checkpoint, grouped by reason.
			"""
	}

	"rejection-reason": {
		requires: []
		def: """
			(
				"already-voted" /  ; Used to indicate a voter already voted in another system (ie, Jormungandr)
				"obsolete-vote"    ; Used to indicate a vote that was cast was replaced with a newer vote.
			)
			"""
		description: """
			The reason a document was rejected at this checkpoint.
			"""
		comment: """
			\(description)
			"""
	}

	"encrypted-tally": {
		requires: [
			"document_ref",
			"encrypted-tally-proposal-result",
		]
		def: """
			{
				+ \(requires[0]) => \(requires[1])
			}
			"""
		description: """
			The Result of an encrypted tally.
			Placeholder until formally defined.
			"""
		comment: """
			Placeholder of encrypted tally result.
			"""
	}
	tally: {
		requires: [
			"document_ref",
			"tally-proposal-result",
		]
		def: """
			{
				+ \(requires[0]) => \(requires[1])
			}
			"""
		description: """
			The Result of an encrypted tally.
			Placeholder until formally defined.
			"""
		comment: """
			Placeholder of encrypted tally result.
			"""
	}

	"encrypted-tally-proposal-result": {
		def: "[ 1, undefined ]"
		description: """
			The Result of an encrypted tally.
			Placeholder until formally defined.
			"""
		comment: """
			Placeholder of encrypted tally result.
			"""
	}

	"tally-proposal-result": {
		requires: [
			"clear-choice",
			"voting-power",
		]
		def: "[ 0, { + \(requires[0]) : \(requires[1]) } ]"
		description: """
			The Result of an encrypted tally.
			Placeholder until formally defined.
			"""
		comment: """
			Placeholder of encrypted tally result.
			"""
	}

	"drep-encryption-key": {
		def: "undefined"
		description: """
			Placeholder to store the drep encryption key so drep votes can be decrypted if required.
			Placeholder until formally defined.
			"""
		comment: """
			Placeholder of drep encryption key.
			"""
	}

	"voting-power": {
		def: "int"
		description: """
			The Voting Power.
			Voting Power is always an integer, any fractional voting power 
			is represented as a fixed point, and not defined here.
			"""
		comment: """
			Voting Power.
			"""
	}
}
