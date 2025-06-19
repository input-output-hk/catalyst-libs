package signed_docs

docs: #DocumentDefinitions & {
	"Representative Category Profile": {
		description: """
			  ## Representative Category Profile Document

			  A Representative Category Profile is created to opt in as a Representative for a specific campaign category, the user must have registered as a Representative.
			  The presence of this docuemnt signifies the user's intent to participate in that category as a Representative.
			  
			  The document's structure is defined by the associated Representative_Category_Profile_Template, which allows an Admin to specify category-specific requirements.
			  
			  The payload must include a status field, indicating whether the Representative is currently active or has revoked their participation.
			"""
		validation: """
			  - The signer MUST be a registered 'Representative'.
			  - The 'ref' metadata field MUST point to a valid 'Representative_Profile' document.
			  - The 'parameters' metadata field MUST point to a valid 'Category Parameters' document.
			  - The 'template' metadata field MUST point to a valid 'Representative_Category_Profile_Template' document.
			  - The payload MUST be valid against the JSON schema defined in the referenced template.
			"""
		business_logic: {
			front_end: """
				  - Allows a Representative to create or update their profile for a category.
				  - The status is set to 'active' when created and the Representative is then discoverable for delegation.
				  - The Representative can opt-out and their status set to 'revoked' to signal they are no longer participating in the category.
				"""
			back_end: """
				  - The backend MUST verify the signer is a 'Representative' and that all referenced documents exist.
				  - The system will only consider Representatives with an 'active' status as eligible for delegation.
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
				  The Representative's profile data for a specific category. Its structure is defined by the referenced template document.
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
