# ProjectResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**slug** | **String** | The slug of a project, used for vanity URLs. Regex: ```^[\\w!@$()`.+,\"\\-']{3,64}$``` | 
**title** | **String** | The title or name of the project | 
**description** | **String** | A short description of the project | 
**categories** | Option<**Vec<String>**> | A list of the categories that the project has | [optional]
**client_side** | **ClientSide** | The client side support of the project (enum: required, optional, unsupported, unknown) | 
**server_side** | **ServerSide** | The server side support of the project (enum: required, optional, unsupported, unknown) | 
**project_type** | **ProjectType** | The project type of the project (enum: mod, modpack, resourcepack, shader) | 
**downloads** | **i32** | The total number of downloads of the project | 
**icon_url** | Option<**String**> | The URL of the project's icon | [optional]
**color** | Option<**i32**> | The RGB color of the project, automatically generated from the project icon | [optional]
**thread_id** | Option<**String**> | The ID of the moderation thread associated with this project | [optional]
**monetization_status** | Option<**MonetizationStatus**> |  (enum: monetized, demonetized, force-demonetized) | [optional]
**project_id** | **String** | The ID of the project | 
**author** | **String** | The username of the project's author | 
**display_categories** | Option<**Vec<String>**> | A list of the categories that the project has which are not secondary | [optional]
**versions** | **Vec<String>** | A list of the minecraft versions supported by the project | 
**follows** | **i32** | The total number of users following the project | 
**date_created** | **String** | The date the project was added to search | 
**date_modified** | **String** | The date the project was last modified | 
**latest_version** | Option<**String**> | The latest version of minecraft that this project supports | [optional]
**license** | **String** | The SPDX license ID of a project | 
**gallery** | Option<**Vec<String>**> | All gallery images attached to the project | [optional]
**featured_gallery** | Option<**String**> | The featured gallery image of the project | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


