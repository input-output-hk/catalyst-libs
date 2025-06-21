package signed_docs

_common_defs: {} @embed(file="./payload_schemas/common_definitions.schema.json")

// Brand Parameters Document Definition

docs: #DocumentDefinitions & {
	"Brand Parameters": {
		metadata: {
			ref:      metadata.ref      // Reference to the next highest document
			template: metadata.template // Reference to a brand/campaign/etc template document
		}
		versions: [
			{
				version:  "v1"
				modified: "2024-06-07"
				changes:  "Initial version. Structure aligned with global metadata requirements."
			},
			{
				version:  "v2"
				modified: "2025-04-04"
				changes:  "Updated to include common definitions and examples."
			},
		]
		// Payload is fully templated and not defined here
	}

}
