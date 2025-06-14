package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Comment Moderation Action": {
		description: """
			A Moderation Action performed on any Comment.
			"""

		metadata: ref: {
			required: "yes"
			type: ["Proposal Comment"]
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
