# Form Template Specification

Form Templates are the basis by which most documents get and validate their payloads.

Form Templates are generic, and not customized per document type.
All Specific Document Form Templates have the same capabilities.

## [JSON Schema][JSON Schema-2020-12]

Form Templates are based on [JSON Schema version 2020-12][JSON Schema-2020-12].
However, [JSON Schema][JSON Schema-2020-12] is only a validation specification and does not
include any details about how such a form should be displayed for data entry.

Form Templates, take standard [JSON Schema][JSON Schema-2020-12] and extend it in a backwards compatible way
so that the Form Template can be interpreted with hints to aid in rendering a form
for data capture.

### Basic Structure

The Base Form Template has the following format.

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "Form Template Base Schema"

    * [form_template.schema.json](schema/form_template.schema.json)

    ``` json
    {{ include_file('./schema/form_template.schema.json', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

#### `"$defs"`

In [JSON Schema][JSON Schema-2020-12] the `$defs` defines reusable definitions.
We extend the definition of `$defs`.
Every possible Form Element is defined in `$defs`.
A fields *MUST* use a pre-defined Form Element from the `$defs` and
it is impermissible for a Form Element to not use a reference to its
definition, nor for a definition to be created which does not follow
this specification.

In any single Form Template, the `$defs` section of the Schema *MAY* only
include definitions of Form Elements used by the template itself.

The definitions within the Base Form Template above serve as the dictionary
of all available defined Form Elements.

#### `"properties"`

The `"properties"` section of the form template contains all of the fields
to be captured by the form, and each field is defined by its element type.

Form Elements *MAY* and typically do have Parameters, the possible parameters
per Form Element are documented with each Form Elements individual documentation.
Parameters are typically standard [JSON Schema][JSON Schema-2020-12] properties on a type.
As not all parameters required are defined by [JSON Schema][JSON Schema-2020-12], extended parameters may exist that
are prefixed by `x-`.

Extended Parameters are intended to provide more information related to each Form element,
to assist with presentation or using any particular Field.
They are not currently associated with validation.

#### `"$schema"`

This defines that the template is a standard [JSON Schema version 2020-12][JSON Schema-2020-12] document.
Any document derived from the template is only valid if it validates
according to [JSON Schema][JSON Schema-2020-12] validation rules against the template.

#### `"title"`

This is a name for the template.
It is not used by consumers of the template, but could be used by
template builders to help identify the template.

#### `"description"`

This is a long multi line description about the template itself.
It is not used by consumers of the template, but could be used by
template builders to help identify and define the purpose of the template.

There is expected to be a number of templates used in the system, and both
`"title"` and `"description"` help template builders organize templates.

#### `"maintainers"`

Is a list of the entities who have made or updated the template.
It is optional, but if it is included, it *MUST* be an array of objects.
Each object must have two fields:

* `"name"` : The name of the individual maintainer or group responsible for the template.
* `"url"` : A link where more information about the maintainer or group can be found.
    If there is no link, the field is still present, but an empty string (`""`).

#### `"type": "object"`

The Template *ONLY* supports a [json][RFC8259] type of Object at the root of the template.
It is not permissible to create templates which are bare strings, or numbers, etc.

#### `"additionalProperties": false`

All Templates must be exhaustively defined.
It is not permissible to add fields to a document derived from a Form Template,
that are not present in the Form Template itself.

`"additionalProperties": false`

## Example Form Template

This is an Example Form Template which has a property shown for every
possible Form Element.

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "Form Template Example Schema"

    * [form_template_example.schema.json](schema/form_template_example.schema.json)

    ``` json
    {{ include_file('./schema/form_template_example.schema.json', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

## Dictionary of all defined Form Elements

| Name | Element |
| --- | --- |
| Drop Down Single Select | `[dropDownSingleSelect](./form_template_elements/drop_down_single_select.md)` |
| Multi Line Text Entry | `[multiLineTextEntry](./form_template_elements/multi_line_text_entry.md)` |
| Multi Line Text Entry List [Markdown][CommonMark] | `[multiLineTextEntryListMarkdown](./form_template_elements/multi_line_text_entry_list_markdown.md)` |
| Multi Line Text Entry [Markdown][CommonMark] | `[multiLineTextEntryMarkdown](./form_template_elements/multi_line_text_entry_markdown.md)` |
| Multi Select | `[multiSelect](./form_template_elements/multi_select.md)` |
| Radio Button Select | `[radioButtonSelect](./form_template_elements/radio_button_select.md)` |
| Section | `[section](./form_template_elements/section.md)` |
| Single Grouped Tag Selector | `[singleGroupedTagSelector](./form_template_elements/single_grouped_tag_selector.md)` |
| Single Line Https Url Entry | `[singleLineHttpsUrlEntry](./form_template_elements/single_line_https_url_entry.md)` |
| Single Line Https Url Entry List | `[singleLineHttpsUrlEntryList](./form_template_elements/single_line_https_url_entry_list.md)` |
| Single Line Text Entry | `[singleLineTextEntry](./form_template_elements/single_line_text_entry.md)` |
| Single Line Text Entry List | `[singleLineTextEntryList](./form_template_elements/single_line_text_entry_list.md)` |

### TODO

[JSON Schema-2020-12]: https://json-schema.org/draft/2020-12
[CommonMark]: https://spec.commonmark.org/0.31.2/
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
