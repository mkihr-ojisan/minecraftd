# BaseProject

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**slug** | Option<**String**> | The slug of a project, used for vanity URLs. Regex: ```^[\\w!@$()`.+,\"\\-']{3,64}$``` | [optional]
**title** | Option<**String**> | The title or name of the project | [optional]
**description** | Option<**String**> | A short description of the project | [optional]
**categories** | Option<**Vec<String>**> | A list of the categories that the project has | [optional]
**client_side** | Option<**ClientSide**> | The client side support of the project (enum: required, optional, unsupported, unknown) | [optional]
**server_side** | Option<**ServerSide**> | The server side support of the project (enum: required, optional, unsupported, unknown) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


