# Version

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The name of this version | 
**version_number** | **String** | The version number. Ideally will follow semantic versioning | 
**changelog** | Option<**String**> | The changelog for this version | [optional]
**dependencies** | Option<[**Vec<models::VersionDependency>**](VersionDependency.md)> | A list of specific versions of projects that this version depends on | [optional]
**game_versions** | **Vec<String>** | A list of versions of Minecraft that this version supports | 
**version_type** | **VersionType** | The release channel for this version (enum: release, beta, alpha) | 
**loaders** | **Vec<String>** | The mod loaders that this version supports. In case of resource packs, use \"minecraft\" | 
**featured** | **bool** | Whether the version is featured or not | 
**status** | Option<**Status**> |  (enum: listed, archived, draft, unlisted, scheduled, unknown) | [optional]
**requested_status** | Option<**RequestedStatus**> |  (enum: listed, archived, draft, unlisted) | [optional]
**id** | **String** | The ID of the version, encoded as a base62 string | 
**project_id** | **String** | The ID of the project this version is for | 
**author_id** | **String** | The ID of the author who published this version | 
**date_published** | **String** |  | 
**downloads** | **i32** | The number of times this version has been downloaded | 
**changelog_url** | Option<**String**> | A link to the changelog for this version. Always null, only kept for legacy compatibility. | [optional]
**files** | [**Vec<models::VersionFile>**](VersionFile.md) | A list of files available for download for this version | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


