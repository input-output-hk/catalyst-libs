# Catalyst Signed Document Specification

## Abstract

Project Catalyst requires a verifiable data format for the publication and validation of
large volumes of off chain information.

The Catalyst Signed Documents Specification is based on [COSE][RFC9052]
and provides the basis of this document specification.

## Motivation

As Project Catalyst decentralizes via both on-chain and off-chain mechanisms, a reliable,
standardized process for authenticating documents and their relationships is required.

## Specification

Project Catalyst generates a large volume of off chain information.
This information requires similar guarantees as on-chain data.
It needs to be verifiably published and also immutable.
However, we also require the ability to publish new versions of documents,
and for documents to be able to securely reference one another.

Catalyst Signed Documents are based on [COSE][RFC9052].
Specifically, the [COSE Sign][RFC9052-CoseSign] format is used.
This allows one or more signatures to be attached to the same document.

While every Catalyst Signed Document is a valid [COSE Sign][RFC9052-CoseSign] format document,
not every [COSE Sign][RFC9052-CoseSign] format document is a valid Catalyst Signed Document.
The following restrictions apply:

### Unprotected Headers are not permitted

It is a requirement that any document that contains exactly the same data, must produce the same
catalyst signed document.
This means that unprotected headers, which do not form part of the data protected by
the signature are not permitted.
Any document which contains any unprotected headers is not a valid Catalyst Signed Document,
even though it may be a valid [COSE Sign][RFC9052-CoseSign] formatted document.

### Only defined metadata and [COSE][RFC9052] Headers are allowed

Each document type, defines a set of metadata and the [COSE][RFC9052] Headers which are allowed in that document type.
Even if the Catalyst Signed document metadata exists in this specification, IF it is not defined as
a valid metadata or [COSE][RFC9052] Header field for that particular document it may not be present.
Unexpected but otherwise valid Metadata or [COSE][RFC9052] Header fields invalidate the Catalyst Signed Document.

### No undefined metadata or unused [COSE][RFC9052] Headers may be present

[COSE][RFC9052] Header Fields which are defined by the [COSE][RFC9052] Specification, but are NOT defined as part of a
Catalyst Signed Document may not be present.
Any such [COSE][RFC9052] Header Fields present in the document render it an invalid Catalyst Signed Document.

Any metadata field that is not defined in this specification may not be present in any protected header.
Unrecognized metadata fields in a document render it an invalid Catalyst Signed Document.

### [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] MUST be used

The Catalyst Signed Document **MUST** be encoded using [CBOR Deterministic Encoding][CBOR-LFD-ENCODING].
The "length-first core deterministic encoding requirements" variant of deterministic encoding *MUST* be used.

