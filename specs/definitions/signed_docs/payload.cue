package signed_docs

import (
	Eg "github.com/input-output-hk/catalyst-libs/specs/generic:examples"
	"github.com/input-output-hk/catalyst-libs/specs/cddl"
	"github.com/input-output-hk/catalyst-libs/specs/regex"
)

// Payload definition
#payload: {
	// Description of the payload
	description: string

	// Is the Payload allowed to be nil?
	// This DOES NOT preclude there being a payload also defined.
	// For example when `revocations` is `true` then the payload may be `nil`.
	nil: true | *false
}


// Payload definition
#payload_json: {
	// Extends #payload
	#payload
	// Optional fixed schema for the payload.
	// A URI or inline JSON Schema that the payload must validate against.
	// Can't work out a way to validated json schema constraint here,
	// but is validated by the documentation generator.
	schema?: _ | =~ regex.def.httpsUrl.pattern
	// Examples of the schema.
	examples?: Eg.#list
}

// Payload definition for cbor payloads
#payload_cbor: { 
	// Extends #payload
	#payload
	// CBOR payloads must have a CDDL Schema defined.
	schema?: cddl.#cddlTypesConstraint
	
	// Examples of the schema.
	examples?: Eg.#list
}
