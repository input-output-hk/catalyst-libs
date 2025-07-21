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

## Parent Elements

The Single Grouped Tag Selector form element, can appear as a child of:

* [Section](section.md)

## Definition

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Definition: Single Grouped Tag Selector"

    ```json
    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$defs": {
        "singleGroupedTagSelector": {
          "additionalProperties": false,
          "required": [
            "group",
            "tag"
          ],
          "type": "object"
        }
      }
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

## Parameters

The Single Grouped Tag Selector form element takes the following parameters:

<!---HTML START-->
<!-- markdownlint-disable -->
<div id="element_Single_Grouped_Tag_Selector_parameters" style="padding-left:0px;padding-right:0px;padding-top:10px;padding-bottom:10px;overflow-x:auto;overflow-y:auto;width:100%;height:auto;">
<style>
#element_Single_Grouped_Tag_Selector_parameters table {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', 'Fira Sans', 'Droid Sans', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }

#element_Single_Grouped_Tag_Selector_parameters thead, tbody, tfoot, tr, td, th { border-style: none; }
 tr { background-color: transparent; }
#element_Single_Grouped_Tag_Selector_parameters p { margin: 0; padding: 0; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_table { display: table; border-collapse: collapse; line-height: normal; margin-left: auto; margin-right: auto; color: #333333; font-size: 16px; font-weight: normal; font-style: normal; background-color: #FFFFFF; width: 100%; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_caption { padding-top: 4px; padding-bottom: 4px; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_title { color: #333333; font-size: 125%; font-weight: initial; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; border-bottom-color: #FFFFFF; border-bottom-width: 0; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_subtitle { color: #333333; font-size: 85%; font-weight: initial; padding-top: 3px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; border-top-color: #FFFFFF; border-top-width: 0; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_heading { background-color: #FFFFFF; text-align: center; border-bottom-color: #FFFFFF; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_bottom_border { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_col_headings { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_col_heading { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; overflow-x: hidden; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_column_spanner_outer { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; padding-top: 0; padding-bottom: 0; padding-left: 4px; padding-right: 4px; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_column_spanner_outer:first-child { padding-left: 0; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_column_spanner_outer:last-child { padding-right: 0; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_column_spanner { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; overflow-x: hidden; display: inline-block; width: 100%; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_spanner_row { border-bottom-style: hidden; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_group_heading { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: middle; text-align: left; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_empty_group_heading { padding: 0.5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: middle; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_from_md> :first-child { margin-top: 0; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_from_md> :last-child { margin-bottom: 0; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_row { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; margin: 10px; border-top-style: none; border-top-width: 1px; border-top-color: #D5D5D5; border-left-style: none; border-left-width: 1px; border-left-color: #D5D5D5; border-right-style: none; border-right-width: 1px; border-right-color: #D5D5D5; vertical-align: middle; overflow-x: hidden; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_stub { color: #333333; background-color: #89D3FE; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D5D5D5; padding-left: 5px; padding-right: 5px; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_stub_row_group { color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D3D3D3; padding-left: 5px; padding-right: 5px; vertical-align: top; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_row_group_first td { border-top-width: 2px; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_row_group_first th { border-top-width: 2px; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_striped { background-color: #EDF7FC; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_table_body { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_sourcenotes { color: #333333; background-color: #FFFFFF; border-bottom-style: none; border-bottom-width: 2px; border-bottom-color: #D3D3D3; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_sourcenote { font-size: 90%; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; text-align: left; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_left { text-align: left; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_center { text-align: center; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_right { text-align: right; font-variant-numeric: tabular-nums; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_font_normal { font-weight: normal; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_font_bold { font-weight: bold; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_font_italic { font-style: italic; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_super { font-size: 65%; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_footnote_marks { font-size: 75%; vertical-align: 0.4em; position: initial; }
 #element_Single_Grouped_Tag_Selector_parameters .gt_asterisk { font-size: 100%; vertical-align: 0; }

</style>
<table style="table-layout: fixed;; width: 100%" class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<colgroup>
  <col style="width:10%;"/>
  <col style="width:50%;"/>
</colgroup>

<thead>

  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_title gt_font_normal">Single Grouped Tag Selector</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

Parameters

</td>
  </tr>

</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>description</code></strong><br>The description of the field presented to the user during data entry.</th>
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
    <th class="gt_group_heading" colspan="2"><strong><code>oneOf_groupedTags</code></strong><br>A set of tags with a group selector.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left gt_striped">excluded</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left"><code>array</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Items</th>
    <td class="gt_row gt_left gt_striped">optional_property_type=None description='\tAn array of grouped tag objects, of which one can be selected.\n\tEach object <em>MUST</em> have the form:\n\t\n\t<code>json\n\t&quot;properties&quot;: {\n\t\t&quot;group&quot;: {\n\t\t\t&quot;$ref&quot;: &quot;$def/tagGroup&quot;,\n\t\t\t&quot;const&quot;: &lt;group name string&gt;\n\t\t},\n\t\t&quot;tag&quot;: {\n\t\t\t&quot;$ref&quot;: &quot;$def/tagSelection&quot;,\n\t\t\t&quot;enum&quot;: [\n\t\t\t\t&lt;tag 1 string&gt;,\n\t\t\t\t&lt;tag 2 string&gt;,\n\t\t\t\t...\n\t\t\t]\n\t\t}\n\t}\n\t</code>' required=&lt;OptionalField.excluded: 'excluded'&gt; type='object' items=None choices=None format=None content_media_type=None pattern=None min_length=None minimum=None maximum=None example=None element_name='Unknown' property_type='object' name='Unknown'</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>title</code></strong><br>The label attached to the field.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">yes</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped"><code>string</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Content Media Type</th>
    <td class="gt_row gt_left"><a href="https://www.rfc-editor.org/rfc/rfc2046.html">text/plain</a></td>
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
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->

## Example Usage

This is an Example Form Template showing just the Single Grouped Tag Selector form element, and its parents.

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: "

    ```json
    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "title": "singleGroupedTagSelector Example",
      "description": "An example of the singleGroupedTagSelector Element, and it's parents.",
      "$defs": {
        "section": {
          "additionalProperties": false,
          "type": "object"
        },
        "singleGroupedTagSelector": {
          "additionalProperties": false,
          "required": [
            "group",
            "tag"
          ],
          "type": "object"
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
                "exampleSingleGroupedTagSelector": {
                  "$ref": "#/$defs/singleGroupedTagSelector",
                  "properties": {}
                }
              }
            },
            "exampleSingleGroupedTagSelector": {
              "$ref": "#/$defs/singleGroupedTagSelector",
              "properties": {}
            }
          }
        }
      },
      "additionalProperties": false
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
