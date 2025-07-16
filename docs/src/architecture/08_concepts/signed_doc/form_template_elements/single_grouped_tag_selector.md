# Form Template Element - Single Grouped Tag Selector

UI - A selector where a top level group selection, allows a single choice from a list of tags.
Select one option from the dropdown menu.
Only one choice is allowed.

The contents of the `singleGroupedTagSelector` *MUST* have the following format:

```json
"oneOf": [
    {
       "properties": {
         "group": {
          "$ref": "#/definitions/tagGroup",
          "const": "Governance"
         },
         "tag": {
          "$ref": "#/definitions/tagSelection",
          "enum": [
              "Governance",
              "DAO"
          ]
         }
       }
    },
```

## Parents

The Single Grouped Tag Selector form element, can appear as a child of:

['section']

## Definition

{'additionalProperties': False, 'required': ['group', 'tag'], 'type': 'object'}

## Parameters

The Single Grouped Tag Selector form element takes the following parameters:

root={'description': Parameter(property=None, description='The description of the field presented during data entry.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'oneOf_groupedTags': Parameter(property='oneOf', description='A set of tags with a group selector.', required=<OptionalField.excluded: 'excluded'>, type='array', items=Parameter(property=None, description='\tAn array of grouped tag objects, of which one can be selected.\n\tEach object *MUST* have the form:\n\t\n\t```json\n\t"properties": {\n\t\t"group": {\n\t\t\t"$ref": "#/definitions/tagGroup",\n\t\t\t"const": <group name string>\n\t\t},\n\t\t"tag": {\n\t\t\t"$ref": "#/definitions/tagSelection",\n\t\t\t"enum": [\n\t\t\t\t<tag 1 string>,\n\t\t\t\t<tag 2 string>,\n\t\t\t\t...\n\t\t\t]\n\t\t}\n\t}\n\t```', required=<OptionalField.excluded: 'excluded'>, type='object', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'title': Parameter(property=None, description='The label attached to the field.', required=<OptionalField.required: 'yes'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None), 'x-guidance': Parameter(property=None, description='Long form [Markdown][CommonMark] formatted description to give guidance about how the field is to be completed.', required=<OptionalField.optional: 'optional'>, type='string', items=None, choices=None, format=None, content_media_type=None, pattern=None, min_length=None, minimum=None, maximum=None, example=None)}

## Example Usage

This is an Example Form Template showing just the Single Grouped Tag Selector form element, and its parents.

TODO

[CommonMark]: https://spec.commonmark.org/0.31.2/
