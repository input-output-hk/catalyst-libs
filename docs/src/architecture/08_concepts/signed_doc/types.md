# Document Types Table

## Document Base Types

All Document Types are defined by composing these base document types:

| Base Type | [UUID][RFC9562] | [CBOR][RFC8949] |
| :--- | :--- | :--- |
| Action | `5e60e623-ad02-4a1b-a1ac-406db978ee48` | `37(h'5e60e623ad024a1ba1ac406db978ee48')` |
| Brand | `ebcabeeb-5bc5-4f95-91e8-cab8ca724172` | `37(h'ebcabeeb5bc54f9591e8cab8ca724172')` |
| Campaign | `5ef32d5d-f240-462c-a7a4-ba4af221fa23` | `37(h'5ef32d5df240462ca7a4ba4af221fa23')` |
| Category | `818938c3-3139-4daa-afe6-974c78488e95` | `37(h'818938c331394daaafe6974c78488e95')` |
| Comment | `b679ded3-0e7c-41ba-89f8-da62a17898ea` | `37(h'b679ded30e7c41ba89f8da62a17898ea')` |
| Election | `788ff4c6-d65a-451f-bb33-575fe056b411` | `37(h'788ff4c6d65a451fbb33575fe056b411')` |
| ModerationAction | `a5d232b8-5e03-4117-9afd-be32b878fcdd` | `37(h'a5d232b85e0341179afdbe32b878fcdd')` |
| Proposal | `7808d2ba-d511-40af-84e8-c0d1625fdfdc` | `37(h'7808d2bad51140af84e8c0d1625fdfdc')` |
| SubmissionAction | `78927329-cfd9-4ea1-9c71-0e019b126a65` | `37(h'78927329cfd94ea19c710e019b126a65')` |
| Template | `0ce8ab38-9258-4fbc-a62e-7faa6e58318f` | `37(h'0ce8ab3892584fbca62e7faa6e58318f')` |

## Document Types

All Defined Document Types

<!-- markdownlint-disable MD033 -->
| Document Type | Base Types | [CBOR][RFC8949] |
| :--- | :--- | :--- |
| [Brand Parameters](./docs/brand_parameters.md) | Brand | [37(h'ebcabeeb5bc54f9591e8cab8ca724172')] |
| [Campaign Parameters](./docs/campaign_parameters.md) | Campaign | [37(h'5ef32d5df240462ca7a4ba4af221fa23')] |
| [Category Parameters](./docs/category_parameters.md) | Category | [37(h'818938c331394daaafe6974c78488e95')] |
| [Comment Action Document](./docs/comment_action_document.md) | Action/Comment/ModerationAction | [37(h'5e60e623ad024a1ba1ac406db978ee48'),<br/>37(h'b679ded30e7c41ba89f8da62a17898ea'),<br/>37(h'a5d232b85e0341179afdbe32b878fcdd')] |
| [Election Parameters](./docs/election_parameters.md) | Election | [37(h'788ff4c6d65a451fbb33575fe056b411')] |
| [Proposal](./docs/proposal.md) | Proposal | [37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Comment](./docs/proposal_comment.md) | Comment/Proposal | [37(h'b679ded30e7c41ba89f8da62a17898ea'),<br/>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Comment Meta Template](./docs/proposal_comment_meta_template.md) | Template/Template/Comment/Proposal | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br/>37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br/>37(h'b679ded30e7c41ba89f8da62a17898ea'),<br/>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Comment Template](./docs/proposal_comment_template.md) | Template/Comment/Proposal | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br/>37(h'b679ded30e7c41ba89f8da62a17898ea'),<br/>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Meta Template](./docs/proposal_meta_template.md) | Template/Template/Proposal | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br/>37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br/>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Moderation Action](./docs/proposal_moderation_action.md) | Action/Proposal/ModerationAction | [37(h'5e60e623ad024a1ba1ac406db978ee48'),<br/>37(h'7808d2bad51140af84e8c0d1625fdfdc'),<br/>37(h'a5d232b85e0341179afdbe32b878fcdd')] |
| [Proposal Submission Action](./docs/proposal_submission_action.md) | Action/Proposal/SubmissionAction | [37(h'5e60e623ad024a1ba1ac406db978ee48'),<br/>37(h'7808d2bad51140af84e8c0d1625fdfdc'),<br/>37(h'78927329cfd94ea19c710e019b126a65')] |
| [Proposal Template](./docs/proposal_template.md) | Template/Proposal | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br/>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
<!-- markdownlint-enable MD033 -->

## Document Relationship Hierarchy

![Document Relationship Hierarchy](doc_relationships.svg)

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-04-03 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC9562]: https://www.rfc-editor.org/rfc/rfc9562.html
