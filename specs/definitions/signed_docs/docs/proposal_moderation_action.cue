package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal Moderation Action": {

		description: """
			A Moderation action performed on a Proposal.
			"""

		metadata: ref: {
			required: "yes"
			type:     "Proposal"
		}

		versions: [
			{
				version:  "0.01"
				modified: "2025-04-04"
				changes: """
					* First Published Version
					"""
			},
		]}
}
