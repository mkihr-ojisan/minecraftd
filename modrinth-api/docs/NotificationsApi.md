# \NotificationsApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_notification**](NotificationsApi.md#delete_notification) | **DELETE** /notification/{id} | Delete notification
[**delete_notifications**](NotificationsApi.md#delete_notifications) | **DELETE** /notifications | Delete multiple notifications
[**get_notification**](NotificationsApi.md#get_notification) | **GET** /notification/{id} | Get notification from ID
[**get_notifications**](NotificationsApi.md#get_notifications) | **GET** /notifications | Get multiple notifications
[**get_user_notifications**](NotificationsApi.md#get_user_notifications) | **GET** /user/{id|username}/notifications | Get user's notifications
[**read_notification**](NotificationsApi.md#read_notification) | **PATCH** /notification/{id} | Mark notification as read
[**read_notifications**](NotificationsApi.md#read_notifications) | **PATCH** /notifications | Mark multiple notifications as read



## delete_notification

> delete_notification(id)
Delete notification

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the notification | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_notifications

> delete_notifications(ids)
Delete multiple notifications

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs of the notifications | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_notification

> models::Notification get_notification(id)
Get notification from ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the notification | [required] |

### Return type

[**models::Notification**](Notification.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_notifications

> Vec<models::Notification> get_notifications(ids)
Get multiple notifications

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs of the notifications | [required] |

### Return type

[**Vec<models::Notification>**](Notification.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_notifications

> Vec<models::Notification> get_user_notifications(id_pipe_username)
Get user's notifications

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_username** | **String** | The ID or username of the user | [required] |

### Return type

[**Vec<models::Notification>**](Notification.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## read_notification

> read_notification(id)
Mark notification as read

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the notification | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## read_notifications

> read_notifications(ids)
Mark multiple notifications as read

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs of the notifications | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

