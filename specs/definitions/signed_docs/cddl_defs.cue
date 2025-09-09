// Signed Document Definitions
// 
// CDDL Definitions
package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/cddl"
)

#cddlDefinitions: [string]: {
	def: string
	requires: [...#cddlTypesConstraint] | *[]
	description?: string // Description - multiline
	comment?:     string // Single line comments are displayed after a definition. Multiline comments, before.
}

#cddlTypes: [
	for k, _ in cddlDefinitions {k},
]

#cddlTypesConstraint: or(#cddlTypes)

cddlDefinitions: cddl.#cddlDefinitions
cddlDefinitions: cddl.cddlDefinitions
