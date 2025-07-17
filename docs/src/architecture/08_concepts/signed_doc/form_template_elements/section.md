# Form Template Element - Section

UI - Logical Document Sub-Section Break.
Subsections are logical breaks in the form structure.
The ONLY Element that can appear in the root of a Form is a [`section`](metadata.md#section).
[`section`](metadata.md#section) can be nested arbitrarily deep.
The form presentation can decide how deep, and what formatting it used
for each section at each level, however, even if the form is flattened and the section not
displayed, then data must still follow the section nesting.

## Parent Elements

The Section form element, can appear as a child of:

* The Root Object of the [JSON Schema][JSON Schema-2020-12]
* [Section](section.md)

## Definition

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Definition: Section"

    ```json
    {
      "$defs": {
        "section": {
          "additionalProperties": false,
          "type": "object"
        }
      },
      "$schema": "https://json-schema.org/draft/2020-12/schema"
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
 #element_Section_parameters .gt_striped { background-color: #EDF7FC; }
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
<table class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<thead>

  <tr class="gt_heading">
    <td colspan="3" class="gt_heading gt_title gt_font_normal">Section</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="3" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

Parameters

</td>
  </tr>

</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">description</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">optional</td>
    <td class="gt_row gt_left">Is the parameter required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped">string</td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Type of the parameter.</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">properties</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">yes</td>
    <td class="gt_row gt_left">Is the parameter required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped">string</td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Type of the parameter.</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">required</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">optional</td>
    <td class="gt_row gt_left">Is the parameter required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped">string</td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Type of the parameter.</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">title</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">yes</td>
    <td class="gt_row gt_left">Is the parameter required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped">string</td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Type of the parameter.</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">x-flatten</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">optional</td>
    <td class="gt_row gt_left">Is the parameter required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped">boolean</td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Type of the parameter.</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">x-icon</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">optional</td>
    <td class="gt_row gt_left">Is the parameter required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left gt_striped">string</td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Type of the parameter.</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Choices</th>
    <td class="gt_row gt_left">['academic-cap', 'ada', 'adjustments', 'all spaces menu', 'all spaces menu-1', 'annotation', 'archive', 'arrow-circle-down', 'arrow-circle-left', 'arrow-circle-right', 'arrow-circle-up', 'arrow-down', 'arrow-left', 'arrow-narrow-down', 'arrow-narrow-left', 'arrow-narrow-right', 'arrow-narrow-up', 'arrow-right', 'arrow-triangle-down', 'arrow-triangle-up', 'arrow-up', 'arrows-expand', 'at-symbol', 'avatar_placeholder', 'backspace', 'badge-check', 'ban', 'beaker', 'bell', 'book-open', 'bookmark', 'bookmark-alt', 'bottom-main-content', 'bottom-rail-toggle', 'briefcase', 'cake', 'calculator', 'calendar', 'camera', 'cash', 'chart-bar', 'chart-pie', 'chart-square-bar', 'chat', 'chat-alt', 'chat-alt-2', 'check', 'check-circle', 'chevron-double-down', 'chevron-double-left', 'chevron-double-right', 'chevron-double-up', 'chevron-down', 'chevron-left', 'chevron-right', 'chevron-up', 'chip', 'clipboard', 'clipboard-check', 'clipboard-copy', 'clipboard-list', 'clock', 'cloud', 'cloud-download', 'cloud-upload', 'code', 'cog-gear', 'collection', 'color-swatch', 'credit-card', 'cube', 'cube-transparent', 'currency-bangladeshi', 'currency-dollar', 'currency-euro', 'currency-pound', 'currency-rupee', 'currency-yen', 'cursor-click', 'curved-arrow-right', 'database', 'desktop-computer', 'device-mobile', 'device-tablet', 'document', 'document-add', 'document-remove', 'document-report', 'document-search', 'document-text', 'dots-circle-horizontal', 'dots-horizontal', 'dots-vertical', 'double_check', 'download', 'duplicate', 'emoji-happy', 'emoji-sad', 'exclamation', 'exclamation-circle', 'external-link', 'eye', 'eye-off', 'facebook', 'fast-forward', 'film', 'filter', 'finger-print', 'fire', 'flag', 'folder', 'folder-add', 'folder-download', 'folder-open', 'folder-remove', 'fund', 'gift', 'globe', 'globe-alt', 'hand', 'hashtag', 'heart', 'home', 'icon-user-remove', 'identification', 'inbox', 'inbox-in', 'information-circle', 'key', 'left-rail-toggle', 'library', 'light-bulb', 'lightning-bolt', 'link', 'linkedin', 'location-marker', 'lock-closed', 'lock-open', 'login', 'logout', 'mail', 'mail-open', 'map', 'maximize-toggle', 'menu', 'menu-alt-1', 'menu-alt-2', 'menu-alt-3', 'menu-alt-4', 'microphone', 'minimize-toggle', 'minus', 'minus-circle', 'moon', 'move-item', 'music-note', 'newspaper', 'node-closed', 'node-line', 'node-line-end', 'node-open', 'office-building', 'paper-airplane', 'paper-clip', 'pause', 'pencil', 'pencil-alt', 'phone', 'phone-incoming', 'phone-missed-call', 'phone-outgoing', 'photograph', 'play', 'plus', 'plus_circle_filled', 'plus_circle_outlined', 'presentation-chart-bar', 'presentation-chart-line', 'printer', 'progress-track-warning', 'puzzle', 'qrcode', 'question-mark-circle', 'receipt-refund', 'receipt-tax', 'reddit', 'refresh', 'reply', 'rewind', 'right-rail-toggle', 'rss', 'rt_bold', 'rt_decrease_indent', 'rt_heading', 'rt_increase_indent', 'rt_italic', 'rt_ordered_list', 'rt_unordered_list', 'save', 'save-as', 'scale', 'scissors', 'search', 'search-circle', 'selector', 'send-airplane', 'server', 'share', 'shield-check', 'shield-exclamation', 'shopping-bag', 'shopping-cart', 'sm-view-grid-add', 'sort-ascending', 'sort-descending', 'sparkles', 'speakerphone', 'star_filled', 'star_outlined', 'status-offline', 'status-online', 'stop', 'summary', 'sun', 'support', 'switch-horizontal', 'switch-vertical', 'table', 'tag', 'template', 'terminal', 'thumb-down', 'thumb-up', 'ticket', 'top-bar', 'top-bar-filled', 'translate', 'trash', 'trending-down', 'trending-up', 'truck', 'unlink', 'upload', 'user', 'user-add', 'user-circle', 'user-group', 'users', 'variable', 'video-camera', 'view-boards', 'view-grid', 'view-list', 'volume-off', 'volume-up', 'vote', 'wallet', 'wifi', 'x', 'x-circle', 'xTwitter', 'zoom-in', 'zoom-out']</td>
    <td class="gt_row gt_left">All the choices.</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">x-order</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left gt_striped">yes</td>
    <td class="gt_row gt_left gt_striped">Is the parameter required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Type</th>
    <td class="gt_row gt_left">string</td>
    <td class="gt_row gt_left"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Type of the parameter.</td>
  </tr>
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->


## Example Usage

This is an Example Form Template showing just the Section form element, and its parents.

TODO

[JSON Schema-2020-12]: https://json-schema.org/draft/2020-12
