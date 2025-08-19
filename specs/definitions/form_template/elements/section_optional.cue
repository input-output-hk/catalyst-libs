// Document Segment
//
// Segment -
//   Section -
//     Topic -
package form_template

dictionary: sectionOptional: {
	description: """
		## Functional Behavior

		Sections have no functional behavior beyond providing 
		structure to the underlying data collected by the form.

		The ONLY Element that can appear in the root of a Form is a section or optionalSection.

		An optionalSection may be set to `null` which means everything contained by the section is not set.
		This has two primary use cases:

		1. Completely optional sections the user may or may not choose to complete.
		2. Sections which may not be present in a draft of a document, but must be present in a 
		final form of the document.

		As JsonSchema allows `null` or `object` to be valid for optionalSchema, checking if its
		valid to be `null` in a finalized document must be done as an extra step, 
		after json schema validation.

		## Visual Representation

		Sections represent logical breaks in the form structure.
		Optional Sections allow for the entire section to be toggled off (incomplete).

		A Optional Section may have whatever visual representation that is required.
		Nominally however, a Optional Section that is in the root of the document
		is known as a ***Optional Document Segment**.
		Whereas a section that is embedded within another section is a 
		**Optional Document Section** or **Optional Sub-Section**.

		There is no limit to how many levels sub-sections are nested,
		however the application is not required to show them any differently
		from one another.

		The visual display of sections has no impact on how it is represented
		in the data captured.

		For Optional Sections, the UI should allow the partial capture and local storage of data
		within the section, but NOT submit it if it would be invalid according to the document template schema.
		The UI may also employ a specific toggle which determines if an Optional Section is included or not.

		Before submitting any document as Final, that contains Optional Sections, the UI must validate that
		those sections have been properly completed if they are not allowed to be `null` in a finalized document.
		"""
	parent: ["{}", "section", "sectionOptional"]
	definition: type: ["object", "null"]

	parameters: {
		title: {
			description: "The title of the section."
			required:    "yes"
		}
		description: description: "The displayable description attached to the section.  Markdown formatted contents."
		properties: description:  "The sub fields of the section when it is not `null`."
		required: {
			description: "Which fields MUST appear in the segment."
			required:    "optional"
		}
		"x-flatten": example: false
		"x-icon": example:    "bookmark"
		"x-order": {}
		"x-final-optional": example: true
	}
}
