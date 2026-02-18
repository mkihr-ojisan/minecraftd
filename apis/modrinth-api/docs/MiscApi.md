# \MiscApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**forge_updates**](MiscApi.md#forge_updates) | **GET** /updates/{id|slug}/forge_updates.json | Forge Updates JSON file
[**statistics**](MiscApi.md#statistics) | **GET** /statistics | Various statistics about this Modrinth instance



## forge_updates

> models::ForgeUpdates forge_updates(id_pipe_slug, neoforge)
Forge Updates JSON file

If you're a Forge mod developer, your Modrinth mods have an automatically generated `updates.json` using the [Forge Update Checker](https://docs.minecraftforge.net/en/latest/misc/updatechecker/).  The only setup is to insert the URL into the `[[mods]]` section of your `mods.toml` file as such:  ```toml [[mods]] # the other stuff here - ID, version, display name, etc. updateJSONURL = \"https://api.modrinth.com/updates/{slug|ID}/forge_updates.json\" ```  Replace `{slug|id}` with the slug or ID of your project.  Modrinth will handle the rest! When you update your mod, Forge will notify your users that their copy of your mod is out of date.  Make sure that the version format you use for your Modrinth releases is the same as the version format you use in your `mods.toml`. If you use a format such as `1.2.3-forge` or `1.2.3+1.19` with your Modrinth releases but your `mods.toml` only has `1.2.3`, the update checker may not function properly.  If you're using NeoForge, NeoForge versions will, by default, not appear in the default URL. You will need to add `?neoforge=only` to show your NeoForge-only versions, or `?neoforge=include` for both.  ```toml [[mods]] # the other stuff here - ID, version, display name, etc. updateJSONURL = \"https://api.modrinth.com/updates/{slug|ID}/forge_updates.json?neoforge=only\" ``` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**neoforge** | Option<**String**> | Whether to include NeoForge versions. Can be `only` (NeoForge-only versions), `include` (both Forge and NeoForge versions), or omitted (Forge-only versions). |  |

### Return type

[**models::ForgeUpdates**](ForgeUpdates.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## statistics

> models::Statistics statistics()
Various statistics about this Modrinth instance

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Statistics**](Statistics.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

