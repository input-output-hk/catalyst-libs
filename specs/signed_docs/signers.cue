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
	"RootCA",
	"BrandCA",
	"CampaignCA",
	"CategoryCA",
	"RootAdmin",
	"BrandAdmin",
	"CampaignAdmin",
	"CategoryAdmin",
	"Moderator",
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

#allowedUpdaters: {
	collaborators?: bool | *false // Listed collaborators can post updated versions
	author:         bool | *true  // The original author can post updated versions
	any?:           bool | *false // Anyone with the correct role can post updated versions
}

#allowedSigners: {
	// Who is allowed to sign a new document
	// TODO: Import roles from a role definition configuration.
	roles: #allowedRoles

	// Limited to the same signer as the document referenced
	referenced?: bool | *false

	// Who is allowed to sign an update to an existing document.
	update: #allowedUpdaters
}

_allowedSigners: #allowedSigners & {
	roles: #allowedRoles & {
		user: #allowedUserRoles & _ | *[
			_allUserRolesList[0],
		]
	}
	update: #allowedUpdaters
}
