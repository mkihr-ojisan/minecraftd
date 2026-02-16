# \TeamsApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_team_member**](TeamsApi.md#add_team_member) | **POST** /team/{id}/members | Add a user to a team
[**delete_team_member**](TeamsApi.md#delete_team_member) | **DELETE** /team/{id}/members/{id|username} | Remove a member from a team
[**get_project_team_members**](TeamsApi.md#get_project_team_members) | **GET** /project/{id|slug}/members | Get a project's team members
[**get_team_members**](TeamsApi.md#get_team_members) | **GET** /team/{id}/members | Get a team's members
[**get_teams**](TeamsApi.md#get_teams) | **GET** /teams | Get the members of multiple teams
[**join_team**](TeamsApi.md#join_team) | **POST** /team/{id}/join | Join a team
[**modify_team_member**](TeamsApi.md#modify_team_member) | **PATCH** /team/{id}/members/{id|username} | Modify a team member's information
[**transfer_team_ownership**](TeamsApi.md#transfer_team_ownership) | **PATCH** /team/{id}/owner | Transfer team's ownership to another user



## add_team_member

> add_team_member(id, user_identifier)
Add a user to a team

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the team | [required] |
**user_identifier** | Option<[**UserIdentifier**](UserIdentifier.md)> | User to be added (must be the ID, usernames cannot be used here) |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_team_member

> delete_team_member(id, id_pipe_username)
Remove a member from a team

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the team | [required] |
**id_pipe_username** | **String** | The ID or username of the user | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_project_team_members

> Vec<models::TeamMember> get_project_team_members(id_pipe_slug)
Get a project's team members

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |

### Return type

[**Vec<models::TeamMember>**](TeamMember.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_members

> Vec<models::TeamMember> get_team_members(id)
Get a team's members

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the team | [required] |

### Return type

[**Vec<models::TeamMember>**](TeamMember.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_teams

> Vec<Vec<models::TeamMember>> get_teams(ids)
Get the members of multiple teams

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs of the teams | [required] |

### Return type

[**Vec<Vec<models::TeamMember>>**](Vec.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## join_team

> join_team(id)
Join a team

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the team | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_team_member

> modify_team_member(id, id_pipe_username, modify_team_member_body)
Modify a team member's information

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the team | [required] |
**id_pipe_username** | **String** | The ID or username of the user | [required] |
**modify_team_member_body** | Option<[**ModifyTeamMemberBody**](ModifyTeamMemberBody.md)> | Contents to be modified |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## transfer_team_ownership

> transfer_team_ownership(id, user_identifier)
Transfer team's ownership to another user

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the team | [required] |
**user_identifier** | Option<[**UserIdentifier**](UserIdentifier.md)> | New owner's ID |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

