# Form Template Element - Drop Down Single Select

UI - Drop Down Selection of a single entry from the defined enum.

Select one option from a selector styled as a dropdown menu.
Only one choice is allowed.

## Parent Elements

The Drop Down Single Select form element, can appear as a child of:

* [Section](section.md)

## Definition

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Definition: Drop Down Single Select"

    ```json
    {
      "$defs": {
        "dropDownSingleSelect": {
          "contentMediaType": "text/plain",
          "pattern": "^[^\\n]*$",
          "type": "string"
        }
      },
      "$schema": "https://json-schema.org/draft/2020-12/schema"
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

## Parameters

The Drop Down Single Select form element takes the following parameters:

<!---HTML START-->
<!-- markdownlint-disable -->
<div id="element_Drop_Down_Single_Select_parameters" style="padding-left:0px;padding-right:0px;padding-top:10px;padding-bottom:10px;overflow-x:auto;overflow-y:auto;width:100%;height:auto;">
<style>
#element_Drop_Down_Single_Select_parameters table {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', 'Fira Sans', 'Droid Sans', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }

#element_Drop_Down_Single_Select_parameters thead, tbody, tfoot, tr, td, th { border-style: none; }
 tr { background-color: transparent; }
#element_Drop_Down_Single_Select_parameters p { margin: 0; padding: 0; }
 #element_Drop_Down_Single_Select_parameters .gt_table { display: table; border-collapse: collapse; line-height: normal; margin-left: auto; margin-right: auto; color: #333333; font-size: 16px; font-weight: normal; font-style: normal; background-color: #FFFFFF; width: 100%; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; }
 #element_Drop_Down_Single_Select_parameters .gt_caption { padding-top: 4px; padding-bottom: 4px; }
 #element_Drop_Down_Single_Select_parameters .gt_title { color: #333333; font-size: 125%; font-weight: initial; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; border-bottom-color: #FFFFFF; border-bottom-width: 0; }
 #element_Drop_Down_Single_Select_parameters .gt_subtitle { color: #333333; font-size: 85%; font-weight: initial; padding-top: 3px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; border-top-color: #FFFFFF; border-top-width: 0; }
 #element_Drop_Down_Single_Select_parameters .gt_heading { background-color: #FFFFFF; text-align: center; border-bottom-color: #FFFFFF; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #element_Drop_Down_Single_Select_parameters .gt_bottom_border { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #element_Drop_Down_Single_Select_parameters .gt_col_headings { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #element_Drop_Down_Single_Select_parameters .gt_col_heading { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; overflow-x: hidden; }
 #element_Drop_Down_Single_Select_parameters .gt_column_spanner_outer { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; padding-top: 0; padding-bottom: 0; padding-left: 4px; padding-right: 4px; }
 #element_Drop_Down_Single_Select_parameters .gt_column_spanner_outer:first-child { padding-left: 0; }
 #element_Drop_Down_Single_Select_parameters .gt_column_spanner_outer:last-child { padding-right: 0; }
 #element_Drop_Down_Single_Select_parameters .gt_column_spanner { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; overflow-x: hidden; display: inline-block; width: 100%; }
 #element_Drop_Down_Single_Select_parameters .gt_spanner_row { border-bottom-style: hidden; }
 #element_Drop_Down_Single_Select_parameters .gt_group_heading { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: middle; text-align: left; }
 #element_Drop_Down_Single_Select_parameters .gt_empty_group_heading { padding: 0.5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: middle; }
 #element_Drop_Down_Single_Select_parameters .gt_from_md> :first-child { margin-top: 0; }
 #element_Drop_Down_Single_Select_parameters .gt_from_md> :last-child { margin-bottom: 0; }
 #element_Drop_Down_Single_Select_parameters .gt_row { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; margin: 10px; border-top-style: none; border-top-width: 1px; border-top-color: #D5D5D5; border-left-style: none; border-left-width: 1px; border-left-color: #D5D5D5; border-right-style: none; border-right-width: 1px; border-right-color: #D5D5D5; vertical-align: middle; overflow-x: hidden; }
 #element_Drop_Down_Single_Select_parameters .gt_stub { color: #333333; background-color: #89D3FE; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D5D5D5; padding-left: 5px; padding-right: 5px; }
 #element_Drop_Down_Single_Select_parameters .gt_stub_row_group { color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D3D3D3; padding-left: 5px; padding-right: 5px; vertical-align: top; }
 #element_Drop_Down_Single_Select_parameters .gt_row_group_first td { border-top-width: 2px; }
 #element_Drop_Down_Single_Select_parameters .gt_row_group_first th { border-top-width: 2px; }
 #element_Drop_Down_Single_Select_parameters .gt_striped { background-color: #EDF7FC; }
 #element_Drop_Down_Single_Select_parameters .gt_table_body { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #element_Drop_Down_Single_Select_parameters .gt_sourcenotes { color: #333333; background-color: #FFFFFF; border-bottom-style: none; border-bottom-width: 2px; border-bottom-color: #D3D3D3; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; }
 #element_Drop_Down_Single_Select_parameters .gt_sourcenote { font-size: 90%; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; text-align: left; }
 #element_Drop_Down_Single_Select_parameters .gt_left { text-align: left; }
 #element_Drop_Down_Single_Select_parameters .gt_center { text-align: center; }
 #element_Drop_Down_Single_Select_parameters .gt_right { text-align: right; font-variant-numeric: tabular-nums; }
 #element_Drop_Down_Single_Select_parameters .gt_font_normal { font-weight: normal; }
 #element_Drop_Down_Single_Select_parameters .gt_font_bold { font-weight: bold; }
 #element_Drop_Down_Single_Select_parameters .gt_font_italic { font-style: italic; }
 #element_Drop_Down_Single_Select_parameters .gt_super { font-size: 65%; }
 #element_Drop_Down_Single_Select_parameters .gt_footnote_marks { font-size: 75%; vertical-align: 0.4em; position: initial; }
 #element_Drop_Down_Single_Select_parameters .gt_asterisk { font-size: 100%; vertical-align: 0; }

</style>
<table style="table-layout: fixed;; width: 100%" class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<colgroup>
  <col style="width:10%;"/>
  <col style="width:50%;"/>
</colgroup>

<thead>

  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_title gt_font_normal">Drop Down Single Select</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

Parameters

</td>
  </tr>

</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>default</code></strong><br>The option from the <strong><code>enum</code></strong> which is chosen by default.<br>This <strong>MUST</strong> be a value defined in the <strong><code>enum</code></strong>.</th>
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
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left gt_striped"><code>default: &quot;option 1&quot;</code></td>
  </tr>
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
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left gt_striped"><code>description: &quot;Drop Down Single Selector.\nChoose a value from the options presented.&quot;</code></td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><strong><code>enum</code></strong><br>Sorted array of string values from which a single value can be selected.
Values must be presented in the order they appear in the array.
No value that is not in the array may be listed or presented.
Each item in the array <strong>MUST</strong> be  unique.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">yes</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped"><code>array</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Items</th>
    <td class="gt_row gt_left"><code>string</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Content Media Type</th>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc2046.html">text/plain</a></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Example</th>
    <td class="gt_row gt_left"><code>enum: [&quot;option 1&quot;, &quot;option 2&quot;, &quot;option 3&quot;]</code></td>
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
    <td class="gt_row gt_left"><code>title: &quot;Selector&quot;</code></td>
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
    <td class="gt_row gt_left"><code>x-guidance: &quot;It is recommended that a good choice be made.\nA bad choice could effect prospects of success.\nA good choice could improve them.\nSo make a good choice.&quot;</code></td>
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
    <td class="gt_row gt_left"><code>x-icon: &quot;emoji-happy&quot;</code></td>
  </tr>
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->

## Example Usage

This is an Example Form Template showing just the Drop Down Single Select form element, and its parents.

TODO
