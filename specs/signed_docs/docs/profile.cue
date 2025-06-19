package signed_docs

docs: #DocumentDefinitions & {
	"Profile": {
		description: """
			## Profile Document

			A profile document for a Catalyst user containing basic user information.
			"""
		validation: """
			The profile must include both a name and a bio. No additional validation beyond schema and required fields.
			"""
		business_logic: {
			front_end: """
				Display and allow editing of profile fields for the user.
				"""
			back_end: """
				Validate profile data and store in the system.
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
				The profile payload contains the minimum profile information for a user. Its structure is defined by the referenced Profile Template.
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
