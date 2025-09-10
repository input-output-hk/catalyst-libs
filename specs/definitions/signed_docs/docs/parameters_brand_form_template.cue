// Brand Parameters Form Template Document Definition
package signed_docs

import (
	"text/template"
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

docs: "Brand Parameters Form Template": #generic_form_template & {
	_data: doc: "Brand Parameters"
	description: template.Execute(_form_template_description, _data)
	payload: {
		description: template.Execute(_form_template_payload_description, _data)
		schema: {
			$schema:              "https://json-schema.org/draft/2020-12/schema"
			title:                "Brand Parameters Form Template"
			description:          "JSON Schema for Brand Parameters using CUE form elements"
			type:                 "object"
			additionalProperties: false
			properties: {
				brandName: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Brand Name"
					description: "The name of the brand"
					maxLength:   100
					required:    true
				}
				brandDescription: {
					$ref:        "#/$defs/multiLineTextEntry"
					title:       "Brand Description"
					description: "Description of the brand"
					maxLength:   1000
					required:    true
				}
				brandLogo: {
					$ref:        "#/$defs/singleLineHttpsUrlEntry"
					title:       "Brand Logo URL"
					description: "URL to the brand logo image"
					required:    false
				}
				primaryColor: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Primary Brand Color"
					description: "Primary brand color in hex format"
					pattern:     "^#[0-9A-Fa-f]{6}$"
					maxLength:   7
					required:    false
				}
				secondaryColor: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Secondary Brand Color"
					description: "Secondary brand color in hex format"
					pattern:     "^#[0-9A-Fa-f]{6}$"
					maxLength:   7
					required:    false
				}
				maxProposalAmount: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Maximum Proposal Amount"
					description: "Maximum amount for a single proposal"
					pattern:     "^[0-9]+$"
					required:    true
				}
				minProposalAmount: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Minimum Proposal Amount"
					description: "Minimum amount for a single proposal"
					pattern:     "^[0-9]+$"
					required:    true
				}
				totalBudget: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Total Budget"
					description: "Total budget available for the brand"
					pattern:     "^[0-9]+$"
					required:    true
				}
				votingPowerCalculation: {
					$ref:        "#/$defs/singleSelect"
					title:       "Voting Power Calculation Method"
					description: "Method for calculating voting power"
					enum: ["linear", "quadratic", "logarithmic"]
					required: true
				}
				minVotingPower: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Minimum Voting Power"
					description: "Minimum voting power required to participate"
					pattern:     "^[0-9]+$"
					required:    true
				}
				defaultCampaignDuration: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Default Campaign Duration"
					description: "Default campaign duration in days"
					pattern:     "^[0-9]+$"
					required:    true
				}
				proposalSubmissionDeadline: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "Proposal Submission Deadline"
					description: "Days before campaign end when proposal submission closes"
					pattern:     "^[0-9]+$"
					required:    true
				}
				termsOfService: {
					$ref:        "#/$defs/multiLineTextEntry"
					title:       "Terms of Service"
					description: "Terms of service text"
					maxLength:   10000
					required:    false
				}
				privacyPolicy: {
					$ref:        "#/$defs/multiLineTextEntry"
					title:       "Privacy Policy"
					description: "Privacy policy text"
					maxLength:   10000
					required:    false
				}
				welcomeMessage: {
					$ref:        "#/$defs/multiLineTextEntry"
					title:       "Welcome Message"
					description: "Welcome message for users"
					maxLength:   1000
					required:    false
				}
				proposalSubmissionSuccessMessage: {
					$ref:        "#/$defs/multiLineTextEntry"
					title:       "Proposal Submission Success Message"
					description: "Message shown when proposal is successfully submitted"
					maxLength:   500
					required:    false
				}
				votingReminderMessage: {
					$ref:        "#/$defs/multiLineTextEntry"
					title:       "Voting Reminder Message"
					description: "Message sent as voting reminder"
					maxLength:   500
					required:    false
				}
				apiBaseUrl: {
					$ref:        "#/$defs/singleLineHttpsUrlEntry"
					title:       "API Base URL"
					description: "Base URL for API endpoints"
					required:    true
				}
				apiTimeout: {
					$ref:        "#/$defs/singleLineTextEntry"
					title:       "API Timeout"
					description: "API timeout in milliseconds"
					pattern:     "^[0-9]+$"
					required:    true
				}
				emailNotificationsEnabled: {
					$ref:        "#/$defs/singleSelect"
					title:       "Email Notifications Enabled"
					description: "Whether email notifications are enabled"
					enum: ["true", "false"]
					required: true
				}
				pushNotificationsEnabled: {
					$ref:        "#/$defs/singleSelect"
					title:       "Push Notifications Enabled"
					description: "Whether push notifications are enabled"
					enum: ["true", "false"]
					required: true
				}
			}
			required: [
				"brandName",
				"brandDescription",
				"maxProposalAmount",
				"minProposalAmount",
				"totalBudget",
				"votingPowerCalculation",
				"minVotingPower",
				"defaultCampaignDuration",
				"proposalSubmissionDeadline",
				"apiBaseUrl",
				"apiTimeout",
				"emailNotificationsEnabled",
				"pushNotificationsEnabled",
			]
		}
	}
	versions: _generic_form_template_versions
}
