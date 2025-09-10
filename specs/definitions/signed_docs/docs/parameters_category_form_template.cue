// Category Parameters Form Template Document Definition
package signed_docs

import (
	"text/template"
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

docs: "Category Parameters Form Template": #generic_form_template & {
	_data: doc: "Category Parameters"

	description: template.Execute(_form_template_description, _data)
	metadata: parameters: _metadataFieldCampaignParameters
	payload: {
		description: template.Execute(_form_template_payload_description, _data)
		schema: {
			$schema:              "https://json-schema.org/draft/2020-12/schema"
			title:                "Category Parameters Form Template"
			description:          "JSON Schema for Category Parameters using CUE form elements"
			type:                 "object"
			additionalProperties: false
			properties: {
				categoryId: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Category ID"
					description: "Unique category identifier"
					pattern:     "^[a-zA-Z0-9\\-_]+$"
					required:    true
				}
				categoryName: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Category Name"
					description: "Category name"
					maxLength:   100
					required:    true
				}
				categoryDescription: {
					$ref:        "#/$defs/multiLineTextEntry"
					title:       "Category Description"
					description: "Category description"
					maxLength:   1000
					required:    true
				}
				maxBudget: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Maximum Budget"
					description: "Maximum budget allocated to this category"
					pattern:     "^[0-9]+$"
					required:    true
				}
				minProposalAmount: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Minimum Proposal Amount"
					description: "Minimum proposal amount for this category"
					pattern:     "^[0-9]+$"
					required:    true
				}
				maxProposalAmount: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Maximum Proposal Amount"
					description: "Maximum proposal amount for this category"
					pattern:     "^[0-9]+$"
					required:    true
				}
				evaluationCriteria: {
					$ref:        "#/$defs/multiLineTextEntryList"
					title:       "Evaluation Criteria"
					description: "Criteria for evaluating proposals in this category"
					maxItems:    10
					required:    false
				}
				categoryIcon: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Category Icon"
					description: "Icon name for this category"
					maxLength:   50
					required:    false
				}
				categoryColor: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Category Color"
					description: "Category color in hex format"
					pattern:     "^#[0-9A-Fa-f]{6}$"
					maxLength:   7
					required:    false
				}
				proposalTemplate: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Proposal Template ID"
					description: "Template ID for proposals in this category"
					pattern:     "^[a-zA-Z0-9\\-_]+$"
					required:    false
				}
				reviewProcess: {
					$ref:        "#/$defs/singleSelect"
					title:       "Review Process"
					description: "Review process for this category"
					enum: ["community-only", "expert-only", "hybrid", "automated"]
					required: false
				}
				approvalThreshold: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Approval Threshold"
					description: "Minimum approval percentage required"
					pattern:     "^[0-9]+$"
					required:    false
				}
				maxProposalsPerCategory: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Maximum Proposals Per Category"
					description: "Maximum number of proposals allowed in this category"
					pattern:     "^[0-9]+$"
					required:    false
				}
				categoryWeight: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Category Weight"
					description: "Weight of this category in overall scoring"
					pattern:     "^[0-9]+$"
					required:    false
				}
				requiresApproval: {
					$ref:        "#/$defs/singleSelect"
					title:       "Requires Approval"
					description: "Whether proposals in this category require approval"
					enum: ["true", "false"]
					required: false
				}
				allowMultipleSubmissions: {
					$ref:        "#/$defs/singleSelect"
					title:       "Allow Multiple Submissions"
					description: "Whether users can submit multiple proposals in this category"
					enum: ["true", "false"]
					required: false
				}
				categoryTags: {
					$ref:        "#/$defs/multiSelect"
					title:       "Category Tags"
					description: "Tags associated with this category"
					contains: ["technology", "community", "education", "infrastructure", "governance", "sustainability"]
					required: false
				}
				visibility: {
					$ref:        "#/$defs/singleSelect"
					title:       "Category Visibility"
					description: "Visibility level of this category"
					enum: ["public", "private", "restricted"]
					required: false
				}
				parentCategoryId: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Parent Category ID"
					description: "ID of parent category if this is a subcategory"
					pattern:     "^[a-zA-Z0-9\\-_]+$"
					required:    false
				}
				sortOrder: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Sort Order"
					description: "Display order of this category"
					pattern:     "^[0-9]+$"
					required:    false
				}
			}
			required: [
				"categoryId",
				"categoryName",
				"categoryDescription",
				"maxBudget",
				"minProposalAmount",
				"maxProposalAmount",
			]
		}
	}
	versions: _generic_form_template_versions
}
