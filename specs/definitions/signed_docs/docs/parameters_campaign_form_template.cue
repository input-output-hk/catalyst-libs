// Campaign Parameters Form Template Document Definition
package signed_docs

import (
	"text/template"
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

docs: "Campaign Parameters Form Template": #generic_form_template & {
	_data: doc: "Campaign Parameters"

	description: template.Execute(_form_template_description, _data)
	metadata: parameters: _metadataFieldBrandParameters
	payload: {
		description: template.Execute(_form_template_payload_description, _data)
		schema: {
			$schema:              "https://json-schema.org/draft/2020-12/schema"
			title:                "Campaign Parameters Form Template"
			description:          "JSON Schema for Campaign Parameters using CUE form elements"
			type:                 "object"
			additionalProperties: false
			properties: {
				campaignName: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Campaign Name"
					description: "The name of the campaign"
					maxLength:   100
					required:    true
				}
				campaignDescription: {
					$ref:        "#/$defs/multiLineTextEntry"
					title:       "Campaign Description"
					description: "Description of the campaign"
					maxLength:   2000
					required:    true
				}
				campaignTheme: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Campaign Theme"
					description: "Campaign theme or focus area"
					maxLength:   200
					required:    true
				}
				bannerImage: {
					$ref:        "#/$defs/singleLineHttpsUrlEntry"
					title:       "Campaign Banner Image URL"
					description: "URL to campaign banner image"
					required:    false
				}
				startDate: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Campaign Start Date"
					description: "Campaign start date (YYYY-MM-DD format)"
					pattern:     "^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
					required:    true
				}
				endDate: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Campaign End Date"
					description: "Campaign end date (YYYY-MM-DD format)"
					pattern:     "^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
					required:    true
				}
				proposalSubmissionDeadline: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Proposal Submission Deadline"
					description: "Deadline for proposal submissions (YYYY-MM-DD format)"
					pattern:     "^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
					required:    true
				}
				votingStartDate: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Voting Start Date"
					description: "When voting begins (YYYY-MM-DD format)"
					pattern:     "^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
					required:    true
				}
				votingEndDate: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Voting End Date"
					description: "When voting ends (YYYY-MM-DD format)"
					pattern:     "^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
					required:    true
				}
				totalBudget: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Total Budget"
					description: "Total budget available for this campaign"
					pattern:     "^[0-9]+$"
					required:    true
				}
				maxProposalAmount: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Maximum Proposal Amount"
					description: "Maximum amount for a single proposal in this campaign"
					pattern:     "^[0-9]+$"
					required:    true
				}
				minProposalAmount: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Minimum Proposal Amount"
					description: "Minimum amount for a single proposal in this campaign"
					pattern:     "^[0-9]+$"
					required:    true
				}
				disbursementMethod: {
					$ref:        "#/$defs/singleSelect"
					title:       "Disbursement Method"
					description: "Method for disbursing funds"
					enum: ["immediate", "milestone-based", "project-completion"]
					required: true
				}
				votingMechanism: {
					$ref:        "#/$defs/singleSelect"
					title:       "Voting Mechanism"
					description: "Voting mechanism used in this campaign"
					enum: ["simple-majority", "ranked-choice", "approval-voting"]
					required: true
				}
				minStakeAmount: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Minimum Stake Amount"
					description: "Minimum stake amount required to vote"
					pattern:     "^[0-9]+$"
					required:    true
				}
				minHoldingPeriod: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Minimum Holding Period"
					description: "Minimum holding period in days"
					pattern:     "^[0-9]+$"
					required:    true
				}
				excludeExchanges: {
					$ref:        "#/$defs/singleSelect"
					title:       "Exclude Exchanges"
					description: "Whether to exclude exchange addresses"
					enum: ["true", "false"]
					required: true
				}
				maxProposalsPerUser: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Maximum Proposals Per User"
					description: "Maximum number of proposals a user can submit"
					pattern:     "^[0-9]+$"
					required:    true
				}
				requireKYC: {
					$ref:        "#/$defs/singleSelect"
					title:       "Require KYC"
					description: "Whether KYC is required for participation"
					enum: ["true", "false"]
					required: true
				}
				proposalTemplate: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Proposal Template ID"
					description: "Template ID for proposals in this campaign"
					pattern:     "^[a-zA-Z0-9\\-_]+$"
					required:    false
				}
				reviewStages: {
					$ref:        "#/$defs/multiSelect"
					title:       "Review Stages"
					description: "Stages in the review process"
					contains: ["initial-screening", "community-review", "expert-evaluation", "final-approval"]
					required: false
				}
				reviewerCount: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Reviewer Count"
					description: "Number of reviewers per proposal"
					pattern:     "^[0-9]+$"
					required:    false
				}
				reminderFrequency: {
					$ref:        "#/$defs/singleSelect"
					title:       "Reminder Frequency"
					description: "Frequency of voting reminders"
					enum: ["daily", "weekly", "bi-weekly"]
					required: false
				}
				milestoneAlerts: {
					$ref:        "#/$defs/singleSelect"
					title:       "Milestone Alerts"
					description: "Whether to send milestone alerts"
					enum: ["true", "false"]
					required: false
				}
			}
			required: [
				"campaignName",
				"campaignDescription",
				"campaignTheme",
				"startDate",
				"endDate",
				"proposalSubmissionDeadline",
				"votingStartDate",
				"votingEndDate",
				"totalBudget",
				"maxProposalAmount",
				"minProposalAmount",
				"disbursementMethod",
				"votingMechanism",
				"minStakeAmount",
				"minHoldingPeriod",
				"excludeExchanges",
				"maxProposalsPerUser",
				"requireKYC",
			]
		}
	}
	versions: _generic_form_template_versions
}
