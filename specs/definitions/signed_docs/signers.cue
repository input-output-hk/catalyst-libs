// Signed Document Definitions
// 
// Metadata Types and Constraints
package signed_docs

import "list"

// TODO: Get Roles from RBAC definition configuration package instead.
// Named User Roles
_allUserRolesList: list.UniqueItems
_allUserRolesList: [
	"Registered",     // Role 0 - A registered User / Voter - Base Role
	"Proposer",       // Registered for posting proposals
	"Representative", // Registered as a rep for voting purposes.
]
_allUserRoles: or(_allUserRolesList)

// Individual Valid User Role Constraint
#UserRoles: _allUserRoles
#allowedUserRoles: [...#UserRoles]

// TODO: Get roles from RBAC definition configuration package instead
// Named Admin Roles
_allAdminRolesList: list.UniqueItems
_allAdminRolesList: [
	"Root CA",
	"Brand CA",
	"Campaign CA",
	"Category CA",
	"Root Admin",
	"Brand Admin",
	"Campaign Admin",
	"Category Admin",
	"Moderator",
	"Bulletin Board Operator",
]
_allAdminRoles: or(_allAdminRolesList)

// Individual Valid Admin Role Constraint
#AdminRoles: _allAdminRoles
#allowedAdminRoles: [...#AdminRoles]

// The roles which are allowed to publish this document
#allowedRoles: {
	// User roles allowed to publish this document
	user: #allowedUserRoles

	// Admin roles allowed to publish this document
	admin?: #allowedAdminRoles
}

#allowedUpdaterType: "collaborators" | "ref" | *"author"

#updaterDescriptions: {
	collaborators: """
		Updates are allowed by the original author and from the 'collaborators' metadata field
		of the previous submitted document's version.
		"""

	ref: """
		Updates are allowed by the original author OR from the 'collaborators' metadata field (if defined)
		of the referenced document specified by the 'ref' metadata field.
		"""

	author: """
		Only the original author can update and sign a new version of documents.
		"""
}

#allowedUpdaters: {
	// The type defaults to "author" from #allowedUpdaterType
	type: #allowedUpdaterType

	// The description is looked up from the map using the value of 'type'
	description: #updaterDescriptions[type]
}

#allowedSigners: {
	// Who is allowed to sign a new document
	// TODO: Import roles from a role definition configuration.
	roles: #allowedRoles

	// Who is allowed to sign an update to an existing document.
	update: #allowedUpdaters
}

_allowedSigners: #allowedSigners & {
	roles: #allowedRoles & {
		user: #allowedUserRoles & _ | *[
			_allUserRolesList[0],
		]
	}
}
