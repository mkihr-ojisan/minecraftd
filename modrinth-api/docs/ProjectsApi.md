# \ProjectsApi

All URIs are relative to *https://api.modrinth.com/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_gallery_image**](ProjectsApi.md#add_gallery_image) | **POST** /project/{id|slug}/gallery | Add a gallery image
[**change_project_icon**](ProjectsApi.md#change_project_icon) | **PATCH** /project/{id|slug}/icon | Change project's icon
[**check_project_validity**](ProjectsApi.md#check_project_validity) | **GET** /project/{id|slug}/check | Check project slug/ID validity
[**create_project**](ProjectsApi.md#create_project) | **POST** /project | Create a project
[**delete_gallery_image**](ProjectsApi.md#delete_gallery_image) | **DELETE** /project/{id|slug}/gallery | Delete a gallery image
[**delete_project**](ProjectsApi.md#delete_project) | **DELETE** /project/{id|slug} | Delete a project
[**delete_project_icon**](ProjectsApi.md#delete_project_icon) | **DELETE** /project/{id|slug}/icon | Delete project's icon
[**follow_project**](ProjectsApi.md#follow_project) | **POST** /project/{id|slug}/follow | Follow a project
[**get_dependencies**](ProjectsApi.md#get_dependencies) | **GET** /project/{id|slug}/dependencies | Get all of a project's dependencies
[**get_project**](ProjectsApi.md#get_project) | **GET** /project/{id|slug} | Get a project
[**get_projects**](ProjectsApi.md#get_projects) | **GET** /projects | Get multiple projects
[**modify_gallery_image**](ProjectsApi.md#modify_gallery_image) | **PATCH** /project/{id|slug}/gallery | Modify a gallery image
[**modify_project**](ProjectsApi.md#modify_project) | **PATCH** /project/{id|slug} | Modify a project
[**patch_projects**](ProjectsApi.md#patch_projects) | **PATCH** /projects | Bulk-edit multiple projects
[**random_projects**](ProjectsApi.md#random_projects) | **GET** /projects_random | Get a list of random projects
[**schedule_project**](ProjectsApi.md#schedule_project) | **POST** /project/{id|slug}/schedule | Schedule a project
[**search_projects**](ProjectsApi.md#search_projects) | **GET** /search | Search projects
[**unfollow_project**](ProjectsApi.md#unfollow_project) | **DELETE** /project/{id|slug}/follow | Unfollow a project



## add_gallery_image

> add_gallery_image(id_pipe_slug, ext, featured, title, description, ordering, body)
Add a gallery image

Modrinth allows you to upload files of up to 5MiB to a project's gallery.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**ext** | **String** | Image extension | [required] |
**featured** | **bool** | Whether an image is featured | [required] |
**title** | Option<**String**> | Title of the image |  |
**description** | Option<**String**> | Description of the image |  |
**ordering** | Option<**i32**> | Ordering of the image |  |
**body** | Option<**std::path::PathBuf**> |  |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: image/png, image/jpeg, image/bmp, image/gif, image/webp, image/svg, image/svgz, image/rgb
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## change_project_icon

> change_project_icon(id_pipe_slug, ext, body)
Change project's icon

The new icon may be up to 256KiB in size.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**ext** | **String** | Image extension | [required] |
**body** | Option<**std::path::PathBuf**> |  |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: image/png, image/jpeg, image/bmp, image/gif, image/webp, image/svg, image/svgz, image/rgb
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## check_project_validity

> models::ProjectIdentifier check_project_validity(id_pipe_slug)
Check project slug/ID validity

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |

### Return type

[**models::ProjectIdentifier**](ProjectIdentifier.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_project

> models::Project create_project(data, icon)
Create a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**data** | [**models::CreatableProject**](CreatableProject.md) |  | [required] |
**icon** | Option<**std::path::PathBuf**> | Project icon file |  |

### Return type

[**models::Project**](Project.md)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_gallery_image

> delete_gallery_image(id_pipe_slug, url)
Delete a gallery image

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**url** | **String** | URL link of the image to delete | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_project

> delete_project(id_pipe_slug)
Delete a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_project_icon

> delete_project_icon(id_pipe_slug)
Delete project's icon

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## follow_project

> follow_project(id_pipe_slug)
Follow a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_dependencies

> models::ProjectDependencyList get_dependencies(id_pipe_slug)
Get all of a project's dependencies

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |

### Return type

[**models::ProjectDependencyList**](ProjectDependencyList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_project

> models::Project get_project(id_pipe_slug)
Get a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |

### Return type

[**models::Project**](Project.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_projects

> Vec<models::Project> get_projects(ids)
Get multiple projects

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs and/or slugs of the projects | [required] |

### Return type

[**Vec<models::Project>**](Project.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_gallery_image

> modify_gallery_image(id_pipe_slug, url, featured, title, description, ordering)
Modify a gallery image

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**url** | **String** | URL link of the image to modify | [required] |
**featured** | Option<**bool**> | Whether the image is featured |  |
**title** | Option<**String**> | New title of the image |  |
**description** | Option<**String**> | New description of the image |  |
**ordering** | Option<**i32**> | New ordering of the image |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_project

> modify_project(id_pipe_slug, editable_project)
Modify a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**editable_project** | Option<[**EditableProject**](EditableProject.md)> | Modified project fields |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## patch_projects

> patch_projects(ids, patch_projects_body)
Bulk-edit multiple projects

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | **String** | The IDs and/or slugs of the projects | [required] |
**patch_projects_body** | Option<[**PatchProjectsBody**](PatchProjectsBody.md)> | Fields to edit on all projects specified |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## random_projects

> Vec<models::Project> random_projects(count)
Get a list of random projects

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**count** | **i32** | The number of random projects to return | [required] |

### Return type

[**Vec<models::Project>**](Project.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## schedule_project

> schedule_project(id_pipe_slug, schedule)
Schedule a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |
**schedule** | Option<[**Schedule**](Schedule.md)> | Information about date and requested status |  |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_projects

> models::SearchResults search_projects(query, facets, index, offset, limit)
Search projects

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**query** | Option<**String**> | The query to search for |  |
**facets** | Option<**String**> | Facets are an essential concept for understanding how to filter out results.  These are the most commonly used facet types: - `project_type` - `categories` (loaders are lumped in with categories in search) - `versions` - `client_side` - `server_side` - `open_source`  Several others are also available for use, though these should not be used outside very specific use cases. - `title` - `author` - `follows` - `project_id` - `license` - `downloads` - `color` - `created_timestamp` (uses Unix timestamp) - `modified_timestamp` (uses Unix timestamp) - `date_created` (uses ISO-8601 timestamp) - `date_modified` (uses ISO-8601 timestamp)  In order to then use these facets, you need a value to filter by, as well as an operation to perform on this value. The most common operation is `:` (same as `=`), though you can also use `!=`, `>=`, `>`, `<=`, and `<`. Join together the type, operation, and value, and you've got your string. ``` {type} {operation} {value} ```  Examples: ``` categories = adventure versions != 1.20.1 downloads <= 100 ```  You then join these strings together in arrays to signal `AND` and `OR` operators.  ##### OR All elements in a single array are considered to be joined by OR statements. For example, the search `[[\"versions:1.16.5\", \"versions:1.17.1\"]]` translates to `Projects that support 1.16.5 OR 1.17.1`.  ##### AND Separate arrays are considered to be joined by AND statements. For example, the search `[[\"versions:1.16.5\"], [\"project_type:modpack\"]]` translates to `Projects that support 1.16.5 AND are modpacks`.  |  |
**index** | Option<**String**> | The sorting method used for sorting search results |  |[default to relevance]
**offset** | Option<**i32**> | The offset into the search. Skips this number of results |  |[default to 0]
**limit** | Option<**i32**> | The number of results returned by the search |  |[default to 10]

### Return type

[**models::SearchResults**](SearchResults.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unfollow_project

> unfollow_project(id_pipe_slug)
Unfollow a project

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id_pipe_slug** | **String** | The ID or slug of the project | [required] |

### Return type

 (empty response body)

### Authorization

[TokenAuth](../README.md#TokenAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

