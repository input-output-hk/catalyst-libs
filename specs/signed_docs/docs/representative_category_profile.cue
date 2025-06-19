package signed_docs

docs: #DocumentDefinitions & {
	"Representative Category Profile": {
		description: "## Representative Category Profile Document\n\nA Representative Category Profile is created to opt in as a Representative for a specific campaign category, the user must have registered as a Representative.\nThe presence of this docuemnt signifies the user's intent to participate in that category as a Representative.\n\nThe document's structure is defined by the associated Representative_Category_Profile_Template, which allows an Admin to specify category-specific requirements.\n\nThe payload must contain a 'status' field to indicate if the Representative is active or has revoked their participation."
		validation: """
				* The signer MUST be a registered 'Representative'.
				* The 'ref' metadata field MUST point to a valid 'Representative Profile' document.
				* The 'parameters' metadata field MUST point to a valid 'Category Parameters' document.
				* The 'template' metadata field MUST point to a valid 'Representative Category Profile Template' document.
				* The payload MUST be valid against the JSON schema defined in the referenced template.
			"""
		business_logic: {
			front_end: """
					* Allows a Representative to create or update their profile for a category.
					* The Representative sets their status to 'active' to be discoverable for delegation.
					* The Representative can set their status to 'revoked' to signal they are no longer participating in the category,
					  without having to revoke the document.
				"""
			back_end: """
					* The backend MUST verify the signer is a 'Representative' and that all referenced documents exist.
					* The system will only consider Representatives with an 'active' status as eligible for delegation.
				"""
		}
		metadata: {
			ref: {
				required: "yes"
				type:     "Representative Profile"
			}
			parameters: {
				required: "yes"
				type:     "Category Parameters"
			}
			template: {
				required: "yes"
				type:     "Representative Category Profile Template"
			}
		}
		payload: {
			description: """
					The Representative's profile data for a specific category.
					Its structure is defined by the referenced template document.
					It MUST contain a 'status' field ('active' or 'revoked') to manage the Representative's participation.
				"""
		}
		signers: {
			roles: {
				user: [
					"Representative",
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
