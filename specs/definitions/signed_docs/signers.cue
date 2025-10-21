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

// Listed collaborators can post updated versions based on the metadata field as a source of collaborators information
#collaboratorsDef: "collaborators" | "ref" | *"excluded" | "what_the_fuck"

#allowedUpdaters: {
	collaborators:  #collaboratorsDef
	author:         bool | *true  // The original author can post updated versions
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
	update: #allowedUpdaters
}
