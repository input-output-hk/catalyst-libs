// Choices that can be made for a string field.
package form_template

import (
	"list"
)

// Valid values for `format`
_allFormats: list.UniqueItems
_allFormats: list.Sort([
	"uri",
	"path",
], list.Ascending)

#formatChoices: or(_allFormats)

// Valid values for `contentMediaType`
_allContentMediaTypes: list.UniqueItems
_allContentMediaTypes: list.Sort([
	"text/plain",
	"text/plain; template=handlebars",
	"text/markdown",
	"text/markdown; template=handlebars",
	"text/html",
	"text/html; template=handlebars",
], list.Ascending)

#contentMediaTypeChoices: or(_allContentMediaTypes)
