// Document Segment
//
// Segment -
//   Section -
//     Topic -
package form_template

dictionary: #formTemplate & {
	section: {
		description: """
			UI - Logical Document Sub-Section Break.
			Subsections containing specific details about the proposal.
			"""
		parent: "segment"
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
			"x-icon": {}
			"x-order": {}
		}
	}
}
