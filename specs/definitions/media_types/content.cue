// Content and Encoding Types
package media_types

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/regex"
)

// Content Type name : Description
#contentTypes: {
	[string]: {
		description: string // description of the content type
		coap_type?:  int
	}
}

_plaintext_description: """
	Plain Text with no markup or special formatting.<br>
	Note:
	* Multiline Plain Text *MUST* always interpret `\\n` 
	as a hard line break.
	"""

_markdown_description: """
	Formatted text using Markdown for rich text.<br>
	Note:
	* Markdown formatting is as defined by CommonMark.
	* IF the document includes HTML, then HTML5 syntax only is supported.
	* The following Markdown Extensions are also supported:
		* None
	"""

_html_description: """
	Formatted text using HTML5 markup for rich text.<br>
	Note:
	* Only HTML5 syntax is supported.
	"""

_css_description: """
	CSS Content used for styling HTML.<br>
	Note:
	* CSS should use the least set of features possible to achieve
	  the desired presentation to ensure the broadest compatibility.
	"""

_handlebars_template_description: """
	* The text includes Handlebars type template fields that need
	processing and replacement prior to display.
	"""

contentTypes: #contentTypes & {
	"application/json": {
		description: "JSON Document"
		coap_type:   50
	}
	"application/schema+json": description: """
		A JSON Schema Draft 2020-12 Document.<br>
		Note:
		* This is a draft/unofficial media type.
		"""
	"application/cbor": {
		description: "An RFC8949 Binary CBOR Encoded Document."
		coap_type:   60
	}
	"application/cddl": description: """
		A CDDL Document.<br>
		Note:
		* This is an unofficial media type
		* RFC9165 Additional Control Operators for CDDL are supported.
		* Must not have Modules, schema must be self-contained.
		"""
	"text/plain; charset=utf-8": {
		description: """
            \(_plaintext_description)
            """
		coap_type:   0
	}
	"text/plain; charset=utf-8; template=handlebars": description:    """
		\(_plaintext_description)
		\(_handlebars_template_description)
		"""
	"text/markdown; charset=utf-8": description:                      """
		\(_markdown_description)
		"""
	"text/markdown; charset=utf-8; template=handlebars": description: """
		\(_markdown_description)
		\(_handlebars_template_description)
		"""
	"text/html; charset=utf-8": description:                          """
		\(_html_description)
		"""
	"text/html; charset=utf-8; template=handlebars": description:     """
		\(_html_description)
		\(_handlebars_template_description)
		"""
	"text/css; charset=utf-8": {
		description: """
            \(_css_description)
            """
		coap_type:   20000
	}
	"text/css; charset=utf-8; template=handlebars": description: """
		\(_css_description)
		\(_handlebars_template_description)
		"""
}

allContentTypes: list.Sort([
	for k, _ in contentTypes {k},
], list.Ascending)

#contentType: or(allContentTypes)

coapTypes: {
	for k, v in contentTypes
	if v.coap_type != _|_ {
		"\(v.coap_type)": k
	}
}

allCoapTypes: list.UniqueItems
allCoapTypes: list.Sort([
	for _, v in contentTypes if v.coap_type != _|_ {v.coap_type},
], list.Ascending)

#allCoapTypes: or(allCoapTypes)

allCoapTypesStr: [...string]
allCoapTypesStr: [for v in allCoapTypes {"\(v)"}]

jsonContentTypes: list.UniqueItems
jsonContentTypes: list.Sort([
	for k, _ in contentTypes if k =~ regex.def.jsonContentType.pattern {k},
], list.Ascending)

cborContentTypes: list.UniqueItems
cborContentTypes: list.Sort([
	for k, _ in contentTypes if k =~ regex.def.cborContentType.pattern {k},
], list.Ascending)

cddlContentTypes: list.UniqueItems
cddlContentTypes: list.Sort([
	for k, _ in contentTypes if k =~ regex.def.cddlContentType.pattern {k},
], list.Ascending)
