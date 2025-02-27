package signed_docs

// Proposal Document Definition

docs: #DocumentDefinitions & {
	"Template": {
		headers:
			"content type": {
				value: [
					"application/schema+json",
					"application/cddl",
				]
			}

		metadata: {
			template: {
				required: "optional"
			}
			template_doc: {}
		}
	}
}
