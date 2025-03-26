---
Title: Catalyst Signed Documents - Validation Summary
Category: Catalyst
Status: Proposed
Authors:
    - Steven Johnson <steven.johnson@iohk.io>
Implementors:
    - Catalyst Fund 14
Discussions: []
Created: 2024-12-29
License: CC-BY-4.0
---

## Validation

This is a list of Metadata fields that apply to each document type, and how they are to be validated.

| Document Type |`ref`  | `ref_hash` | `template` | `reply` | `section` | `brand_id` | `campaign_id` | `category_id` |
| ------------- | ----- | ---------- | --------- | ------- | --------- | ---------- | ------------ | ----------------- |
| Proposal      | None  | `p`        | `p`       | `p`     | `p`       | `p`        | `p`          | `p`               |
| Comment       | `c`    | `p`   | `p`        | `p`       | `p`     | `p`       | `p`        | `p`          |
