package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Comment Moderation Action": {
		draft: true

		description: """
			A Moderation Action performed on any Comment.
			"""
		headers: "content type": value: "application/json"

		metadata: ref: {
			required: "yes"
			type: ["Proposal Comment"]
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
