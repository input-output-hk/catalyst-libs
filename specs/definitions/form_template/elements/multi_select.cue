// Radio Button selection from a list of text options.

package form_template

dictionary: multiSelect: {
	description: """
		## Functional Behavior

		A List of items, where multiple values can be selected.
		
		Each entry in the list is a `\(definition._itemElement)` type Form element.
		However, the contents are not entered free-form, they are chosen from the 
		provided options.

		## Visual Representation

		A List of items to be selected.
		The selector is not valid if less than `minItems` are selected.
		And no more than `maxItems` can be selected.

		The value encoded in the document is the text value for each item
		selected, in the order presented.

		The Items should appear and be parameterizable in the same way the
		base `\(definition._itemElement)` type Form element can be.

		It should appear similar to `\(definition._itemElement)` except that
		multiple items can be selected, instead of just one.
		"""
	parent: ["section"]

	definition: {
		type:         "array"
		_itemElement: "singleSelect"
		items: $ref: "#/$defs/\(_itemElement)"
		uniqueItems: true
	}
	parameters: {
		title: example:       "Select Items"
		description: example: "Select a few items you prefer."
		default: {
			description: "Default selections can be supplied."
			required:    "optional"
			example: [
				"option1",
				"option3",
			]
		}
		items: dictionary.singleSelect.parameters
		minItems: {
			description: """
				The minimum number of items that may be selected.
				Default to 0 if not specified.
				"""
			example: 2
		}
		maxItems: {
			description: """
				The maximum number of items that may be selected.
				Default to the size of the selectable items if not specified.
				If it is less than `minItems` it is taken to == `minItems`.
				"""
			example: 2
		}
		"x-guidance": example: "Select two items only, no more, no less."
		"x-icon": example:     "cursor-click"
	}
}
