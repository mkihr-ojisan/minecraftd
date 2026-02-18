# Report

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**report_type** | **String** | The type of the report being sent | 
**item_id** | **String** | The ID of the item (project, version, or user) being reported | 
**item_type** | **ItemType** | The type of the item being reported (enum: project, user, version) | 
**body** | **String** | The extended explanation of the report | 
**id** | Option<**String**> | The ID of the report | [optional]
**reporter** | **String** | The ID of the user who reported the item | 
**created** | **String** | The time at which the report was created | 
**closed** | **bool** | Whether the report is resolved | 
**thread_id** | **String** | The ID of the moderation thread associated with this report | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


