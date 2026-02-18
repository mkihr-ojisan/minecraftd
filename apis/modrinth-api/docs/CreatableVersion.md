# CreatableVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The name of this version | 
**version_number** | **String** | The version number. Ideally will follow semantic versioning | 
**changelog** | Option<**String**> | The changelog for this version | [optional]
**dependencies** | [**Vec<models::VersionDependency>**](VersionDependency.md) | A list of specific versions of projects that this version depends on | 
**game_versions** | **Vec<String>** | A list of versions of Minecraft that this version supports | 
**version_type** | **VersionType** | The release channel for this version (enum: release, beta, alpha) | 
**loaders** | **Vec<String>** | The mod loaders that this version supports. In case of resource packs, use \"minecraft\" | 
**featured** | **bool** | Whether the version is featured or not | 
**status** | Option<**Status**> |  (enum: listed, archived, draft, unlisted, scheduled, unknown) | [optional]
**requested_status** | Option<**RequestedStatus**> |  (enum: listed, archived, draft, unlisted) | [optional]
**project_id** | **String** | The ID of the project this version is for | 
**file_parts** | **Vec<String>** | An array of the multipart field names of each file that goes with this version | 
**primary_file** | Option<**String**> | The multipart field name of the primary file | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


