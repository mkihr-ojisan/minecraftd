# EditableVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | Option<**String**> | The name of this version | [optional]
**version_number** | Option<**String**> | The version number. Ideally will follow semantic versioning | [optional]
**changelog** | Option<**String**> | The changelog for this version | [optional]
**dependencies** | Option<[**Vec<models::VersionDependency>**](VersionDependency.md)> | A list of specific versions of projects that this version depends on | [optional]
**game_versions** | Option<**Vec<String>**> | A list of versions of Minecraft that this version supports | [optional]
**version_type** | Option<**VersionType**> | The release channel for this version (enum: release, beta, alpha) | [optional]
**loaders** | Option<**Vec<String>**> | The mod loaders that this version supports. In case of resource packs, use \"minecraft\" | [optional]
**featured** | Option<**bool**> | Whether the version is featured or not | [optional]
**status** | Option<**Status**> |  (enum: listed, archived, draft, unlisted, scheduled, unknown) | [optional]
**requested_status** | Option<**RequestedStatus**> |  (enum: listed, archived, draft, unlisted) | [optional]
**primary_file** | Option<**Vec<String>**> | The hash format and the hash of the new primary file | [optional]
**file_types** | Option<[**Vec<models::EditableFileType>**](EditableFileType.md)> | A list of file_types to edit | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


