# Thread

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | The ID of the thread | 
**r#type** | **Type** |  (enum: project, report, direct_message) | 
**project_id** | Option<**String**> | The ID of the associated project if a project thread | [optional]
**report_id** | Option<**String**> | The ID of the associated report if a report thread | [optional]
**messages** | [**Vec<models::ThreadMessage>**](ThreadMessage.md) |  | 
**members** | [**Vec<models::User>**](User.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


