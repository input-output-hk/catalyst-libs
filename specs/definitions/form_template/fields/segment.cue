// Document Segment
//
// Segment -
//   Section -
//     Topic -
package form_template

dictionary: segment: {
	description: """
		Logical Document Section - Grouping Only - Highest Level Group
		"x-note": "
			Major sections of the proposal. Each segment contains sections of information grouped together.
			"
		"""
	definition: type: "object"
	parameters: {
		title: description:       "The title of the segment."
		description: description: "The displayable description attached to the segment.  Markdown formatted contents."
		properties: description:  "The sub fields of the segment."
		required: {
			description: "Which fields MUST appear in the segment."
			required:    "optional"
		}
		"x-icon": {}
		"x-order": {}
	}
}
