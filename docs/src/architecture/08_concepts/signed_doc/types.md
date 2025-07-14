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
| Contest | `788ff4c6-d65a-451f-bb33-575fe056b411` | `37(h'788ff4c6d65a451fbb33575fe056b411')` |
| Delegation | `764f17fb-cc50-4979-b14a-b213dbac5994` | `37(h'764f17fbcc504979b14ab213dbac5994')` |
| FormTemplate | `0ce8ab38-9258-4fbc-a62e-7faa6e58318f` | `37(h'0ce8ab3892584fbca62e7faa6e58318f')` |
| ModerationAction | `a5d232b8-5e03-4117-9afd-be32b878fcdd` | `37(h'a5d232b85e0341179afdbe32b878fcdd')` |
| Nomination | `bf9abd97-5d1f-4429-8e80-740fea371a9c` | `37(h'bf9abd975d1f44298e80740fea371a9c')` |
| Parameters | `60185874-7e13-407c-a06c-238ffe637ae6` | `37(h'601858747e13407ca06c238ffe637ae6')` |
| PresentationTemplate | `cb99b9bd-681a-49d8-9836-89107c02e8ef` | `37(h'cb99b9bd681a49d8983689107c02e8ef')` |
| Profile | `0f2c86a2-ffda-40b0-ad38-23709e1c10b3` | `37(h'0f2c86a2ffda40b0ad3823709e1c10b3')` |
| Proposal | `7808d2ba-d511-40af-84e8-c0d1625fdfdc` | `37(h'7808d2bad51140af84e8c0d1625fdfdc')` |
| RegisteredProposer | `7311c63b-95c6-402e-a258-f9bf622093eb` | `37(h'7311c63b95c6402ea258f9bf622093eb')` |
| RegisteredRep | `94579df1-a6dc-433b-a8e8-910c5dc2f0e3` | `37(h'94579df1a6dc433ba8e8910c5dc2f0e3')` |
| RegisteredUser | `ff4b7724-3db5-44cd-a433-78ba6d29505e` | `37(h'ff4b77243db544cda43378ba6d29505e')` |
| SubmissionAction | `78927329-cfd9-4ea1-9c71-0e019b126a65` | `37(h'78927329cfd94ea19c710e019b126a65')` |

## Document Types

All Defined Document Types

