# \UsersApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**change_user_icon**](UsersApi.md#change_user_icon) | **PATCH** /user/{id|username}/icon | Change user's avatar
[**delete_user_icon**](UsersApi.md#delete_user_icon) | **DELETE** /user/{id|username}/icon | Remove user's avatar
[**get_followed_projects**](UsersApi.md#get_followed_projects) | **GET** /user/{id|username}/follows | Get user's followed projects
[**get_payout_history**](UsersApi.md#get_payout_history) | **GET** /user/{id|username}/payouts | Get user's payout history
[**get_user**](UsersApi.md#get_user) | **GET** /user/{id|username} | Get a user
[**get_user_from_auth**](UsersApi.md#get_user_from_auth) | **GET** /user | Get user from authorization header
[**get_user_projects**](UsersApi.md#get_user_projects) | **GET** /user/{id|username}/projects | Get user's projects
[**get_users**](UsersApi.md#get_users) | **GET** /users | Get multiple users
[**modify_user**](UsersApi.md#modify_user) | **PATCH** /user/{id|username} | Modify a user
[**withdraw_payout**](UsersApi.md#withdraw_payout) | **POST** /user/{id|username}/payouts | Withdraw payout balance to PayPal or Venmo



## change_user_icon

> change_user_icon(id_pipe_username, body)
Change user's avatar

The new avatar may be up to 2MiB in size.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |
**body** | Option<**std::path::PathBuf**> |  |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: image/png, image/jpeg, image/bmp, image/gif, image/webp, image/svg, image/svgz, image/rgb
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_user_icon

> delete_user_icon(id_pipe_username)
Remove user's avatar

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_followed_projects

> Vec<models::Project> get_followed_projects(id_pipe_username)
Get user's followed projects

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |

### Return type

[**Vec<models::Project>**](Project.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_payout_history

> models::UserPayoutHistory get_payout_history(id_pipe_username)
Get user's payout history

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |

### Return type

[**models::UserPayoutHistory**](UserPayoutHistory.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user

> models::User get_user(id_pipe_username)
Get a user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |

### Return type

[**models::User**](User.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_from_auth

> models::User get_user_from_auth()
Get user from authorization header

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::User**](User.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_projects

> Vec<models::Project> get_user_projects(id_pipe_username)
Get user's projects

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |

### Return type

[**Vec<models::Project>**](Project.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users

> Vec<models::User> get_users(ids)
Get multiple users

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs of the users | [required] |

### Return type

[**Vec<models::User>**](User.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_user

> modify_user(id_pipe_username, editable_user)
Modify a user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |
**editable_user** | Option<[**EditableUser**](EditableUser.md)> | Modified user fields |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## withdraw_payout

> withdraw_payout(id_pipe_username, amount)
Withdraw payout balance to PayPal or Venmo

Warning: certain amounts get withheld for fees. Please do not call this API endpoint without first acknowledging the warnings on the corresponding frontend page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |
**amount** | **i32** | Amount to withdraw | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

