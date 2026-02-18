# Notification

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | The id of the notification | 
**user_id** | **String** | The id of the user who received the notification | 
**r#type** | Option<**Type**> | The type of notification (enum: project_update, team_invite, status_change, moderator_message) | [optional]
**title** | **String** | The title of the notification | 
**text** | **String** | The body text of the notification | 
**link** | **String** | A link to the related project or version | 
**read** | **bool** | Whether the notification has been read or not | 
**created** | **String** | The time at which the notification was created | 
**actions** | [**Vec<models::NotificationAction>**](NotificationAction.md) | A list of actions that can be performed | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


