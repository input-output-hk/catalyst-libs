# Presentation Template Specification

Data in Project Catalyst is divided into:

* Form captured data, which is controlled and validated by FormTemplates; and
* Dynamic System Data, which is derived by the system based on activity or on-chain information.

There needs to be a way to present this data in a meaningful, cross platform, structured approach.

This is achieved with this Presentation Template.

Each Presentation Template defines a presentation for a particular Card of information.
The UI composes its display based on these cards, which allow for both static content
and dynamic content defined within the system to be merged and presented.

This helps divide the system by intent, data capture/generation and presentation.

Presentation Templates *DO NOT* capture user flows.
They *ONLY* define how data could be displayed by combining source for a particular purpose.
The purpose is derived solely by the cards id.

## [JSON Schema][JSON Schema-2020-12]

Presentation Template payloads all follow the same structure,
which is defined by [JSON Schema version 2020-12][JSON Schema-2020-12].

### Schema

The Presentation template schema has the following format.

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "Presentation Template Schema"

    * [presentation_template.schema.json](schema/presentation_template.schema.json)

    ``` json
    {{ include_file('./schema/presentation_template.schema.json', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line -->

#### `"$schema"`

This defines that the template is a standard [JSON Schema version 2020-12][JSON Schema-2020-12] document.
Any document derived from the template is only valid if it validates
according to [JSON Schema][JSON Schema-2020-12] validation rules against the template.

#### `"maintainers"`

Is a list of the entities who have made or updated the template.
It is optional, but if it is included, it *MUST* be an array of objects.
Each object must have two fields:

* `"name"` : The name of the individual maintainer or group responsible for the template.
* `"url"` : A link where more information about the maintainer or group can be found.
    If there is no link, the field is still present, but an empty string (`""`).

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

#### `"$defs"`

Defines the fields within the properties that define the presentation template.

#### `"type": "object"`

The Presentation Template is an `object` type.

#### `"properties"`

The `"properties"` section defines each field that is used to define the Presentation Template.

#### `"additionalProperties": false`

This Templates is exhaustively defined.
It is not permissible to add fields to a document,
that are not present in this schema.

`"additionalProperties": false`

## Example Presentation Template

 *This Specification is draft, and there are currently no examples.*

## Presentation Template Cards

<!---HTML START-->
<!-- markdownlint-disable -->
<div id="cards_Unnamed" style="padding-left:0px;padding-right:0px;padding-top:10px;padding-bottom:10px;overflow-x:auto;overflow-y:auto;width:auto;height:auto;">
<style>
#cards_Unnamed table {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', 'Fira Sans', 'Droid Sans', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }

#cards_Unnamed thead, tbody, tfoot, tr, td, th { border-style: none; }
 tr { background-color: transparent; }
#cards_Unnamed p { margin: 0; padding: 0; }
 #cards_Unnamed .gt_table { display: table; border-collapse: collapse; line-height: normal; margin-left: auto; margin-right: auto; color: #333333; font-size: 16px; font-weight: normal; font-style: normal; background-color: #FFFFFF; width: auto; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; }
 #cards_Unnamed .gt_caption { padding-top: 4px; padding-bottom: 4px; }
 #cards_Unnamed .gt_title { color: #333333; font-size: 125%; font-weight: initial; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; border-bottom-color: #FFFFFF; border-bottom-width: 0; }
 #cards_Unnamed .gt_subtitle { color: #333333; font-size: 85%; font-weight: initial; padding-top: 3px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; border-top-color: #FFFFFF; border-top-width: 0; }
 #cards_Unnamed .gt_heading { background-color: #FFFFFF; text-align: center; border-bottom-color: #FFFFFF; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #cards_Unnamed .gt_bottom_border { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #cards_Unnamed .gt_col_headings { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #cards_Unnamed .gt_col_heading { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; overflow-x: hidden; }
 #cards_Unnamed .gt_column_spanner_outer { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; padding-top: 0; padding-bottom: 0; padding-left: 4px; padding-right: 4px; }
 #cards_Unnamed .gt_column_spanner_outer:first-child { padding-left: 0; }
 #cards_Unnamed .gt_column_spanner_outer:last-child { padding-right: 0; }
 #cards_Unnamed .gt_column_spanner { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; overflow-x: hidden; display: inline-block; width: 100%; }
 #cards_Unnamed .gt_spanner_row { border-bottom-style: hidden; }
 #cards_Unnamed .gt_group_heading { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: middle; text-align: left; }
 #cards_Unnamed .gt_empty_group_heading { padding: 0.5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: middle; }
 #cards_Unnamed .gt_from_md> :first-child { margin-top: 0; }
 #cards_Unnamed .gt_from_md> :last-child { margin-bottom: 0; }
 #cards_Unnamed .gt_row { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; margin: 10px; border-top-style: none; border-top-width: 1px; border-top-color: #D5D5D5; border-left-style: none; border-left-width: 1px; border-left-color: #D5D5D5; border-right-style: none; border-right-width: 1px; border-right-color: #D5D5D5; vertical-align: middle; overflow-x: hidden; }
 #cards_Unnamed .gt_stub { color: #333333; background-color: #89D3FE; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D5D5D5; padding-left: 5px; padding-right: 5px; }
 #cards_Unnamed .gt_stub_row_group { color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D3D3D3; padding-left: 5px; padding-right: 5px; vertical-align: top; }
 #cards_Unnamed .gt_row_group_first td { border-top-width: 2px; }
 #cards_Unnamed .gt_row_group_first th { border-top-width: 2px; }
 #cards_Unnamed .gt_striped { color: #333333; background-color: #EDF7FC; }
 #cards_Unnamed .gt_table_body { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #cards_Unnamed .gt_sourcenotes { color: #333333; background-color: #FFFFFF; border-bottom-style: none; border-bottom-width: 2px; border-bottom-color: #D3D3D3; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; }
 #cards_Unnamed .gt_sourcenote { font-size: 90%; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; text-align: left; }
 #cards_Unnamed .gt_left { text-align: left; }
 #cards_Unnamed .gt_center { text-align: center; }
 #cards_Unnamed .gt_right { text-align: right; font-variant-numeric: tabular-nums; }
 #cards_Unnamed .gt_font_normal { font-weight: normal; }
 #cards_Unnamed .gt_font_bold { font-weight: bold; }
 #cards_Unnamed .gt_font_italic { font-style: italic; }
 #cards_Unnamed .gt_super { font-size: 65%; }
 #cards_Unnamed .gt_footnote_marks { font-size: 75%; vertical-align: 0.4em; position: initial; }
 #cards_Unnamed .gt_asterisk { font-size: 100%; vertical-align: 0; }

</style>
<table class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<thead>

  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_title gt_font_normal">Defined Presentation Cards</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="2" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

All Presentation Card Names that may be defined by Presentation Templates.

</td>
  </tr>
<tr class="gt_col_headings">
  <th class="gt_col_heading gt_columns_bottom_border gt_left" rowspan="1" colspan="1" scope="col" id=""></th>
  <th class="gt_col_heading gt_columns_bottom_border gt_left" rowspan="1" colspan="1" scope="col" id="cards_Unnamed-Value"></th>
</tr>
</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><h3>Draft Proposal Summary</h3>
<p>A Summary of a draft proposal.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Card ID</th>
    <td class="gt_row gt_left"><code>draft-proposal-summary</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Available Documents</th>
    <td class="gt_row gt_left gt_striped"><ul>
<li><a href="../docs/brand_parameters/">Brand Parameters</a></li>
<li><a href="../docs/campaign_parameters/">Campaign Parameters</a></li>
<li><a href="../docs/category_parameters/">Category Parameters</a></li>
<li><a href="../docs/proposal/">Proposal</a></li>
<li><a href="../docs/proposal_form_template/">Proposal Form Template</a></li>
</ul>
</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="2"><h3>Proposal Contest Summary</h3>
<p>A Summary shown when selecting proposals to vote on.</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Card ID</th>
    <td class="gt_row gt_left"><code>proposal-contest-summary</code></td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Available Documents</th>
    <td class="gt_row gt_left gt_striped"><ul>
<li><a href="../docs/brand_parameters/">Brand Parameters</a></li>
<li><a href="../docs/campaign_parameters/">Campaign Parameters</a></li>
<li><a href="../docs/category_parameters/">Category Parameters</a></li>
<li><a href="../docs/proposal/">Proposal</a></li>
<li><a href="../docs/proposal_form_template/">Proposal Form Template</a></li>
<li><a href="../docs/proposal_submission_action/">Proposal Submission Action</a></li>
</ul>
</td>
  </tr>
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->

[JSON Schema-2020-12]: https://json-schema.org/draft/2020-12
