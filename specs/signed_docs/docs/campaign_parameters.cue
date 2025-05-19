package signed_docs

// Campaign Parameters Document Definition
docs: #DocumentDefinitions & {
	"Campaign Parameters": {
		description: """
			Parameters which define a Campaign within a Brand in the system.
			"""
		metadata: {
			parameters: {
				required: "yes"
				type:     "Brand Parameters"
				validation: """
					In addition to the validation performed for `Document Reference` type fields: 
					* Any linked referenced document that includes a `parameters` metadata must match the 
					  `parameters` of the referencing document.
				"""
			}
			collaborators: {
				required: "optional"
				validation: """
					This list does not imply these collaborators have consented to collaborate, only that the author/s
					are permitting these potential collaborators to participate in the drafting and submission process.
					However, any document submission referencing a proposal MUST be signed by all collaborators in
					addition to the author.
				"""
			}
		}
		payload: {
			schema: _ @embed(file="payload_schemas/campaign_parameters.schema.json")
			// Updated to reference the Campaign type from signed_doc.json
			type: "Campaign"
		}
		versions: [
			{
				version:  "0.01"
				modified: "2025-04-04"
				changes: """
					* First Published Version
				"""
			},
			{
				version:  "0.03"
				modified: "2025-05-05"
				changes: """
					* Use generalized parameters.
				"""
			},
		]
	}
}
