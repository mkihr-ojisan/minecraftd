# \VersionFilesApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_file_from_hash**](VersionFilesApi.md#delete_file_from_hash) | **DELETE** /version_file/{hash} | Delete a file from its hash
[**get_latest_version_from_hash**](VersionFilesApi.md#get_latest_version_from_hash) | **POST** /version_file/{hash}/update | Latest version of a project from a hash, loader(s), and game version(s)
[**get_latest_versions_from_hashes**](VersionFilesApi.md#get_latest_versions_from_hashes) | **POST** /version_files/update | Latest versions of multiple project from hashes, loader(s), and game version(s)
[**version_from_hash**](VersionFilesApi.md#version_from_hash) | **GET** /version_file/{hash} | Get version from hash
[**versions_from_hashes**](VersionFilesApi.md#versions_from_hashes) | **POST** /version_files | Get versions from hashes



## delete_file_from_hash

> delete_file_from_hash(hash, algorithm, version_id)
Delete a file from its hash

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hash** | **String** | The hash of the file, considering its byte content, and encoded in hexadecimal | [required] |
**algorithm** | **String** | The algorithm of the hash | [required] |[default to sha1]
**version_id** | Option<**String**> | Version ID to delete the version from, if multiple files of the same hash exist |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_latest_version_from_hash

> models::Version get_latest_version_from_hash(hash, algorithm, get_latest_version_from_hash_body)
Latest version of a project from a hash, loader(s), and game version(s)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hash** | **String** | The hash of the file, considering its byte content, and encoded in hexadecimal | [required] |
**algorithm** | **String** | The algorithm of the hash | [required] |[default to sha1]
**get_latest_version_from_hash_body** | Option<[**GetLatestVersionFromHashBody**](GetLatestVersionFromHashBody.md)> | Parameters of the updated version requested |  |

### Return type

[**models::Version**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_latest_versions_from_hashes

> std::collections::HashMap<String, models::Version> get_latest_versions_from_hashes(get_latest_versions_from_hashes_body)
Latest versions of multiple project from hashes, loader(s), and game version(s)

This is the same as [`/version_file/{hash}/update`](#operation/getLatestVersionFromHash) except it accepts multiple hashes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**get_latest_versions_from_hashes_body** | Option<[**GetLatestVersionsFromHashesBody**](GetLatestVersionsFromHashesBody.md)> | Parameters of the updated version requested |  |

### Return type

[**std::collections::HashMap<String, models::Version>**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## version_from_hash

> models::Version version_from_hash(hash, algorithm, multiple)
Get version from hash

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hash** | **String** | The hash of the file, considering its byte content, and encoded in hexadecimal | [required] |
**algorithm** | **String** | The algorithm of the hash | [required] |[default to sha1]
**multiple** | Option<**bool**> | Whether to return multiple results when looking for this hash |  |[default to false]

### Return type

[**models::Version**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## versions_from_hashes

> std::collections::HashMap<String, models::Version> versions_from_hashes(hash_list)
Get versions from hashes

This is the same as [`/version_file/{hash}`](#operation/versionFromHash) except it accepts multiple hashes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hash_list** | Option<[**HashList**](HashList.md)> | Hashes and algorithm of the versions requested |  |

### Return type

[**std::collections::HashMap<String, models::Version>**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

