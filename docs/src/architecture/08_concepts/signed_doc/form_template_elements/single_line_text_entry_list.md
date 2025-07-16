# Form Template Element - Single Line Text Entry List

UI - A Growable List of single line text
A single line of text.
No formatting, markup, line breaks, or special characters are allowed.
Add multiple single-line text entries.
Each entry should be unique.

## Parents

The Single Line Text Entry List form element, can appear as a child of:

['section']

## Definition

{'items': {'minLength': 1, 'pattern': '^[^\\n]*$', 'type': 'string'}, 'type': 'array', 'uniqueItems': True}

## Parameters

The Single Line Text Entry List form element takes the following parameters:

root={'contains': Parameter(property=None, description='The choices the multi select can contain.', required=<OptionalField.required: 'yes'>, type='array', items=Parameter(property=None, description='An individual Choice.', required=<OptionalField.excluded: 'excluded'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'default': Parameter(property=None, description='Default Array of text can be supplied.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'description': Parameter(property=None, description='The description of the field presented during data entry.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'maxItems': Parameter(property=None, description='An array instance is valid against "maxItems" if its size is less than, or equal to, the value of this keyword.', required=<OptionalField.required: 'yes'>, type='integer', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'minItems': Parameter(property=None, description='An array instance is valid against "minItems" if its size is greater than, or equal to, the value of this keyword.', required=<OptionalField.optional: 'optional'>, type='integer', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'title': Parameter(property=None, description='The label attached to the field.', required=<OptionalField.required: 'yes'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'x-guidance': Parameter(property=None, description='Long form [Markdown][CommonMark] formatted description to give guidance about how the field is to be completed.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'x-placeholder': Parameter(property=None, description='Placeholder text to display inside the field if it is empty.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None)}

## Example Usage

This is an Example Form Template showing just the Single Line Text Entry List form element, and its parents.

TODO

[CommonMark]: https://spec.commonmark.org/0.31.2/
