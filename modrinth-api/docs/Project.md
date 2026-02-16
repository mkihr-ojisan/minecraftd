# Project

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**slug** | **String** | The slug of a project, used for vanity URLs. Regex: ```^[\\w!@$()`.+,\"\\-']{3,64}$``` | 
**title** | **String** | The title or name of the project | 
**description** | **String** | A short description of the project | 
**categories** | **Vec<String>** | A list of the categories that the project has | 
**client_side** | **ClientSide** | The client side support of the project (enum: required, optional, unsupported, unknown) | 
**server_side** | **ServerSide** | The server side support of the project (enum: required, optional, unsupported, unknown) | 
**body** | **String** | A long form description of the project | 
**status** | **Status** | The status of the project (enum: approved, archived, rejected, draft, unlisted, processing, withheld, scheduled, private, unknown) | 
**requested_status** | Option<**RequestedStatus**> | The requested status when submitting for review or scheduling the project for release (enum: approved, archived, unlisted, private, draft) | [optional]
**additional_categories** | Option<**Vec<String>**> | A list of categories which are searchable but non-primary | [optional]
**issues_url** | Option<**String**> | An optional link to where to submit bugs or issues with the project | [optional]
**source_url** | Option<**String**> | An optional link to the source code of the project | [optional]
**wiki_url** | Option<**String**> | An optional link to the project's wiki page or other relevant information | [optional]
**discord_url** | Option<**String**> | An optional invite link to the project's discord | [optional]
**donation_urls** | Option<[**Vec<models::ProjectDonationUrl>**](ProjectDonationURL.md)> | A list of donation links for the project | [optional]
**project_type** | **ProjectType** | The project type of the project (enum: mod, modpack, resourcepack, shader) | 
**downloads** | **i32** | The total number of downloads of the project | 
**icon_url** | Option<**String**> | The URL of the project's icon | [optional]
**color** | Option<**i32**> | The RGB color of the project, automatically generated from the project icon | [optional]
**thread_id** | Option<**String**> | The ID of the moderation thread associated with this project | [optional]
**monetization_status** | Option<**MonetizationStatus**> |  (enum: monetized, demonetized, force-demonetized) | [optional]
**id** | **String** | The ID of the project, encoded as a base62 string | 
**team** | **String** | The ID of the team that has ownership of this project | 
**body_url** | Option<**String**> | The link to the long description of the project. Always null, only kept for legacy compatibility. | [optional]
**moderator_message** | Option<[**models::ModeratorMessage**](ModeratorMessage.md)> |  | [optional]
**published** | **String** | The date the project was published | 
**updated** | **String** | The date the project was last updated | 
**approved** | Option<**String**> | The date the project's status was set to an approved status | [optional]
**queued** | Option<**String**> | The date the project's status was submitted to moderators for review | [optional]
**followers** | **i32** | The total number of users following the project | 
**license** | Option<[**models::ProjectLicense**](ProjectLicense.md)> |  | [optional]
**versions** | Option<**Vec<String>**> | A list of the version IDs of the project (will never be empty unless `draft` status) | [optional]
**game_versions** | Option<**Vec<String>**> | A list of all of the game versions supported by the project | [optional]
**loaders** | Option<**Vec<String>**> | A list of all of the loaders supported by the project | [optional]
**gallery** | Option<[**Vec<models::GalleryImage>**](GalleryImage.md)> | A list of images that have been uploaded to the project's gallery | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


