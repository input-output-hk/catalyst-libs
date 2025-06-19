@extern(embed)

package signed_docs

docs: #DocumentDefinitions & {
	"Representative Category Profile Template": {
		description: "## Representative Category Profile Template\nDefines the JSON schema for a 'Representative Category Profile'.\nThis allows an 'Admin' to specify different profile requirements for each category."
		validation: """
			* The signer MUST be a registered 'Admin'.
			* The payload MUST be a valid JSON schema.
			* The schema MUST include a 'status' field.
			"""
		business_logic: {
			front_end: """
				"""
			back_end: """
				* Validate and store the JSON schema that defines the structure for 'Representative Category Profile' documents.
				"""
		}
		metadata: {
			// Add any template-specific metadata here if needed
		}
		headers: {
			"content type": {
				value: "application/schema+json"
			}
		}
		payload: {
			description: """
				JSON Schema document which defines the valid contents of a Representative Category Profile document.
				The schema MUST include a 'status' field to indicate if the Representative is active or withdrawn from the category.
				"""
			schema: _ @embed(file="payload_schemas/representative_category_profile_template.schema.json")
		}
		signers: {
			roles: {
				admin: [
					"Brand Admin",
					"Campaign Admin",
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
