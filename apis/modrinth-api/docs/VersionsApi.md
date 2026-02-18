# \VersionsApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_files_to_version**](VersionsApi.md#add_files_to_version) | **POST** /version/{id}/file | Add files to version
[**create_version**](VersionsApi.md#create_version) | **POST** /version | Create a version
[**delete_version**](VersionsApi.md#delete_version) | **DELETE** /version/{id} | Delete a version
[**get_project_versions**](VersionsApi.md#get_project_versions) | **GET** /project/{id|slug}/version | List project's versions
[**get_version**](VersionsApi.md#get_version) | **GET** /version/{id} | Get a version
[**get_version_from_id_or_number**](VersionsApi.md#get_version_from_id_or_number) | **GET** /project/{id|slug}/version/{id|number} | Get a version given a version number or ID
[**get_versions**](VersionsApi.md#get_versions) | **GET** /versions | Get multiple versions
[**modify_version**](VersionsApi.md#modify_version) | **PATCH** /version/{id} | Modify a version
[**schedule_version**](VersionsApi.md#schedule_version) | **POST** /version/{id}/schedule | Schedule a version



## add_files_to_version

> add_files_to_version(id, data)
Add files to version

Project files are attached. `.mrpack` and `.jar` files are accepted.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the version | [required] |
**data** | Option<[**serde_json::Value**](SerdeJson__Value.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_version

> models::Version create_version(data)
Create a version

This route creates a version on an existing project. There must be at least one file attached to each new version, unless the new version's status is `draft`. `.mrpack`, `.jar`, `.zip`, and `.litemod` files are accepted.  The request is a [multipart request](https://www.ietf.org/rfc/rfc2388.txt) with at least two form fields: one is `data`, which includes a JSON body with the version metadata as shown below, and at least one field containing an upload file.  You can name the file parts anything you would like, but you must list each of the parts' names in `file_parts`, and optionally, provide one to use as the primary file in `primary_file`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**data** | [**models::CreatableVersion**](CreatableVersion.md) |  | [required] |

### Return type

[**models::Version**](Version.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_version

> delete_version(id)
Delete a version

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the version | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_project_versions

> Vec<models::Version> get_project_versions(id_pipe_slug, loaders, game_versions, featured, include_changelog)
List project's versions

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**loaders** | Option<**String**> | The types of loaders to filter for |  |
**game_versions** | Option<**String**> | The game versions to filter for |  |
**featured** | Option<**bool**> | Allows to filter for featured or non-featured versions only |  |
**include_changelog** | Option<**bool**> | Allows you to toggle the inclusion of the changelog field in the response. It is highly recommended to use include_changelog=false in most cases unless you specifically need the changelog for all versions. |  |[default to true]

### Return type

[**Vec<models::Version>**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_version

> models::Version get_version(id)
Get a version

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the version | [required] |

### Return type

[**models::Version**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_version_from_id_or_number

> models::Version get_version_from_id_or_number(id_pipe_slug, id_pipe_number)
Get a version given a version number or ID

Please note that, if the version number provided matches multiple versions, only the **oldest matching version** will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**id_pipe_number** | **String** | The version ID or version number | [required] |

### Return type

[**models::Version**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_versions

> Vec<models::Version> get_versions(ids)
Get multiple versions

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs of the versions | [required] |

### Return type

[**Vec<models::Version>**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_version

> modify_version(id, editable_version)
Modify a version

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the version | [required] |
**editable_version** | Option<[**EditableVersion**](EditableVersion.md)> | Modified version fields |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## schedule_version

> schedule_version(id, schedule)
Schedule a version

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the version | [required] |
**schedule** | Option<[**Schedule**](Schedule.md)> | Information about date and requested status |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

