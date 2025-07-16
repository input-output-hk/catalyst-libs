# Form Template Element - Multi Line Text Entry [Markdown][CommonMark]

UI - Multiline text entry with [Markdown][CommonMark] content.
Use [Markdown][CommonMark] formatting for rich text.
[Markdown][CommonMark] formatting is as defined by [CommonMark].

The following [Markdown][CommonMark] Extensions are also supported:

* None

## Parents

The Multi Line Text Entry [Markdown][CommonMark] form element, can appear as a child of:

['section']

## Definition

{'contentMediaType': 'text/markdown', 'pattern': '^[\\S\\s]*$', 'type': 'string'}

## Parameters

The Multi Line Text Entry [Markdown][CommonMark] form element takes the following parameters:

root={'description': Parameter(property=None, description='The description of the field presented during data entry.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'maxLength': Parameter(property=None, description='Maximum number of characters allowed in the field.', required=<OptionalField.required: 'yes'>, type='integer', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'minLength': Parameter(property=None, description='Minimum number of characters allowed in the field.', required=<OptionalField.optional: 'optional'>, type='integer', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'title': Parameter(property=None, description='The label attached to the field.', required=<OptionalField.required: 'yes'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'x-guidance': Parameter(property=None, description='Long form [Markdown][CommonMark] formatted description to give guidance about how the field is to be completed.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'x-placeholder': Parameter(property=None, description='Placeholder text to display inside the field if it is empty.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None)}

## Example Usage

This is an Example Form Template showing just the Multi Line Text Entry [Markdown][CommonMark] form element, and its parents.

TODO

[CommonMark]: https://spec.commonmark.org/0.31.2/
