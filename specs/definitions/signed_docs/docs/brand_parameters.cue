package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Brand Parameters": {

		description: """
			Parameters which define this brand within the system.
			"""

		headers: "content type": value: "application/json"

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