<!-- markdownlint-disable MD033 -->
| Document Type | Base Types | [CBOR][RFC8949] |
| :--- | :--- | :--- |
| [Brand Parameters](docs/brand_parameters.md) | Parameters/Brand | [37(h'601858747e13407ca06c238ffe637ae6'),<br>37(h'ebcabeeb5bc54f9591e8cab8ca724172')] |
| [Brand Parameters Form Template](docs/brand_parameters_form_template.md) | FormTemplate/Parameters/Brand | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br>37(h'601858747e13407ca06c238ffe637ae6'),<br>37(h'ebcabeeb5bc54f9591e8cab8ca724172')] |
| [Campaign Parameters](docs/campaign_parameters.md) | Parameters/Campaign | [37(h'601858747e13407ca06c238ffe637ae6'),<br>37(h'5ef32d5df240462ca7a4ba4af221fa23')] |
| [Campaign Parameters Form Template](docs/campaign_parameters_form_template.md) | FormTemplate/Parameters/Campaign | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br>37(h'601858747e13407ca06c238ffe637ae6'),<br>37(h'5ef32d5df240462ca7a4ba4af221fa23')] |
| [Category Parameters](docs/category_parameters.md) | Parameters/Category | [37(h'601858747e13407ca06c238ffe637ae6'),<br>37(h'818938c331394daaafe6974c78488e95')] |
| [Category Parameters Form Template](docs/category_parameters_form_template.md) | FormTemplate/Parameters/Category | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br>37(h'601858747e13407ca06c238ffe637ae6'),<br>37(h'818938c331394daaafe6974c78488e95')] |
| [Comment Moderation Action](docs/comment_moderation_action.md) | Action/Comment/ModerationAction | [37(h'5e60e623ad024a1ba1ac406db978ee48'),<br>37(h'b679ded30e7c41ba89f8da62a17898ea'),<br>37(h'a5d232b85e0341179afdbe32b878fcdd')] |
| [Contest Delegation](docs/contest_delegation.md) | Delegation/Contest | [37(h'764f17fbcc504979b14ab213dbac5994'),<br>37(h'788ff4c6d65a451fbb33575fe056b411')] |
| [Contest Parameters](docs/contest_parameters.md) | Parameters/Contest | [37(h'601858747e13407ca06c238ffe637ae6'),<br>37(h'788ff4c6d65a451fbb33575fe056b411')] |
| [Contest Parameters Form Template](docs/contest_parameters_form_template.md) | FormTemplate/Parameters/Contest | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br>37(h'601858747e13407ca06c238ffe637ae6'),<br>37(h'788ff4c6d65a451fbb33575fe056b411')] |
| [Proposal](docs/proposal.md) | Proposal | [37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Comment](docs/proposal_comment.md) | Comment/Proposal | [37(h'b679ded30e7c41ba89f8da62a17898ea'),<br>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Comment Form Template](docs/proposal_comment_form_template.md) | FormTemplate/Comment/Proposal | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br>37(h'b679ded30e7c41ba89f8da62a17898ea'),<br>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Comment Presentation Template](docs/proposal_comment_presentation_template.md) | PresentationTemplate/Comment/Proposal | [37(h'cb99b9bd681a49d8983689107c02e8ef'),<br>37(h'b679ded30e7c41ba89f8da62a17898ea'),<br>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Form Template](docs/proposal_form_template.md) | FormTemplate/Proposal | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Moderation Action](docs/proposal_moderation_action.md) | Action/Proposal/ModerationAction | [37(h'5e60e623ad024a1ba1ac406db978ee48'),<br>37(h'7808d2bad51140af84e8c0d1625fdfdc'),<br>37(h'a5d232b85e0341179afdbe32b878fcdd')] |
| [Proposal Presentation Template](docs/proposal_presentation_template.md) | PresentationTemplate/Proposal | [37(h'cb99b9bd681a49d8983689107c02e8ef'),<br>37(h'7808d2bad51140af84e8c0d1625fdfdc')] |
| [Proposal Submission Action](docs/proposal_submission_action.md) | Action/Proposal/SubmissionAction | [37(h'5e60e623ad024a1ba1ac406db978ee48'),<br>37(h'7808d2bad51140af84e8c0d1625fdfdc'),<br>37(h'78927329cfd94ea19c710e019b126a65')] |
| [Rep Nomination](docs/rep_nomination.md) | Nomination/RegisteredRep | [37(h'bf9abd975d1f44298e80740fea371a9c'),<br>37(h'94579df1a6dc433ba8e8910c5dc2f0e3')] |
| [Rep Nomination Form Template](docs/rep_nomination_form_template.md) | FormTemplate/Nomination/RegisteredRep | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br>37(h'bf9abd975d1f44298e80740fea371a9c'),<br>37(h'94579df1a6dc433ba8e8910c5dc2f0e3')] |
| [Rep Profile](docs/rep_profile.md) | Profile/RegisteredRep | [37(h'0f2c86a2ffda40b0ad3823709e1c10b3'),<br>37(h'94579df1a6dc433ba8e8910c5dc2f0e3')] |
| [Rep Profile Form Template](docs/rep_profile_form_template.md) | FormTemplate/Profile/RegisteredRep | [37(h'0ce8ab3892584fbca62e7faa6e58318f'),<br>37(h'0f2c86a2ffda40b0ad3823709e1c10b3'),<br>37(h'94579df1a6dc433ba8e8910c5dc2f0e3')] |
<!-- markdownlint-enable MD033 -->

## Document Relationship Hierarchy

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot all.dot.png
{{ include_file('./diagrams/all.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-05-30 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC9562]: https://www.rfc-editor.org/rfc/rfc9562.html
