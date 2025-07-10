// Document Segment
//
// Segment -
//   Section -
//     Topic -
package form_template

dictionary: section: {
	description: """
		UI - Logical Document Sub-Section Break.
		Subsections are logical breaks in the form structure.
		The ONLY Element that can appear in the root of a Form is a `section`.
		`section` can be nested arbitrarily deep.
		The form presentation can decide how deep, and what formatting it used
		for each section at each level, however, even if the form is flattened and the section not
		displayed, then data must still follow the section nesting.
		"""
	parent: ["{}", "section"]
	definition: type: "object"
	parameters: {
		title: {
			description: "The title of the section."
			required:    "yes"
		}
		description: description: "The displayable description attached to the section.  Markdown formatted contents."
		properties: description:  "The sub fields of the section."
		required: {
			description: "Which fields MUST appear in the segment."
			required:    "optional"
		}
		"x-flatten": {}
		"x-icon": {}
		"x-order": {}
	}
}
