// Single Line Text Entry
package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: #formTemplate & {
	multiLineTextEntryMarkdown: {
		description: """
			UI - Multiline text entry with Markdown content.
			Use Markdown formatting for rich text. 
			Markdown formatting is as defined by CommonMark.

			The following Markdown Extensions are also supported:

			* None
			"""
		definition: {
			type:             "string"
			contentMediaType: "text/markdown"
			pattern:          regex.def.multiLine.pattern
		}
		parent: "section"
		parameters: {
			title: {}
			description: {}
			minLength: {}
			maxLength: {}
			"x-guidance": {}
			"x-placeholder": {}
		}
	}
}
