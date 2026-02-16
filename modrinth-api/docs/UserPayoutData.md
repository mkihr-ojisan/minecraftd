# UserPayoutData

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**balance** | Option<**i32**> | The payout balance available for the user to withdraw (note, you cannot modify this in a PATCH request) | [optional]
**payout_wallet** | Option<**PayoutWallet**> | The wallet that the user has selected (enum: paypal, venmo) | [optional]
**payout_wallet_type** | Option<**PayoutWalletType**> | The type of the user's wallet (enum: email, phone, user_handle) | [optional]
**payout_address** | Option<**String**> | The user's payout address | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


