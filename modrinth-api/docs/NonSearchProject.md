# NonSearchProject

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**slug** | Option<**String**> | The slug of a project, used for vanity URLs. Regex: ```^[\\w!@$()`.+,\"\\-']{3,64}$``` | [optional]
**title** | Option<**String**> | The title or name of the project | [optional]
**description** | Option<**String**> | A short description of the project | [optional]
**categories** | Option<**Vec<String>**> | A list of the categories that the project has | [optional]
**client_side** | Option<**ClientSide**> | The client side support of the project (enum: required, optional, unsupported, unknown) | [optional]
**server_side** | Option<**ServerSide**> | The server side support of the project (enum: required, optional, unsupported, unknown) | [optional]
**body** | Option<**String**> | A long form description of the project | [optional]
**status** | Option<**Status**> | The status of the project (enum: approved, archived, rejected, draft, unlisted, processing, withheld, scheduled, private, unknown) | [optional]
**requested_status** | Option<**RequestedStatus**> | The requested status when submitting for review or scheduling the project for release (enum: approved, archived, unlisted, private, draft) | [optional]
**additional_categories** | Option<**Vec<String>**> | A list of categories which are searchable but non-primary | [optional]
**issues_url** | Option<**String**> | An optional link to where to submit bugs or issues with the project | [optional]
**source_url** | Option<**String**> | An optional link to the source code of the project | [optional]
**wiki_url** | Option<**String**> | An optional link to the project's wiki page or other relevant information | [optional]
**discord_url** | Option<**String**> | An optional invite link to the project's discord | [optional]
**donation_urls** | Option<[**Vec<models::ProjectDonationUrl>**](ProjectDonationURL.md)> | A list of donation links for the project | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


