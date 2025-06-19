package signed_docs

docs: #DocumentDefinitions & {
	"Profile": {
		description: """
			## Profile Document

			A minimal user profile that provides basic information about a user.
			Its structure is defined by the referenced Profile Template.
			It is used as a base for more specific profiles like the Representative Profile.
			"""
		validation: """
			* The signer must be a registered 'User'.
			* The payload must be valid against the JSON schema defined in the referenced 'Profile Template'.
			"""
		business_logic: {
			front_end: """
				* Display the user's profile information.
				* Allow a user to edit their own profile data.
				"""
			back_end: """
				* Validate and store profile data against the referenced 'Profile_Template'.
				* This profile serves as the base document for a user.
				  Its scope can be extended to create more specific profiles.
				"""
		}
		metadata: {
			template: {
				required: "yes"
				type:     "Profile Template"
			}
		}
		payload: {
			description: """
				The profile payload contains all base profile fields.
				Its structure is defined by the referenced Profile Template.
				"""
		}
		signers: {
			roles: {
				user: [
					"Registered",
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
