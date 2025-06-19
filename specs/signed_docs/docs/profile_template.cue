@extern(embed)

package signed_docs

docs: #DocumentDefinitions & {
	"Profile Template": {
		description: """
			## Profile Template Document

			Defines the allowed payload contents and constraints for a generic user profile.
			"""
		metadata: {
			// Add any template-specific metadata here if needed
		}
		payload: {
			description: """
				JSON Schema document which defines the valid contents of a profile document.
				"""
			schema: _ @embed(file="payload_schemas/profile_template.schema.json")
		}
		signers: {
			roles: {
				admin: [
					"Brand Admin",
				]
			}
		}
		authors: {
			"Neil McAuliffe": "neil.mcauliffe@iohk.io"
		}
		versions: [
			{
				version:  "0.01"
				modified: "2025-06-19"
				changes: """
						* First Published Version
					"""
			},
		]
	}
}
