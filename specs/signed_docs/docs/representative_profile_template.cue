@extern(embed)

package signed_docs

docs: #DocumentDefinitions & {
	"Representative Profile Template": {
		description: "## Representative Profile Template\n\nDefines the JSON schema for a 'Representative Profile'.\nThis template allows an 'Admin' to enforce a specific structure and set of constraints for Representative profiles."
		validation: """
			* The signer MUST be a registered 'Admin'.
			* The payload MUST be a valid JSON schema.
			"""
		business_logic: {
			front_end: """
				"""
			back_end: """
				* Validate and store the JSON schema that defines the structure for all 'Representative Profile' documents.
				* The schema MUST extend the base 'Profile' schema with Representative-specific fields.
				"""
		}
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
