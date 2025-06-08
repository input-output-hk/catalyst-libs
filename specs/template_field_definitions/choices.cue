// Choices that can be made for a string field.
package template_field_definitions

import (
	"list"
)

// Valid values for `format`
_allFormats: list.UniqueItems
_allFormats: list.Sort([
	"uri",
	"path",
	"radioButtonSelect",
	"dropDownSingleSelect",
	"multiSelect",
], list.Ascending)

#formatChoices: or(_allFormats)

// Valid values for `contentMediaType`
_allContentMediaTypes: list.UniqueItems
_allContentMediaTypes: list.Sort([
	"text/plain",
	"text/markdown",
], list.Ascending)

#contentMediaTypeChoices: or(_allContentMediaTypes)
