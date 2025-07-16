# Form Template Element - Radio Button Select

UI - Radio Button Selection.

Select one option from a list of text options.
Selector is styled as a set of Radio Buttons.

## Parents

The Radio Button Select form element, can appear as a child of:

['section']

## Definition

{'contentMediaType': 'text/plain', 'pattern': '^[^\\n]*$', 'type': 'string'}

## Parameters

The Radio Button Select form element takes the following parameters:

root={'description': Parameter(property=None, description='The description of the field presented during data entry.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'enum': Parameter(property=None, description='An array of string to select from.', required=<OptionalField.required: 'yes'>, type='array', items=Parameter(property=None, description='An element of the Enum.', required=<OptionalField.excluded: 'excluded'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'title': Parameter(property=None, description='The label attached to the field.', required=<OptionalField.required: 'yes'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'x-guidance': Parameter(property=None, description='Long form [Markdown][CommonMark] formatted description to give guidance about how the field is to be completed.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None)}

## Example Usage

This is an Example Form Template showing just the Radio Button Select form element, and its parents.

TODO

[CommonMark]: https://spec.commonmark.org/0.31.2/
