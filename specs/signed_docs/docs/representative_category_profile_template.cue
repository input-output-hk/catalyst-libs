@extern(embed)

package signed_docs

docs: #DocumentDefinitions & {
	"Representative Category Profile Template": {
		description: """
			  ## Representative Category Profile Template Document

			  Defines the allowed payload contents and constraints for a Representative's category-specific profile.
			  This template is created by an Admin to enforce a consistent structure for all Representatives within a given category.
			"""
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
