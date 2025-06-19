package signed_docs

// Import common definitions JSON schema for use in embedded schemas
_common_defs: {} @embed(file="./payload_schemas/common_definitions.schema.json")

// Category Parameters Document Definition
docs: #DocumentDefinitions & {
	"Category Parameters": {
		metadata: {
			ref:      metadata.ref
			template: metadata.template
		}
		versions: [
			{
				version:  "v1"
				modified: "2024-06-07"
				changes:  "Initial version. Structure aligned with global metadata requirements."
			},
		]
		// Payload is fully templated and not defined here
	}
}
