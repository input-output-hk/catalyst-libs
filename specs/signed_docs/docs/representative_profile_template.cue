@extern(embed)

package signed_docs

docs: #DocumentDefinitions & {
	"Representative Profile Template": {
		description: """
			  ## Representative Profile Template Document

			  Defines the allowed payload contents and constraints for a Representative profile.
			"""
		metadata: {
			// Add any template-specific metadata here if needed
		}
		payload: {
			description: """
				JSON Schema document which defines the valid contents of a Representative profile document.
				"""
			schema: _ @embed(file="payload_schemas/representative_profile_template.schema.json")
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
