# Form Template Element - Multi Line Text Entry

UI - One or more Lines of text entry.
Line breaks, and special characters are allowed.
Special formatted markup, such as [Markdown][CommonMark] are not allowed.
Enter multiple lines of plain text. You can use line breaks but no special formatting.

## Parents

The Multi Line Text Entry form element, can appear as a child of:

['section']

## Definition

{'contentMediaType': 'text/plain', 'pattern': '^[\\S\\s]*$', 'type': 'string'}

## Parameters

The Multi Line Text Entry form element takes the following parameters:

root={'description': Parameter(property=None, description='The description of the field presented during data entry.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'maxLength': Parameter(property=None, description='Maximum number of characters allowed in the field.', required=<OptionalField.required: 'yes'>, type='integer', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'minLength': Parameter(property=None, description='Minimum number of characters allowed in the field.', required=<OptionalField.optional: 'optional'>, type='integer', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'title': Parameter(property=None, description='The label attached to the field.', required=<OptionalField.required: 'yes'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'x-guidance': Parameter(property=None, description='Long form [Markdown][CommonMark] formatted description to give guidance about how the field is to be completed.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'x-placeholder': Parameter(property=None, description='Placeholder text to display inside the field if it is empty.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None)}

## Example Usage

This is an Example Form Template showing just the Multi Line Text Entry form element, and its parents.

TODO

[CommonMark]: https://spec.commonmark.org/0.31.2/
