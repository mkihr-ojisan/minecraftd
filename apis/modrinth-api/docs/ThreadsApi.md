# \ThreadsApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_thread_message**](ThreadsApi.md#delete_thread_message) | **DELETE** /message/{id} | Delete a thread message
[**get_open_reports**](ThreadsApi.md#get_open_reports) | **GET** /report | Get your open reports
[**get_report**](ThreadsApi.md#get_report) | **GET** /report/{id} | Get report from ID
[**get_reports**](ThreadsApi.md#get_reports) | **GET** /reports | Get multiple reports
[**get_thread**](ThreadsApi.md#get_thread) | **GET** /thread/{id} | Get a thread
[**get_threads**](ThreadsApi.md#get_threads) | **GET** /threads | Get multiple threads
[**modify_report**](ThreadsApi.md#modify_report) | **PATCH** /report/{id} | Modify a report
[**send_thread_message**](ThreadsApi.md#send_thread_message) | **POST** /thread/{id} | Send a text message to a thread
[**submit_report**](ThreadsApi.md#submit_report) | **POST** /report | Report a project, user, or version



## delete_thread_message

> delete_thread_message(id)
Delete a thread message

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the message | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_open_reports

> Vec<models::Report> get_open_reports(count)
Get your open reports

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**count** | Option<**i32**> |  |  |

### Return type

[**Vec<models::Report>**](Report.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_report

> models::Report get_report(id)
Get report from ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the report | [required] |

### Return type

[**models::Report**](Report.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_reports

> Vec<models::Report> get_reports(ids)
Get multiple reports

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs of the reports | [required] |

### Return type

[**Vec<models::Report>**](Report.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_thread

> models::Thread get_thread(id)
Get a thread

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the thread | [required] |

### Return type

[**models::Thread**](Thread.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_threads

> Vec<models::Thread> get_threads(ids)
Get multiple threads

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs of the threads | [required] |

### Return type

[**Vec<models::Thread>**](Thread.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_report

> modify_report(id, modify_report_request)
Modify a report

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the report | [required] |
**modify_report_request** | Option<[**ModifyReportRequest**](ModifyReportRequest.md)> | What to modify about the report |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## send_thread_message

> models::Thread send_thread_message(id, thread_message_body)
Send a text message to a thread

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the thread | [required] |
**thread_message_body** | Option<[**ThreadMessageBody**](ThreadMessageBody.md)> | The message to be sent. Note that you only need the fields applicable for the `text` type. |  |

### Return type

[**models::Thread**](Thread.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## submit_report

> models::Report submit_report(creatable_report)
Report a project, user, or version

Bring a project, user, or version to the attention of the moderators by reporting it.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**creatable_report** | Option<[**CreatableReport**](CreatableReport.md)> | The report to be sent |  |

### Return type

[**models::Report**](Report.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

