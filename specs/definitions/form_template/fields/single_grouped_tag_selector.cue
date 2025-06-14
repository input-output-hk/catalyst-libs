// Tag Input
package form_template

dictionary: #formTemplate & {

	singleGroupedTagSelector: {
		description: """
			UI - A selector where a top level group selection, allows a single choice from a list of tags.
			Select one option from the dropdown menu. 
			Only one choice is allowed.

			The contents of the `singleGroupedTagSelector` *MUST* have the following format:

			```json
			"oneOf": [
				{
					"properties": {
						"group": {
							"$ref": "#/definitions/tagGroup",
							"const": "Governance"
						},
						"tag": {
							"$ref": "#/definitions/tagSelection",
							"enum": [
								"Governance",
								"DAO"
							]
						}
					}
				},
			```
			"""
		parent: "section"

		definition: {
			type: "object"
			required: [
				"group",
				"tag",
			]
		}
		parameters: {
			title: {}
			description: {}
			"x-guidance": {}
			oneOf_groupedTags: {}
		}
	}
}
