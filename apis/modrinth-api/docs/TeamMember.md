# TeamMember

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**team_id** | **String** | The ID of the team this team member is a member of | 
**user** | [**models::User**](User.md) |  | 
**role** | **String** | The user's role on the team | 
**permissions** | Option<**i32**> | The user's permissions in bitfield format (requires authorization to view)  In order from first to tenth bit, the bits are: - UPLOAD_VERSION - DELETE_VERSION - EDIT_DETAILS - EDIT_BODY - MANAGE_INVITES - REMOVE_MEMBER - EDIT_MEMBER - DELETE_PROJECT - VIEW_ANALYTICS - VIEW_PAYOUTS  | [optional]
**accepted** | **bool** | Whether or not the user has accepted to be on the team (requires authorization to view) | 
**payouts_split** | Option<**i32**> | The split of payouts going to this user. The proportion of payouts they get is their split divided by the sum of the splits of all members. | [optional]
**ordering** | Option<**i32**> | The order of the team member. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


