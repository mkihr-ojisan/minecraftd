# ThreadMessageBody

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **Type** | The type of message (enum: status_change, text, thread_closure, deleted) | 
**body** | Option<**String**> | The actual message text. **Only present for `text` message type** | [optional]
**private** | Option<**bool**> | Whether the message is only visible to moderators. **Only present for `text` message type** | [optional]
**replying_to** | Option<**String**> | The ID of the message being replied to by this message. **Only present for `text` message type** | [optional]
**old_status** | Option<**OldStatus**> | The old status of the project. **Only present for `status_change` message type** (enum: approved, archived, rejected, draft, unlisted, processing, withheld, scheduled, private, unknown) | [optional]
**new_status** | Option<**NewStatus**> | The new status of the project. **Only present for `status_change` message type** (enum: approved, archived, rejected, draft, unlisted, processing, withheld, scheduled, private, unknown) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


