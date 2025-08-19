# Form Template Element - Multi Line Text Entry [Markdown][CommonMark] List

## Functional Behavior

A growable list of multi line text entry field, with [Markdown][CommonMark] format.
Each entry:

* is a multiline [markdown][CommonMark] formatted string.
* supports [Markdown][CommonMark] Formatting, line breaks, or special characters.
* *MUST* be unique.
* *MUST* be a `multiLineTextEntryMarkdown` type Form element
and can be parameterized in the same way as that Element type.

## Visual Representation

Preferably, A minimum of one (and maximum of `maxItems`) rich text entry
boxes are presented.

The user can complete the entry as they would a single multi line text entry field.
They may choose to add another multiline text entry to the list, or remove an existing one.
The values they enter are encoded in the order they appear on screen,
in the order they appear in the array.

The Items should appear and be parameterizable in the same way the
base `multiLineTextEntryMarkdown` type Form element can be.

## Parent Elements

The Multi Line Text Entry [Markdown][CommonMark] List form element, can appear as a child of:

* [Section](section.md)

## Definition

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Definition: Multi Line Text Entry [Markdown][CommonMark] List"

    ```json
    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$defs": {
        "multiLineTextEntryMarkdown": {
          "contentMediaType": "text/markdown",
          "pattern": "^[\\S\\s]*$",
          "type": "string"
        },
        "multiLineTextEntryMarkdownList": {
          "items": {
            "$ref": "#/$defs/multiLineTextEntryMarkdown"
          },
          "type": "array",
          "uniqueItems": true
        }
      }
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

## Parameters

The Multi Line Text Entry [Markdown][CommonMark] List form element takes the following parameters:

<!---HTML START-->
<!-- markdownlint-disable -->
<div id="element_Multi_Line_Text_Entry_Markdown_List_parameters" style="padding-left:0px;padding-right:0px;padding-top:10px;padding-bottom:10px;overflow-x:auto;overflow-y:auto;width:100%;height:auto;">
<style>
#element_Multi_Line_Text_Entry_Markdown_List_parameters table {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', 'Fira Sans', 'Droid Sans', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }

#element_Multi_Line_Text_Entry_Markdown_List_parameters thead, tbody, tfoot, tr, td, th { border-style: none; }
 tr { background-color: transparent; }
#element_Multi_Line_Text_Entry_Markdown_List_parameters p { margin: 0; padding: 0; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_table { display: table; border-collapse: collapse; line-height: normal; margin-left: auto; margin-right: auto; color: #333333; font-size: 16px; font-weight: normal; font-style: normal; background-color: #FFFFFF; width: 100%; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_caption { padding-top: 4px; padding-bottom: 4px; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_title { color: #333333; font-size: 125%; font-weight: initial; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; border-bottom-color: #FFFFFF; border-bottom-width: 0; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_subtitle { color: #333333; font-size: 85%; font-weight: initial; padding-top: 3px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; border-top-color: #FFFFFF; border-top-width: 0; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_heading { background-color: #FFFFFF; text-align: center; border-bottom-color: #FFFFFF; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_bottom_border { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_col_headings { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_col_heading { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; overflow-x: hidden; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_column_spanner_outer { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; padding-top: 0; padding-bottom: 0; padding-left: 4px; padding-right: 4px; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_column_spanner_outer:first-child { padding-left: 0; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_column_spanner_outer:last-child { padding-right: 0; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_column_spanner { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; overflow-x: hidden; display: inline-block; width: 100%; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_spanner_row { border-bottom-style: hidden; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_group_heading { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: middle; text-align: left; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_empty_group_heading { padding: 0.5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: middle; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_from_md> :first-child { margin-top: 0; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_from_md> :last-child { margin-bottom: 0; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_row { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; margin: 10px; border-top-style: none; border-top-width: 1px; border-top-color: #D5D5D5; border-left-style: none; border-left-width: 1px; border-left-color: #D5D5D5; border-right-style: none; border-right-width: 1px; border-right-color: #D5D5D5; vertical-align: middle; overflow-x: hidden; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_stub { color: #333333; background-color: #89D3FE; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D5D5D5; padding-left: 5px; padding-right: 5px; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_stub_row_group { color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D3D3D3; padding-left: 5px; padding-right: 5px; vertical-align: top; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_row_group_first td { border-top-width: 2px; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_row_group_first th { border-top-width: 2px; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_striped { background-color: #EDF7FC; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_table_body { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_sourcenotes { color: #333333; background-color: #FFFFFF; border-bottom-style: none; border-bottom-width: 2px; border-bottom-color: #D3D3D3; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_sourcenote { font-size: 90%; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; text-align: left; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_left { text-align: left; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_center { text-align: center; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_right { text-align: right; font-variant-numeric: tabular-nums; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_font_normal { font-weight: normal; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_font_bold { font-weight: bold; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_font_italic { font-style: italic; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_super { font-size: 65%; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_footnote_marks { font-size: 75%; vertical-align: 0.4em; position: initial; }
 #element_Multi_Line_Text_Entry_Markdown_List_parameters .gt_asterisk { font-size: 100%; vertical-align: 0; }

</style>
<table style="table-layout: fixed;; width: 100%" class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<colgroup>
  <col style="width:10%;"/>
  <col style="width:50%;"/>
</colgroup>

<thead>

  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_title gt_font_normal">Multi Line Text Entry <a href="https://spec.commonmark.org/0.31.2/">Markdown</a> List</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

Parameters

</td>
  </tr>

</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>default</code></strong><br>Default Array of text can be supplied.</th>
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
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left"><code>default: [&quot;# Chapter 1\n\n## The beginning.\n\nOnce upon a time.&quot;, &quot;...&quot;, &quot;# Chapter 93\n\n## The exciting finale.\n\nMaybe the real treasure was the friends we made along the way.\n\n***The End***\n\n(or is it...)&quot;]</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>description</code></strong><br>The description of the field presented to the user during data entry.</th>
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
    <th class="gt_row gt_left gt_stub">Content Media Type</th>
    <td class="gt_row gt_left gt_striped"><a href="https://spec.commonmark.org/0.31.2/">text/markdown;</a> <a href="https://handlebarsjs.com/">template=handlebars</a></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left"><code>description: &quot;A set of chapters used to tell your story.&quot;</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>maxItems</code></strong><br>An array instance is valid against &quot;maxItems&quot; if its size is less than, or equal to, the value of this keyword.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left gt_striped">yes</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left"><code>integer</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left gt_striped"><code>maxItems: 100</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>minItems</code></strong><br>An array instance is valid against &quot;minItems&quot; if its size is greater than, or equal to, the value of this keyword.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">optional</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped"><code>integer</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left"><code>minItems: 1</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>title</code></strong><br>The label attached to the field.</th>
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
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left"><code>title: &quot;Chapters&quot;</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>x-guidance</code></strong><br>Long form <a href="https://spec.commonmark.org/0.31.2/">Markdown</a> formatted description to give guidance about how the field is to be completed.</th>
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
    <th class="gt_row gt_left gt_stub">Content Media Type</th>
    <td class="gt_row gt_left gt_striped"><a href="https://spec.commonmark.org/0.31.2/">text/markdown;</a> <a href="https://handlebarsjs.com/">template=handlebars</a></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left"><code>x-guidance: &quot;Tell us your never-ending story.&quot;</code></td>
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
    <td class="gt_row gt_left"><code>x-icon: &quot;collection&quot;</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>x-placeholder</code></strong><br>Placeholder text to display inside the field if it is empty.
Unlike <code>default</code> it does not provide a default value for the field.</th>
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
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left gt_striped"><code>x-placeholder: &quot;There is a default, so this placeholder won't show.&quot;</code></td>
  </tr>
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->

## Example Usage

This is an Example Form Template showing just the Multi Line Text Entry [Markdown][CommonMark] List form element, and its parents.

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: "

    ```json
    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "title": "multiLineTextEntryMarkdownList Example",
      "description": "An example of the multiLineTextEntryMarkdownList Element, and it's parents.",
      "$defs": {
        "multiLineTextEntryMarkdown": {
          "contentMediaType": "text/markdown",
          "pattern": "^[\\S\\s]*$",
          "type": "string"
        },
        "multiLineTextEntryMarkdownList": {
          "items": {
            "$ref": "#/$defs/multiLineTextEntryMarkdown"
          },
          "type": "array",
          "uniqueItems": true
        },
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
            "exampleMultiLineTextEntryMarkdownList": {
              "$ref": "#/$defs/multiLineTextEntryMarkdownList",
              "default": [
                "# Chapter 1\n\n## The beginning.\n\nOnce upon a time.",
                "...",
                "# Chapter 93\n\n## The exciting finale.\n\nMaybe the real treasure was the friends we made along the way.\n\n***The End***\n\n(or is it...)"
              ],
              "description": "A set of chapters used to tell your story.",
              "items": {
                "default": "# My Story\n\nOnce **upon** a *time*...",
                "description": "Tell a story to the reader.",
                "maxLength": 5000,
                "minLength": 20,
                "title": "Story",
                "x-guidance": "Engaging stories are better than boring ones.\nTry to be engaging.",
                "x-icon": "book-open",
                "x-placeholder": "# ..."
              },
              "maxItems": 100,
              "minItems": 1,
              "title": "Chapters",
              "x-guidance": "Tell us your never-ending story.",
              "x-icon": "collection",
              "x-placeholder": "There is a default, so this placeholder won't show."
            },
            "exampleSection": {
              "$ref": "#/$defs/section",
              "properties": {
                "exampleMultiLineTextEntryMarkdownList": {
                  "$ref": "#/$defs/multiLineTextEntryMarkdownList",
                  "default": [
                    "# Chapter 1\n\n## The beginning.\n\nOnce upon a time.",
                    "...",
                    "# Chapter 93\n\n## The exciting finale.\n\nMaybe the real treasure was the friends we made along the way.\n\n***The End***\n\n(or is it...)"
                  ],
                  "description": "A set of chapters used to tell your story.",
                  "items": {
                    "default": "# My Story\n\nOnce **upon** a *time*...",
                    "description": "Tell a story to the reader.",
                    "maxLength": 5000,
                    "minLength": 20,
                    "title": "Story",
                    "x-guidance": "Engaging stories are better than boring ones.\nTry to be engaging.",
                    "x-icon": "book-open",
                    "x-placeholder": "# ..."
                  },
                  "maxItems": 100,
                  "minItems": 1,
                  "title": "Chapters",
                  "x-guidance": "Tell us your never-ending story.",
                  "x-icon": "collection",
                  "x-placeholder": "There is a default, so this placeholder won't show."
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

[CommonMark]: https://spec.commonmark.org/0.31.2/
