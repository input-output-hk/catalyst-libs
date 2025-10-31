# Form Template Element - Section

## Functional Behavior

Sections have no functional behavior beyond providing
structure to the underlying data collected by the form.

The ONLY Element that can appear in the root of a Form is a section.

## Visual Representation

Sections represent logical breaks in the form structure.

A Section may have whatever visual representation that is required.
Nominally however, a section that is in the root of the document
is known as a ***Document Segment**.
Whereas a section that is embedded within another section is a
**Document Section** or **Sub-Section**.

There is no limit to how many levels sub-sections are nested,
however the application is not required to show them any differently
from one another.

The visual display of sections has no impact on how it is represented
in the data captured.

## Parent Elements

The Section form element, can appear as a child of:

* The Root Object of the [JSON Schema][JSON Schema-2020-12]
* [Section](section.md)
* [Section Optional](section_optional.md)

## Definition

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Definition: Section"

    ```json
    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$defs": {
        "section": {
          "additionalProperties": false,
          "type": "object"
        }
      }
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

## Parameters

The Section form element takes the following parameters:

<!---HTML START-->
<!-- markdownlint-disable -->
<div id="element_Section_parameters" style="padding-left:0px;padding-right:0px;padding-top:10px;padding-bottom:10px;overflow-x:auto;overflow-y:auto;width:100%;height:auto;">
<style>
#element_Section_parameters table {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', 'Fira Sans', 'Droid Sans', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }

#element_Section_parameters thead, tbody, tfoot, tr, td, th { border-style: none; }
 tr { background-color: transparent; }
#element_Section_parameters p { margin: 0; padding: 0; }
 #element_Section_parameters .gt_table { display: table; border-collapse: collapse; line-height: normal; margin-left: auto; margin-right: auto; color: #333333; font-size: 16px; font-weight: normal; font-style: normal; background-color: #FFFFFF; width: 100%; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; }
 #element_Section_parameters .gt_caption { padding-top: 4px; padding-bottom: 4px; }
 #element_Section_parameters .gt_title { color: #333333; font-size: 125%; font-weight: initial; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; border-bottom-color: #FFFFFF; border-bottom-width: 0; }
 #element_Section_parameters .gt_subtitle { color: #333333; font-size: 85%; font-weight: initial; padding-top: 3px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; border-top-color: #FFFFFF; border-top-width: 0; }
 #element_Section_parameters .gt_heading { background-color: #FFFFFF; text-align: center; border-bottom-color: #FFFFFF; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #element_Section_parameters .gt_bottom_border { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #element_Section_parameters .gt_col_headings { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #element_Section_parameters .gt_col_heading { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; overflow-x: hidden; }
 #element_Section_parameters .gt_column_spanner_outer { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; padding-top: 0; padding-bottom: 0; padding-left: 4px; padding-right: 4px; }
 #element_Section_parameters .gt_column_spanner_outer:first-child { padding-left: 0; }
 #element_Section_parameters .gt_column_spanner_outer:last-child { padding-right: 0; }
 #element_Section_parameters .gt_column_spanner { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; overflow-x: hidden; display: inline-block; width: 100%; }
 #element_Section_parameters .gt_spanner_row { border-bottom-style: hidden; }
 #element_Section_parameters .gt_group_heading { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: middle; text-align: left; }
 #element_Section_parameters .gt_empty_group_heading { padding: 0.5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: middle; }
 #element_Section_parameters .gt_from_md> :first-child { margin-top: 0; }
 #element_Section_parameters .gt_from_md> :last-child { margin-bottom: 0; }
 #element_Section_parameters .gt_row { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; margin: 10px; border-top-style: none; border-top-width: 1px; border-top-color: #D5D5D5; border-left-style: none; border-left-width: 1px; border-left-color: #D5D5D5; border-right-style: none; border-right-width: 1px; border-right-color: #D5D5D5; vertical-align: middle; overflow-x: hidden; }
 #element_Section_parameters .gt_stub { color: #333333; background-color: #89D3FE; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D5D5D5; padding-left: 5px; padding-right: 5px; }
 #element_Section_parameters .gt_stub_row_group { color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D3D3D3; padding-left: 5px; padding-right: 5px; vertical-align: top; }
 #element_Section_parameters .gt_row_group_first td { border-top-width: 2px; }
 #element_Section_parameters .gt_row_group_first th { border-top-width: 2px; }
 #element_Section_parameters .gt_striped { color: #333333; background-color: #EDF7FC; }
 #element_Section_parameters .gt_table_body { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #element_Section_parameters .gt_sourcenotes { color: #333333; background-color: #FFFFFF; border-bottom-style: none; border-bottom-width: 2px; border-bottom-color: #D3D3D3; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; }
 #element_Section_parameters .gt_sourcenote { font-size: 90%; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; text-align: left; }
 #element_Section_parameters .gt_left { text-align: left; }
 #element_Section_parameters .gt_center { text-align: center; }
 #element_Section_parameters .gt_right { text-align: right; font-variant-numeric: tabular-nums; }
 #element_Section_parameters .gt_font_normal { font-weight: normal; }
 #element_Section_parameters .gt_font_bold { font-weight: bold; }
 #element_Section_parameters .gt_font_italic { font-style: italic; }
 #element_Section_parameters .gt_super { font-size: 65%; }
 #element_Section_parameters .gt_footnote_marks { font-size: 75%; vertical-align: 0.4em; position: initial; }
 #element_Section_parameters .gt_asterisk { font-size: 100%; vertical-align: 0; }

</style>
<table style="table-layout: fixed;; width: 100%" class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<colgroup>
  <col style="width:10%;"/>
  <col style="width:50%;"/>
</colgroup>

<thead>

  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_title gt_font_normal">Section</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

Parameters

</td>
  </tr>

</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>description</code></strong><br>The displayable description attached to the section.  <a href="https://spec.commonmark.org/0.31.2/">Markdown</a> formatted contents.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">optional</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped"><code>string</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Content Media Type</th>
    <td class="gt_row gt_left"><a href="https://spec.commonmark.org/0.31.2/">text/markdown;</a> <a href="https://handlebarsjs.com/">template=handlebars</a></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>properties</code></strong><br>The sub fields of the section.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left gt_striped">yes</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left"><code>string</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>required</code></strong><br>Which fields MUST appear in the segment.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left gt_striped">optional</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left"><code>string</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>title</code></strong><br>The title of the section.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left gt_striped">yes</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left"><code>string</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Content Media Type</th>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc2046.html">text/plain</a></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>x-flatten</code></strong><br>If present, and true, then form element is to be flattened into its parent.
Typically this parameter is only present in sections.
The UI is free to decide how it presents flattened sections.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">optional</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped"><code>boolean</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left"><code>x-flatten: false</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>x-icon</code></strong><br>The name of the Icon to display with the field.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left gt_striped">optional</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left"><code>string</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Choices</th>
    <td class="gt_row gt_left gt_striped"><a href="../../form_templates/#icons">Icons</a></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left"><code>x-icon: &quot;bookmark&quot;</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>x-order</code></strong><br>The ordering of the properties to be enforced when displayed.
Any field not listed here will get displayed in an alphabetical order following the listed fields.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left gt_striped">yes</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left"><code>string</code></td>
  </tr>
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->

## Example Usage

This is an Example Form Template showing just the Section form element, and its parents.

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: "

    ```json
    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "title": "section Example",
      "description": "An example of the section Element, and it's parents.",
      "$defs": {
        "section": {
          "additionalProperties": false,
          "type": "object"
        },
        "sectionOptional": {
          "additionalProperties": false,
          "type": [
            "object",
            "null"
          ]
        }
      },
      "type": "object",
      "properties": {
        "exampleSection": {
          "$ref": "#/$defs/section",
          "properties": {
            "exampleSection": {
              "$ref": "#/$defs/section",
              "properties": {
                "exampleSectionOptional": {
                  "$ref": "#/$defs/sectionOptional",
                  "x-final-optional": true,
                  "x-flatten": false,
                  "x-icon": "bookmark"
                }
              },
              "x-flatten": false,
              "x-icon": "bookmark"
            },
            "exampleSectionOptional": {
              "$ref": "#/$defs/sectionOptional",
              "x-final-optional": true,
              "x-flatten": false,
              "x-icon": "bookmark"
            }
          },
          "x-flatten": false,
          "x-icon": "bookmark"
        },
        "exampleSectionOptional": {
          "$ref": "#/$defs/sectionOptional",
          "x-final-optional": true,
          "x-flatten": false,
          "x-icon": "bookmark"
        }
      },
      "additionalProperties": false
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

[JSON Schema-2020-12]: https://json-schema.org/draft/2020-12
