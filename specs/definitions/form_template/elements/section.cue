// Document Segment
//
// Segment -
//   Section -
//     Topic -
package form_template

dictionary: section: {
	description: """
		## Functional Behavior

		Sections have no functional behavior beyond providing 
		structure to the underlying data collected by the form.

		The ONLY Element that can appear in the root of a Form is a section.

		## Visual Representation

		Sections represent logical breaks in the form structure.

		A Section may have whatever visual representation that is required.
		Nominally however, a section that is in the root of the document
		is known as a ***Document Segment**.
		Whereas a section that is embedded within another section is a 
		**Document Section** or **Sub-Section**.

		There is no limit to how many levels sub-sections are nested,
		however the application is not required to show them any differently
		from one another.

		The visual display of sections has no impact on how it is represented
		in the data captured.
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
		"x-flatten": example: false
		"x-icon": example:    "bookmark"
		"x-order": {}
	}
}
