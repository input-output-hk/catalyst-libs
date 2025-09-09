// Presentation Template Json Schema Definitions Dictionary
// 
// Structure and Defined Cards for Presentation Templates.
package presentation_template

// Generic Presentation Template
presentationTemplate: {
	$schema: "https://json-schema.org/draft/2020-12/schema"
	title:   "Presentation Template"
	description: """
		Presentation Templates define how data extracted from Form Data is to be presented.
		They provide a way to parameterize the contents of a UI in response to the changing
		needs of the Forms themselves.
		"""
	maintainers: [{
		name: "Catalyst Team"
		url:  "https://projectcatalyst.io/"
	}]
	type: "object"
	properties: {
		name: $ref:         "#/$defs/cardName"
		title: $ref:        "#/$defs/cardTitle"
		description: $ref:  "#/$defs/cardDescription"
		requiredDocs: $ref: "#/$defs/requiredDocumentTypes"
		layout: $ref:       "#/$defs/layoutParameters"
		template: $ref:     "#/$defs/cardTemplate"
		css: $ref:          "#/$defs/cardTemplateCss"
	}
	required: [
		"cardName",
		"requiredDocuments",
		"layoutParameters",
		"template",
	]
	additionalProperties: false

	$defs: {
		cardName: {
			type: "string"
			enum: _allCardNames
			description: """
				A Card has to have one of the well known defined names.
				These are the primary identifier which is used by the UI to determine
				where the UI will place the card.
				"""
		}
		cardTitle: {
			type:        "string"
			description: "A title shown to the editor of the card.  Not used by the UI."
		}
		cardDescription: {
			type:        "string"
			description: "A long form description of the purpose of the card. Not used by the UI."
		}
		requiredDocumentTypes: {
			type:        "array"
			uniqueItems: true
			items: {
				type:   "string"
				format: "uuid"
			}
			description: "A list of the document types (UUIDs) the presentation template needs."
		}
		layoutParameters: {
			type:        "object"
			description: "Parameters which help the front end layout the provided template. To be defined."
		}
		cardTemplate: {
			type:        "string"
			contentType: "text/html; charset=utf-8; template=handlebars"
			description: """
				HTML5 defined presentation layout for the card.
				The data is templated with handlebars, and the data that can be inserted is
				derived from the `requiredDocumentTypes` and available system wide dynamic data.
				"""
		}
		cardTemplateCss: {
			type:        "string"
			contentType: "text/css; charset=utf-8; template=handlebars"
			description: """
				Optional styling that can be used by the HTML generated from the template for presentation.
				"""
		}
	}
}
