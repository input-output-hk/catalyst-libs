// Signed Document Definitions
// 
// CDDL Definitions
package cddl

import (
	Eg "github.com/input-output-hk/catalyst-libs/specs/generic:examples"
)

// List of cddl definitions, cddl_type_name: cddl_definition
#cddlDefinitions: {
	[string]: {
		def: string
		requires: [...#cddlTypesConstraint] | *[]
		description?: string // Description - multiline
		comment?:     string // Single line comments are displayed after a definition. Multiline comments, before.
		examples?:    Eg.#list
	}
}

#cddlTypes: [
	for k, _ in cddlDefinitions {k},
]

#cddlTypesConstraint: or(#cddlTypes)
