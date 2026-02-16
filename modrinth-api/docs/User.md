# User

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**username** | **String** | The user's username | 
**name** | Option<**String**> | The user's display name | [optional]
**email** | Option<**String**> | The user's email (only displayed if requesting your own account). Requires `USER_READ_EMAIL` PAT scope. | [optional]
**bio** | Option<**String**> | A description of the user | [optional]
**payout_data** | Option<[**models::UserPayoutData**](UserPayoutData.md)> |  | [optional]
**id** | **String** | The user's ID | 
**avatar_url** | **String** | The user's avatar url | 
**created** | **String** | The time at which the user was created | 
**role** | **Role** | The user's role (enum: admin, moderator, developer) | 
**badges** | Option<**i32**> | Any badges applicable to this user. These are currently unused and undisplayed, and as such are subject to change  In order from first to seventh bit, the current bits are: - (unused) - EARLY_MODPACK_ADOPTER - EARLY_RESPACK_ADOPTER - EARLY_PLUGIN_ADOPTER - ALPHA_TESTER - CONTRIBUTOR - TRANSLATOR  | [optional]
**auth_providers** | Option<**Vec<String>**> | A list of authentication providers you have signed up for (only displayed if requesting your own account) | [optional]
**email_verified** | Option<**bool**> | Whether your email is verified (only displayed if requesting your own account) | [optional]
**has_password** | Option<**bool**> | Whether you have a password associated with your account (only displayed if requesting your own account) | [optional]
**has_totp** | Option<**bool**> | Whether you have TOTP two-factor authentication connected to your account (only displayed if requesting your own account) | [optional]
**github_id** | Option<**i32**> | Deprecated - this is no longer public for security reasons and is always null | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