### Signed Document [CDDL][RFC8610] Definition

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [signed_document.cddl](cddl/signed_document.cddl)

    ``` cddl
    {{ include_file('./cddl/signed_document.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### [COSE Header Parameters][RFC9052-HeaderParameters]

[COSE][RFC9052] documents define a set of standard [COSE header parameters][RFC9052-HeaderParameters].
All [COSE Header Parameters][RFC9052-HeaderParameters] are protected and
*MUST* appear in the protected headers section of the document.
The [COSE header parameters][RFC9052-HeaderParameters] defined and used by Catalyst Signed Documents are as follows:

#### `content type`

Media Type/s allowed in the Payload

<!---HTML START-->
<!-- markdownlint-disable -->
<div id="spec_content_type" style="padding-left:0px;padding-right:0px;padding-top:10px;padding-bottom:10px;overflow-x:auto;overflow-y:auto;width:100%;height:auto;">
<style>
#spec_content_type table {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', 'Fira Sans', 'Droid Sans', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }

#spec_content_type thead, tbody, tfoot, tr, td, th { border-style: none; }
 tr { background-color: transparent; }
#spec_content_type p { margin: 0; padding: 0; }
 #spec_content_type .gt_table { display: table; border-collapse: collapse; line-height: normal; margin-left: auto; margin-right: auto; color: #333333; font-size: 16px; font-weight: normal; font-style: normal; background-color: #FFFFFF; width: 100%; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; }
 #spec_content_type .gt_caption { padding-top: 4px; padding-bottom: 4px; }
 #spec_content_type .gt_title { color: #333333; font-size: 125%; font-weight: initial; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; border-bottom-color: #FFFFFF; border-bottom-width: 0; }
 #spec_content_type .gt_subtitle { color: #333333; font-size: 85%; font-weight: initial; padding-top: 3px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; border-top-color: #FFFFFF; border-top-width: 0; }
 #spec_content_type .gt_heading { background-color: #FFFFFF; text-align: center; border-bottom-color: #FFFFFF; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #spec_content_type .gt_bottom_border { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #spec_content_type .gt_col_headings { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #spec_content_type .gt_col_heading { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; overflow-x: hidden; }
 #spec_content_type .gt_column_spanner_outer { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; padding-top: 0; padding-bottom: 0; padding-left: 4px; padding-right: 4px; }
 #spec_content_type .gt_column_spanner_outer:first-child { padding-left: 0; }
 #spec_content_type .gt_column_spanner_outer:last-child { padding-right: 0; }
 #spec_content_type .gt_column_spanner { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; overflow-x: hidden; display: inline-block; width: 100%; }
 #spec_content_type .gt_spanner_row { border-bottom-style: hidden; }
 #spec_content_type .gt_group_heading { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: middle; text-align: left; }
 #spec_content_type .gt_empty_group_heading { padding: 0.5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: middle; }
 #spec_content_type .gt_from_md> :first-child { margin-top: 0; }
 #spec_content_type .gt_from_md> :last-child { margin-bottom: 0; }
 #spec_content_type .gt_row { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; margin: 10px; border-top-style: none; border-top-width: 1px; border-top-color: #D5D5D5; border-left-style: none; border-left-width: 1px; border-left-color: #D5D5D5; border-right-style: none; border-right-width: 1px; border-right-color: #D5D5D5; vertical-align: middle; overflow-x: hidden; }
 #spec_content_type .gt_stub { color: #333333; background-color: #89D3FE; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D5D5D5; padding-left: 5px; padding-right: 5px; }
 #spec_content_type .gt_stub_row_group { color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D3D3D3; padding-left: 5px; padding-right: 5px; vertical-align: top; }
 #spec_content_type .gt_row_group_first td { border-top-width: 2px; }
 #spec_content_type .gt_row_group_first th { border-top-width: 2px; }
 #spec_content_type .gt_striped { color: #333333; background-color: #EDF7FC; }
 #spec_content_type .gt_table_body { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #spec_content_type .gt_grand_summary_row { color: #333333; background-color: #D5D5D5; text-transform: inherit; padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; }
 #spec_content_type .gt_first_grand_summary_row_bottom { border-top-style: double; border-top-width: 6px; border-top-color: #D3D3D3; }
 #spec_content_type .gt_last_grand_summary_row_top { border-bottom-style: double; border-bottom-width: 6px; border-bottom-color: #D3D3D3; }
 #spec_content_type .gt_sourcenotes { color: #333333; background-color: #FFFFFF; border-bottom-style: none; border-bottom-width: 2px; border-bottom-color: #D3D3D3; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; }
 #spec_content_type .gt_sourcenote { font-size: 90%; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; text-align: left; }
 #spec_content_type .gt_left { text-align: left; }
 #spec_content_type .gt_center { text-align: center; }
 #spec_content_type .gt_right { text-align: right; font-variant-numeric: tabular-nums; }
 #spec_content_type .gt_font_normal { font-weight: normal; }
 #spec_content_type .gt_font_bold { font-weight: bold; }
 #spec_content_type .gt_font_italic { font-style: italic; }
 #spec_content_type .gt_super { font-size: 65%; }
 #spec_content_type .gt_footnote_marks { font-size: 75%; vertical-align: 0.4em; position: initial; }
 #spec_content_type .gt_asterisk { font-size: 100%; vertical-align: 0; }

</style>
<table class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<thead>

  <tr class="gt_heading">
    <td colspan="3" class="gt_heading gt_title gt_font_normal">content type</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="3" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

Media Type/s allowed in the Payload

</td>
  </tr>

</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">Definition</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">yes</td>
    <td class="gt_row gt_left">Is the field required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"><a href="https://datatracker.ietf.org/doc/html/rfc9052">Cose</a> Label</th>
    <td class="gt_row gt_left gt_striped">3</td>
    <td class="gt_row gt_left gt_striped"><a href="https://datatracker.ietf.org/doc/html/rfc9052">COSE</a> Standard header parameter label.</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Format</th>
    <td class="gt_row gt_left">Media Type</td>
    <td class="gt_row gt_left">A Media Type string which identifies the payload.</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">Supported Values</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left gt_striped"><a href="https://www.iana.org/assignments/media-types/application/cbor">application/cbor</a></td>
    <td class="gt_row gt_left gt_striped">An <a href="https://www.rfc-editor.org/rfc/rfc8949.html">RFC8949</a> Binary <a href="https://www.rfc-editor.org/rfc/rfc8949.html">CBOR</a> Encoded Document.</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left"><a href="https://www.rfc-editor.org/rfc/rfc8610">application/cddl</a></td>
    <td class="gt_row gt_left">A <a href="https://www.rfc-editor.org/rfc/rfc8610">CDDL</a> Document.<br>
Note:</p>
<ul>
<li>This is an unofficial media type</li>
<li><a href="https://www.rfc-editor.org/rfc/rfc9165">RFC9165</a> Additional Control Operators for <a href="https://www.rfc-editor.org/rfc/rfc8610">CDDL</a> are supported.</li>
<li>Must not have Modules, schema must be self-contained.</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left gt_striped"><a href="https://www.iana.org/assignments/media-types/application/json">application/json</a></td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Document</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left"><a href="https://json-schema.org/draft/2020-12">application/schema+json</a></td>
    <td class="gt_row gt_left">A <a href="https://json-schema.org/draft/2020-12"><a href="https://json-schema.org/draft/2020-12"><a href="https://www.rfc-editor.org/rfc/rfc8259.html">JSON</a> Schema</a> Draft 2020-12</a> Document.<br>
Note:</p>
<ul>
<li>This is a draft/unofficial media type.</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc2318.html"><a href="https://www.rfc-editor.org/rfc/rfc2318.html">text/css</a>;</a> <a href="https://datatracker.ietf.org/doc/html/rfc3629">charset=utf-8</a></td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.w3.org/Style/CSS/">CSS</a> Content used for styling <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML</a>.<br>
Note:</p>
<ul>
<li><a href="https://www.w3.org/Style/CSS/">CSS</a> should use the least set of features possible to achieve
the desired presentation to ensure the broadest compatibility.</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left"><a href="https://www.rfc-editor.org/rfc/rfc2318.html"><a href="https://www.rfc-editor.org/rfc/rfc2318.html">text/css</a>;</a> <a href="https://datatracker.ietf.org/doc/html/rfc3629"><a href="https://datatracker.ietf.org/doc/html/rfc3629">charset=utf-8</a>;</a> <a href="https://handlebarsjs.com/">template=handlebars</a></td>
    <td class="gt_row gt_left"><a href="https://www.w3.org/Style/CSS/">CSS</a> Content used for styling <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML</a>.<br>
Note:</p>
<ul>
<li><a href="https://www.w3.org/Style/CSS/">CSS</a> should use the least set of features possible to achieve
the desired presentation to ensure the broadest compatibility.</li>
<li>The text includes <a href="https://handlebarsjs.com/">Handlebars</a> type template fields that need
processing and replacement prior to display.</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left gt_striped"><a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">text/html;</a> <a href="https://datatracker.ietf.org/doc/html/rfc3629">charset=utf-8</a></td>
    <td class="gt_row gt_left gt_striped">Formatted text using <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML5</a> markup for rich text.<br>
Note:</p>
<ul>
<li>Only <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML5</a> syntax is supported.</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left"><a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">text/html;</a> <a href="https://datatracker.ietf.org/doc/html/rfc3629"><a href="https://datatracker.ietf.org/doc/html/rfc3629">charset=utf-8</a>;</a> <a href="https://handlebarsjs.com/">template=handlebars</a></td>
    <td class="gt_row gt_left">Formatted text using <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML5</a> markup for rich text.<br>
Note:</p>
<ul>
<li>Only <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML5</a> syntax is supported.</li>
<li>The text includes <a href="https://handlebarsjs.com/">Handlebars</a> type template fields that need
processing and replacement prior to display.</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left gt_striped"><a href="https://spec.commonmark.org/0.31.2/">text/markdown;</a> <a href="https://datatracker.ietf.org/doc/html/rfc3629">charset=utf-8</a></td>
    <td class="gt_row gt_left gt_striped">Formatted text using <a href="https://spec.commonmark.org/0.31.2/">Markdown</a> for rich text.<br>
Note:</p>
<ul>
<li><a href="https://spec.commonmark.org/0.31.2/">Markdown</a> formatting is as defined by <a href="https://spec.commonmark.org/0.31.2/">CommonMark</a>.</li>
<li>IF the document includes <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML</a>, then <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML5</a> syntax only is supported.</li>
<li>The following <a href="https://spec.commonmark.org/0.31.2/">Markdown</a> Extensions are also supported:
<ul>
<li>None</li>
</ul>
</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left"><a href="https://spec.commonmark.org/0.31.2/">text/markdown;</a> <a href="https://datatracker.ietf.org/doc/html/rfc3629"><a href="https://datatracker.ietf.org/doc/html/rfc3629">charset=utf-8</a>;</a> <a href="https://handlebarsjs.com/">template=handlebars</a></td>
    <td class="gt_row gt_left">Formatted text using <a href="https://spec.commonmark.org/0.31.2/">Markdown</a> for rich text.<br>
Note:</p>
<ul>
<li><a href="https://spec.commonmark.org/0.31.2/">Markdown</a> formatting is as defined by <a href="https://spec.commonmark.org/0.31.2/">CommonMark</a>.</li>
<li>IF the document includes <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML</a>, then <a href="https://html.spec.whatwg.org/multipage/syntax.html#syntax">HTML5</a> syntax only is supported.</li>
<li>The following <a href="https://spec.commonmark.org/0.31.2/">Markdown</a> Extensions are also supported:
<ul>
<li>None</li>
</ul>
</li>
<li>The text includes <a href="https://handlebarsjs.com/">Handlebars</a> type template fields that need
processing and replacement prior to display.</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc2046.html"><a href="https://www.rfc-editor.org/rfc/rfc2046.html">text/plain</a>;</a> <a href="https://datatracker.ietf.org/doc/html/rfc3629">charset=utf-8</a></td>
    <td class="gt_row gt_left gt_striped">Plain Text with no markup or special formatting.<br>
Note:</p>
<ul>
<li>Multiline Plain Text <em>MUST</em> always interpret <code>\n</code>
as a hard line break.</li>
</ul>
</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left"><a href="https://www.rfc-editor.org/rfc/rfc2046.html"><a href="https://www.rfc-editor.org/rfc/rfc2046.html">text/plain</a>;</a> <a href="https://datatracker.ietf.org/doc/html/rfc3629"><a href="https://datatracker.ietf.org/doc/html/rfc3629">charset=utf-8</a>;</a> <a href="https://handlebarsjs.com/">template=handlebars</a></td>
    <td class="gt_row gt_left">Plain Text with no markup or special formatting.<br>
Note:</p>
<ul>
<li>Multiline Plain Text <em>MUST</em> always interpret <code>\n</code>
as a hard line break.</li>
<li>The text includes <a href="https://handlebarsjs.com/">Handlebars</a> type template fields that need
processing and replacement prior to display.</li>
</ul>
</td>
  </tr>
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->
#### `content-encoding`

Supported HTTP Encodings of the Payload.
If no compression or encoding is used, then this field must not be present.

<!---HTML START-->
<!-- markdownlint-disable -->
<div id="spec_content-encoding" style="padding-left:0px;padding-right:0px;padding-top:10px;padding-bottom:10px;overflow-x:auto;overflow-y:auto;width:100%;height:auto;">
<style>
#spec_content-encoding table {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', 'Fira Sans', 'Droid Sans', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }

#spec_content-encoding thead, tbody, tfoot, tr, td, th { border-style: none; }
 tr { background-color: transparent; }
#spec_content-encoding p { margin: 0; padding: 0; }
 #spec_content-encoding .gt_table { display: table; border-collapse: collapse; line-height: normal; margin-left: auto; margin-right: auto; color: #333333; font-size: 16px; font-weight: normal; font-style: normal; background-color: #FFFFFF; width: 100%; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; }
 #spec_content-encoding .gt_caption { padding-top: 4px; padding-bottom: 4px; }
 #spec_content-encoding .gt_title { color: #333333; font-size: 125%; font-weight: initial; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; border-bottom-color: #FFFFFF; border-bottom-width: 0; }
 #spec_content-encoding .gt_subtitle { color: #333333; font-size: 85%; font-weight: initial; padding-top: 3px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; border-top-color: #FFFFFF; border-top-width: 0; }
 #spec_content-encoding .gt_heading { background-color: #FFFFFF; text-align: center; border-bottom-color: #FFFFFF; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #spec_content-encoding .gt_bottom_border { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #spec_content-encoding .gt_col_headings { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #spec_content-encoding .gt_col_heading { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; overflow-x: hidden; }
 #spec_content-encoding .gt_column_spanner_outer { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; padding-top: 0; padding-bottom: 0; padding-left: 4px; padding-right: 4px; }
 #spec_content-encoding .gt_column_spanner_outer:first-child { padding-left: 0; }
 #spec_content-encoding .gt_column_spanner_outer:last-child { padding-right: 0; }
 #spec_content-encoding .gt_column_spanner { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; overflow-x: hidden; display: inline-block; width: 100%; }
 #spec_content-encoding .gt_spanner_row { border-bottom-style: hidden; }
 #spec_content-encoding .gt_group_heading { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: middle; text-align: left; }
 #spec_content-encoding .gt_empty_group_heading { padding: 0.5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: middle; }
 #spec_content-encoding .gt_from_md> :first-child { margin-top: 0; }
 #spec_content-encoding .gt_from_md> :last-child { margin-bottom: 0; }
 #spec_content-encoding .gt_row { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; margin: 10px; border-top-style: none; border-top-width: 1px; border-top-color: #D5D5D5; border-left-style: none; border-left-width: 1px; border-left-color: #D5D5D5; border-right-style: none; border-right-width: 1px; border-right-color: #D5D5D5; vertical-align: middle; overflow-x: hidden; }
 #spec_content-encoding .gt_stub { color: #333333; background-color: #89D3FE; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D5D5D5; padding-left: 5px; padding-right: 5px; }
 #spec_content-encoding .gt_stub_row_group { color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D3D3D3; padding-left: 5px; padding-right: 5px; vertical-align: top; }
 #spec_content-encoding .gt_row_group_first td { border-top-width: 2px; }
 #spec_content-encoding .gt_row_group_first th { border-top-width: 2px; }
 #spec_content-encoding .gt_striped { color: #333333; background-color: #EDF7FC; }
 #spec_content-encoding .gt_table_body { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #spec_content-encoding .gt_grand_summary_row { color: #333333; background-color: #D5D5D5; text-transform: inherit; padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; }
 #spec_content-encoding .gt_first_grand_summary_row_bottom { border-top-style: double; border-top-width: 6px; border-top-color: #D3D3D3; }
 #spec_content-encoding .gt_last_grand_summary_row_top { border-bottom-style: double; border-bottom-width: 6px; border-bottom-color: #D3D3D3; }
 #spec_content-encoding .gt_sourcenotes { color: #333333; background-color: #FFFFFF; border-bottom-style: none; border-bottom-width: 2px; border-bottom-color: #D3D3D3; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; }
 #spec_content-encoding .gt_sourcenote { font-size: 90%; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; text-align: left; }
 #spec_content-encoding .gt_left { text-align: left; }
 #spec_content-encoding .gt_center { text-align: center; }
 #spec_content-encoding .gt_right { text-align: right; font-variant-numeric: tabular-nums; }
 #spec_content-encoding .gt_font_normal { font-weight: normal; }
 #spec_content-encoding .gt_font_bold { font-weight: bold; }
 #spec_content-encoding .gt_font_italic { font-style: italic; }
 #spec_content-encoding .gt_super { font-size: 65%; }
 #spec_content-encoding .gt_footnote_marks { font-size: 75%; vertical-align: 0.4em; position: initial; }
 #spec_content-encoding .gt_asterisk { font-size: 100%; vertical-align: 0; }

</style>
<table class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<thead>

  <tr class="gt_heading">
    <td colspan="3" class="gt_heading gt_title gt_font_normal">content-encoding</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="3" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

Supported HTTP Encodings of the Payload

</td>
  </tr>

</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">Definition</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">optional</td>
    <td class="gt_row gt_left">Is the field required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"><a href="https://datatracker.ietf.org/doc/html/rfc9052">Cose</a> Label</th>
    <td class="gt_row gt_left gt_striped">content-encoding</td>
    <td class="gt_row gt_left gt_striped">Custom Header parameter label.</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Format</th>
    <td class="gt_row gt_left">HTTP Content Encoding</td>
    <td class="gt_row gt_left">Encoding, if any, of the payload.</td>
  </tr>
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">Supported Values</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"></th>
    <td class="gt_row gt_left gt_striped"><a href="https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding#br">br</a></td>
    <td class="gt_row gt_left gt_striped"><a href="https://www.rfc-editor.org/rfc/rfc7932">BROTLI</a> Compression</td>
  </tr>
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->

### Metadata

Catalyst Signed Documents extend the Header Parameters with a series of
[Metadata fields](metadata).

### Signing Catalyst Signed Documents

Catalyst Signed Documents are based on the [COSE Sign][RFC9052-CoseSign] format.
This allows one or more signatures to be attached to the same document.
A catalyst signed document *MUST* have at least one valid signature attached.
Multiple signatures may also be attached to the same document, where that is required.

Each signature is contained in an array of signatures attached to the document.
The signatures contain protected headers, and the signature itself.
The headers currently defined for the signatures are:

#### `kid`

Catalyst ID [URI][RFC3986] identifying the Public Key.

The `kid` is a [UTF-8][RFC3629] encoded Catalyst ID [URI][RFC3986].
Any `kid` [URI][RFC3986] which conforms to the Catalyst ID specification may be used.
The Catalyst ID unambiguously defines both the signing keys and signing algorithm
used to sign the protected portion of the document.

There may be **MULTIPLE** [Cose][RFC9052] Signatures attached to any document.
In the event there are **MULTIPLE** [Cose][RFC9052] Signatures `kid` attached, then they **MUST**
be sorted.

Sorting for each [cose][RFC9052] signature follows the same sort order as specified for Map Keys,
as defined by [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] (4.3.2 Length-First Map Key Ordering).

<!---HTML START-->
<!-- markdownlint-disable -->
<div id="spec_kid" style="padding-left:0px;padding-right:0px;padding-top:10px;padding-bottom:10px;overflow-x:auto;overflow-y:auto;width:100%;height:auto;">
<style>
#spec_kid table {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', 'Fira Sans', 'Droid Sans', Arial, sans-serif;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }

#spec_kid thead, tbody, tfoot, tr, td, th { border-style: none; }
 tr { background-color: transparent; }
#spec_kid p { margin: 0; padding: 0; }
 #spec_kid .gt_table { display: table; border-collapse: collapse; line-height: normal; margin-left: auto; margin-right: auto; color: #333333; font-size: 16px; font-weight: normal; font-style: normal; background-color: #FFFFFF; width: 100%; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; }
 #spec_kid .gt_caption { padding-top: 4px; padding-bottom: 4px; }
 #spec_kid .gt_title { color: #333333; font-size: 125%; font-weight: initial; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; border-bottom-color: #FFFFFF; border-bottom-width: 0; }
 #spec_kid .gt_subtitle { color: #333333; font-size: 85%; font-weight: initial; padding-top: 3px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; border-top-color: #FFFFFF; border-top-width: 0; }
 #spec_kid .gt_heading { background-color: #FFFFFF; text-align: center; border-bottom-color: #FFFFFF; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #spec_kid .gt_bottom_border { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #spec_kid .gt_col_headings { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; }
 #spec_kid .gt_col_heading { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; padding-left: 5px; padding-right: 5px; overflow-x: hidden; }
 #spec_kid .gt_column_spanner_outer { color: #FFFFFF; background-color: #0076BA; font-size: 100%; font-weight: normal; text-transform: inherit; padding-top: 0; padding-bottom: 0; padding-left: 4px; padding-right: 4px; }
 #spec_kid .gt_column_spanner_outer:first-child { padding-left: 0; }
 #spec_kid .gt_column_spanner_outer:last-child { padding-right: 0; }
 #spec_kid .gt_column_spanner { border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: bottom; padding-top: 5px; padding-bottom: 5px; overflow-x: hidden; display: inline-block; width: 100%; }
 #spec_kid .gt_spanner_row { border-bottom-style: hidden; }
 #spec_kid .gt_group_heading { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; border-left-style: none; border-left-width: 1px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 1px; border-right-color: #D3D3D3; vertical-align: middle; text-align: left; }
 #spec_kid .gt_empty_group_heading { padding: 0.5px; color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; vertical-align: middle; }
 #spec_kid .gt_from_md> :first-child { margin-top: 0; }
 #spec_kid .gt_from_md> :last-child { margin-bottom: 0; }
 #spec_kid .gt_row { padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; margin: 10px; border-top-style: none; border-top-width: 1px; border-top-color: #D5D5D5; border-left-style: none; border-left-width: 1px; border-left-color: #D5D5D5; border-right-style: none; border-right-width: 1px; border-right-color: #D5D5D5; vertical-align: middle; overflow-x: hidden; }
 #spec_kid .gt_stub { color: #333333; background-color: #89D3FE; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D5D5D5; padding-left: 5px; padding-right: 5px; }
 #spec_kid .gt_stub_row_group { color: #333333; background-color: #FFFFFF; font-size: 100%; font-weight: initial; text-transform: inherit; border-right-style: solid; border-right-width: 2px; border-right-color: #D3D3D3; padding-left: 5px; padding-right: 5px; vertical-align: top; }
 #spec_kid .gt_row_group_first td { border-top-width: 2px; }
 #spec_kid .gt_row_group_first th { border-top-width: 2px; }
 #spec_kid .gt_striped { color: #333333; background-color: #EDF7FC; }
 #spec_kid .gt_table_body { border-top-style: solid; border-top-width: 2px; border-top-color: #5F5F5F; border-bottom-style: solid; border-bottom-width: 2px; border-bottom-color: #5F5F5F; }
 #spec_kid .gt_grand_summary_row { color: #333333; background-color: #D5D5D5; text-transform: inherit; padding-top: 8px; padding-bottom: 8px; padding-left: 5px; padding-right: 5px; }
 #spec_kid .gt_first_grand_summary_row_bottom { border-top-style: double; border-top-width: 6px; border-top-color: #D3D3D3; }
 #spec_kid .gt_last_grand_summary_row_top { border-bottom-style: double; border-bottom-width: 6px; border-bottom-color: #D3D3D3; }
 #spec_kid .gt_sourcenotes { color: #333333; background-color: #FFFFFF; border-bottom-style: none; border-bottom-width: 2px; border-bottom-color: #D3D3D3; border-left-style: none; border-left-width: 2px; border-left-color: #D3D3D3; border-right-style: none; border-right-width: 2px; border-right-color: #D3D3D3; }
 #spec_kid .gt_sourcenote { font-size: 90%; padding-top: 4px; padding-bottom: 4px; padding-left: 5px; padding-right: 5px; text-align: left; }
 #spec_kid .gt_left { text-align: left; }
 #spec_kid .gt_center { text-align: center; }
 #spec_kid .gt_right { text-align: right; font-variant-numeric: tabular-nums; }
 #spec_kid .gt_font_normal { font-weight: normal; }
 #spec_kid .gt_font_bold { font-weight: bold; }
 #spec_kid .gt_font_italic { font-style: italic; }
 #spec_kid .gt_super { font-size: 65%; }
 #spec_kid .gt_footnote_marks { font-size: 75%; vertical-align: 0.4em; position: initial; }
 #spec_kid .gt_asterisk { font-size: 100%; vertical-align: 0; }

</style>
<table class="gt_table" data-quarto-disable-processing="false" data-quarto-bootstrap="false">
<thead>

  <tr class="gt_heading">
    <td colspan="3" class="gt_heading gt_title gt_font_normal">kid</td>
  </tr>
  <tr class="gt_heading">
    <td colspan="3" class="gt_heading gt_subtitle gt_font_normal gt_bottom_border">

Catalyst ID <a href="https://datatracker.ietf.org/doc/html/rfc3986">URI</a> identifying the Public Key

</td>
  </tr>

</thead>
<tbody class="gt_table_body">
  <tr class="gt_group_heading_row">
    <th class="gt_group_heading" colspan="3">Definition</th>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Required</th>
    <td class="gt_row gt_left">yes</td>
    <td class="gt_row gt_left">Is the field required?</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub"><a href="https://datatracker.ietf.org/doc/html/rfc9052">Cose</a> Label</th>
    <td class="gt_row gt_left gt_striped">4</td>
    <td class="gt_row gt_left gt_striped"><a href="https://datatracker.ietf.org/doc/html/rfc9052">COSE</a> Standard header parameter label.</td>
  </tr>
  <tr>
    <th class="gt_row gt_left gt_stub">Format</th>
    <td class="gt_row gt_left">Catalyst ID</td>
    <td class="gt_row gt_left">KID (Catalyst ID URI)</td>
  </tr>
</tbody>


</table>

</div>


<!-- markdownlint-enable -->
<!---HTML END-->

## Copyright

| Copyright | :copyright: 2024-2026 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2026-01-13 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.0.1 (2025-04-04)

* First Published Version

#### 0.0.2 (2025-04-09)

* Add version control changelogs to the specification.

#### 0.0.3 (2025-05-05)

* Use generalized parameters.

#### 0.0.4 (2025-06-25)

* Improve and make document serialization more repeatable, and stricter.
* Defined Systems parameters documents
* Defined DReps documents.

#### 0.1.0 (2025-07-30)

* Fixed typographical and layout issues.
* Added specifications for Form Templates and Elements.
* Reverted Document Types to a Single [UUID][RFC9562] from an Array of UUIDs
* Changed versions to be semantic (0.04 became 0.0.4)
* Corrected Parameter References for Brand/Campaign/Category/Contest Templates
* Replaced poorly formatting [markdown][CommonMark] tables with [HTML][HTML5] equivalents.
* Added draft placeholders for Moderation Documents (subject to change)
* Clarified How Representatives may delegate.
* Clarified what happens when a new version of a Nomination Document is published.
* Clarified how delegations can be revoked.
* Clarified the payload for delegations.
* Generalized Presentation Templates, and defined sample (subject to change) cards.
* Removed specialized presentation templates, as a universal presentation template is all thats required.
* Converted draft-7 [Json][RFC8259] Schemas to 2020-12
* Add standard ICON definitions for Forms.

#### 0.1.1 (2025-08-19)

* Define an Optional Section in a Form Template, to enable partial form submission while in Draft.

#### 0.1.2 (2025-09-08)

* Updated `payload` field, it become required.
* Added new `draft` field for Signed Document with the default value `false`.
* Made `payload.nil` non optional with the default value `false`.
* If `payload.nil` is `true` automatically set `"content type"` and `"content-encoding"` fields to `"excluded"`.

#### 0.1.3 (2025-09-09)

* Fixed an invalid 'Presentation Template' [JSON schema][JSON Schema-2020-12].

#### 0.1.4 (2025-10-17)

* Modified [`collaborators`](metadata.md#collaborators) [cddl][RFC8610] definition, it must have at least one element in array.

#### 0.1.5 (2025-10-24)

* Updated 'Proposal Submission Action' document, set [`ref`](metadata.md#ref) metadata field `multiply` property to `false`.
* Changed spec `signers.update` property structure.

#### 0.2.0 (2025-11-10)

* Added a new 'Contest Ballot' and 'Contest Ballot Checkpoint' document types.
* Improved the specification for 'Contest Delegation' document type.
* 'content encoding' metadata field become non optional for all document types where it was an optional field.
* Added new 'payload.schema' type - [CDDL][RFC8610] schema, defined as string.

#### 0.2.1 (2025-12-02)

* Added missing [`ref`](metadata.md#ref) metadata field definition.
* Improved `payload` [cddl][RFC8610] definition, replaced `document_ref` to the `uint` as a map keys to the `choices`.

#### 0.2.2 (2025-12-15)

* Added missing `signers: update: type: "ref"` definition for [Rep Nomination](./docs/rep_nomination.md) document type.

#### 0.2.3 (2026-01-09)

* Internal dependency updates.

#### 0.2.4 (2026-01-13)

* Internal dependency updates.

#### 0.2.5 (2026-01-15)

* `catalyst-signed-doc-spec` payload `Schema::Json` type.

[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[JSON Schema-2020-12]: https://json-schema.org/draft/2020-12
[RFC9052-CoseSign]: https://datatracker.ietf.org/doc/html/rfc9052#name-signing-with-one-or-more-si
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[CommonMark]: https://spec.commonmark.org/0.31.2/
[RFC3629]: https://datatracker.ietf.org/doc/html/rfc3629
[RFC8610]: https://www.rfc-editor.org/rfc/rfc8610
[RFC9052]: https://datatracker.ietf.org/doc/html/rfc9052
[HTML5]: https://html.spec.whatwg.org/multipage/syntax.html#syntax
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
[RFC9562]: https://www.rfc-editor.org/rfc/rfc9562.html
[RFC3986]: https://datatracker.ietf.org/doc/html/rfc3986
