# \TagsApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**category_list**](TagsApi.md#category_list) | **GET** /tag/category | Get a list of categories
[**donation_platform_list**](TagsApi.md#donation_platform_list) | **GET** /tag/donation_platform | Get a list of donation platforms
[**license_list**](TagsApi.md#license_list) | **GET** /tag/license | Get a list of licenses
[**license_text**](TagsApi.md#license_text) | **GET** /tag/license/{id} | Get the text and title of a license
[**loader_list**](TagsApi.md#loader_list) | **GET** /tag/loader | Get a list of loaders
[**project_type_list**](TagsApi.md#project_type_list) | **GET** /tag/project_type | Get a list of project types
[**report_type_list**](TagsApi.md#report_type_list) | **GET** /tag/report_type | Get a list of report types
[**side_type_list**](TagsApi.md#side_type_list) | **GET** /tag/side_type | Get a list of side types
[**version_list**](TagsApi.md#version_list) | **GET** /tag/game_version | Get a list of game versions



## category_list

> Vec<models::CategoryTag> category_list()
Get a list of categories

Gets an array of categories, their icons, and applicable project types

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::CategoryTag>**](CategoryTag.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## donation_platform_list

> Vec<models::DonationPlatformTag> donation_platform_list()
Get a list of donation platforms

Gets an array of donation platforms and information about them

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::DonationPlatformTag>**](DonationPlatformTag.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## license_list

> Vec<models::LicenseTag> license_list()
Get a list of licenses

Deprecated - simply use SPDX IDs.

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::LicenseTag>**](LicenseTag.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## license_text

> models::License license_text(id)
Get the text and title of a license

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The license ID to get the text of | [required] |

### Return type

[**models::License**](License.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## loader_list

> Vec<models::LoaderTag> loader_list()
Get a list of loaders

Gets an array of loaders, their icons, and supported project types

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::LoaderTag>**](LoaderTag.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## project_type_list

> Vec<String> project_type_list()
Get a list of project types

Gets an array of valid project types

### Parameters

This endpoint does not need any parameter.

### Return type

**Vec<String>**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## report_type_list

> Vec<String> report_type_list()
Get a list of report types

Gets an array of valid report types

### Parameters

This endpoint does not need any parameter.

### Return type

**Vec<String>**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## side_type_list

> Vec<String> side_type_list()
Get a list of side types

Gets an array of valid side types

### Parameters

This endpoint does not need any parameter.

### Return type

**Vec<String>**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## version_list

> Vec<models::GameVersionTag> version_list()
Get a list of game versions

Gets an array of game versions and information about them

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::GameVersionTag>**](GameVersionTag.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

