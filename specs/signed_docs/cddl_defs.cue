// Signed Document Definitions
// 
// CDDL Definitions
package signed_docs

// List of cddl definitions, cddl_type_name: cddl_definition
#cddlDefinitions: {
	[string]: {
		def: string
		requires: [...#cddlTypesConstraint] | *[]
	}
}

cddlDefinitions: #cddlDefinitions & {
	"uuid_v7": {
		def: "6.37(bytes .size 16)"
	}
	"uuid_v4": {
		def: "6.37(bytes .size 16)"
	}
	"document_type": {
		def: "[ 1* uuid_v4 ]"
		requires: ["uuid_v4"]
	}
	"blake2b_256": {
		def: "bytes .size 32"
	}
	"document_id": {
		def: "uuid_v7"
		requires: ["uuid_v7"]
	}
	"document_ver": {
		def: "uuid_v7"
		requires: ["uuid_v7"]
	}
	"cid": {
		def: "text"
	}
	"generic_future_hash": {
		def: "[uint, text / bytes]"
	}
	"document_hash": {
		def: "{ \"cid\" => cid }"
		requires: ["cid"]
	}
	"document_ref": {
		def: "[ 1* [ document_id, document_ver, document_hash ] ]"
		requires: ["document_id", "document_ver", "document_hash"]
	}
	"json_pointer": {
		def: "text"
	}
	"section_ref": {
		def: "json_pointer"
		requires: ["json_pointer"]
	}
	"catalyst_id": {
		def: "text"
	}
	"collaborators": {
		def: "[ * catalyst_id ]"
		requires: ["catalyst_id"]
	}
	"revocations": {
		def: "[ * document_ver ] / true "
		requires: ["document_ver"]
	}
}

#cddlTypes: [
	for k, _ in cddlDefinitions {k},
]

#cddlTypesConstraint: or(#cddlTypes)
