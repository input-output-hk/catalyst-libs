package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Proposal Moderation Action": {
		draft: true

		description: """
			A Moderation action performed on a Proposal.
			"""
		headers: "content type": value: "application/json"

		metadata: ref: {
			required: "yes"
			type:     "Proposal"
		}
		
		// TODO add more detailed description
		payload: description: "Comment moderation action payload"

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
