# Rust API client for openapi

This documentation doesn't provide a way to test our API. In order to facilitate testing, we recommend the following tools:

- [cURL](https://curl.se/) (recommended, command-line)
- [ReqBIN](https://reqbin.com/) (recommended, online)
- [Postman](https://www.postman.com/downloads/)
- [Insomnia](https://insomnia.rest/)
- Your web browser, if you don't need to send headers or a request body

Once you have a working client, you can test that it works by making a `GET` request to `https://staging-api.modrinth.com/`:

```json
{
  \"about\": \"Welcome traveler!\",
  \"documentation\": \"https://docs.modrinth.com\",
  \"name\": \"modrinth-labrinth\",
  \"version\": \"2.7.0\"
}
```

If you got a response similar to the one above, you can use the Modrinth API!
When you want to go live using the production API, use `api.modrinth.com` instead of `staging-api.modrinth.com`.

## Authentication
This API has two options for authentication: personal access tokens and [OAuth2](https://en.wikipedia.org/wiki/OAuth).
All tokens are tied to a Modrinth user and use the `Authorization` header of the request.

Example:
```
Authorization: mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic
```

You do not need a token for most requests. Generally speaking, only the following types of requests require a token:
- those which create data (such as version creation)
- those which modify data (such as editing a project)
- those which access private data (such as draft projects, notifications, emails, and payout data)

Each request requiring authentication has a certain scope. For example, to view the email of the user being requested, the token must have the `USER_READ_EMAIL` scope.
You can find the list of available scopes [on GitHub](https://github.com/modrinth/labrinth/blob/master/src/models/pats.rs#L15). Making a request with an invalid scope will return a 401 error.

Please note that certain scopes and requests cannot be completed with a personal access token or using OAuth.
For example, deleting a user account can only be done through Modrinth's frontend.

A detailed guide on OAuth has been published in [Modrinth's technical documentation](https://docs.modrinth.com/guide/oauth).

### Personal access tokens
Personal access tokens (PATs) can be generated in from [the user settings](https://modrinth.com/settings/account).

### GitHub tokens
For backwards compatibility purposes, some types of GitHub tokens also work for authenticating a user with Modrinth's API, granting all scopes.
**We urge any application still using GitHub tokens to start using personal access tokens for security and reliability purposes.**
GitHub tokens will cease to function to authenticate with Modrinth's API as soon as version 3 of the API is made generally available.

## Cross-Origin Resource Sharing
This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with the [W3C spec](https://www.w3.org/TR/cors/).
This allows for cross-domain communication from the browser.
All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site.

## Identifiers
The majority of items you can interact with in the API have a unique eight-digit base62 ID.
Projects, versions, users, threads, teams, and reports all use this same way of identifying themselves.
Version files use the sha1 or sha512 file hashes as identifiers.

Each project and user has a friendlier way of identifying them; slugs and usernames, respectively.
While unique IDs are constant, slugs and usernames can change at any moment.
If you want to store something in the long term, it is recommended to use the unique ID.

## Ratelimits
The API has a ratelimit defined per IP. Limits and remaining amounts are given in the response headers.
- `X-Ratelimit-Limit`: the maximum number of requests that can be made in a minute
- `X-Ratelimit-Remaining`: the number of requests remaining in the current ratelimit window
- `X-Ratelimit-Reset`: the time in seconds until the ratelimit window resets

Ratelimits are the same no matter whether you use a token or not.
The ratelimit is currently 300 requests per minute. If you have a use case requiring a higher limit, please [contact us](mailto:admin@modrinth.com).

## User Agents
To access the Modrinth API, you **must** use provide a uniquely-identifying `User-Agent` header.
Providing a user agent that only identifies your HTTP client library (such as \"okhttp/4.9.3\") increases the likelihood that we will block your traffic.
It is recommended, but not required, to include contact information in your user agent.
This allows us to contact you if we would like a change in your application's behavior without having to block your traffic.
- Bad: `User-Agent: okhttp/4.9.3`
- Good: `User-Agent: project_name`
- Better: `User-Agent: github_username/project_name/1.56.0`
- Best: `User-Agent: github_username/project_name/1.56.0 (launcher.com)` or `User-Agent: github_username/project_name/1.56.0 (contact@launcher.com)`

## Versioning
Modrinth follows a simple pattern for its API versioning.
In the event of a breaking API change, the API version in the URL path is bumped, and migration steps will be published below.

When an API is no longer the current one, it will immediately be considered deprecated.
No more support will be provided for API versions older than the current one.
It will be kept for some time, but this amount of time is not certain.

We will exercise various tactics to get people to update their implementation of our API.
One example is by adding something like `STOP USING THIS API` to various data returned by the API.

Once an API version is completely deprecated, it will permanently return a 410 error.
Please ensure your application handles these 410 errors.

### Migrations
Inside the following spoiler, you will be able to find all changes between versions of the Modrinth API, accompanied by tips and a guide to migrate applications to newer versions.

Here, you can also find changes for [Minotaur](https://github.com/modrinth/minotaur), Modrinth's official Gradle plugin. Major versions of Minotaur directly correspond to major versions of the Modrinth API.

<details><summary>API v1 to API v2</summary>

These bullet points cover most changes in the v2 API, but please note that fields containing `mod` in most contexts have been shifted to `project`.  For example, in the search route, the field `mod_id` was renamed to `project_id`.

- The search route has been moved from `/api/v1/mod` to `/v2/search`
- New project fields: `project_type` (may be `mod` or `modpack`), `moderation_message` (which has a `message` and `body`), `gallery`
- New search facet: `project_type`
- Alphabetical sort removed (it didn't work and is not possible due to limits in MeiliSearch)
- New search fields: `project_type`, `gallery`
  - The gallery field is an array of URLs to images that are part of the project's gallery
- The gallery is a new feature which allows the user to upload images showcasing their mod to the CDN which will be displayed on their mod page
- Internal change: Any project file uploaded to Modrinth is now validated to make sure it's a valid Minecraft mod, Modpack, etc.
  - For example, a Forge 1.17 mod with a JAR not containing a mods.toml will not be allowed to be uploaded to Modrinth
- In project creation, projects may not upload a mod with no versions to review, however they can be saved as a draft
  - Similarly, for version creation, a version may not be uploaded without any files
- Donation URLs have been enabled
- New project status: `archived`. Projects with this status do not appear in search
- Tags (such as categories, loaders) now have icons (SVGs) and specific project types attached
- Dependencies have been wiped and replaced with a new system
- Notifications now have a `type` field, such as `project_update`

Along with this, project subroutes (such as `/v2/project/{id}/version`) now allow the slug to be used as the ID. This is also the case with user routes.

</details><details><summary>Minotaur v1 to Minotaur v2</summary>

Minotaur 2.x introduced a few breaking changes to how your buildscript is formatted.

First, instead of registering your own `publishModrinth` task, Minotaur now automatically creates a `modrinth` task. As such, you can replace the `task publishModrinth(type: TaskModrinthUpload) {` line with just `modrinth {`.

To declare supported Minecraft versions and mod loaders, the `gameVersions` and `loaders` arrays must now be used. The syntax for these are pretty self-explanatory.

Instead of using `releaseType`, you must now use `versionType`. This was actually changed in v1.2.0, but very few buildscripts have moved on from v1.1.0.

Dependencies have been changed to a special DSL. Create a `dependencies` block within the `modrinth` block, and then use `scope.type(\"project/version\")`. For example, `required.project(\"fabric-api\")` adds a required project dependency on Fabric API.

You may now use the slug anywhere that a project ID was previously required.

</details>


For more information, please visit [https://support.modrinth.com](https://support.modrinth.com)

## Overview

This API client was generated by the [OpenAPI Generator](https://openapi-generator.tech) project.  By using the [openapi-spec](https://openapis.org) from a remote server, you can easily generate an API client.

- API version: v2.7.0/366f528
- Package version: v2.7.0/366f528
- Generator version: 7.20.0
- Build package: `org.openapitools.codegen.languages.RustClientCodegen`

## Installation

Put the package under your project folder in a directory named `openapi` and add the following to `Cargo.toml` under `[dependencies]`:

```
openapi = { path = "./openapi" }
```

## Documentation for API Endpoints

All URIs are relative to *https://api.modrinth.com/v2*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*MiscApi* | [**forge_updates**](docs/MiscApi.md#forge_updates) | **GET** /updates/{id|slug}/forge_updates.json | Forge Updates JSON file
*MiscApi* | [**statistics**](docs/MiscApi.md#statistics) | **GET** /statistics | Various statistics about this Modrinth instance
*NotificationsApi* | [**delete_notification**](docs/NotificationsApi.md#delete_notification) | **DELETE** /notification/{id} | Delete notification
*NotificationsApi* | [**delete_notifications**](docs/NotificationsApi.md#delete_notifications) | **DELETE** /notifications | Delete multiple notifications
*NotificationsApi* | [**get_notification**](docs/NotificationsApi.md#get_notification) | **GET** /notification/{id} | Get notification from ID
*NotificationsApi* | [**get_notifications**](docs/NotificationsApi.md#get_notifications) | **GET** /notifications | Get multiple notifications
*NotificationsApi* | [**get_user_notifications**](docs/NotificationsApi.md#get_user_notifications) | **GET** /user/{id|username}/notifications | Get user's notifications
*NotificationsApi* | [**read_notification**](docs/NotificationsApi.md#read_notification) | **PATCH** /notification/{id} | Mark notification as read
*NotificationsApi* | [**read_notifications**](docs/NotificationsApi.md#read_notifications) | **PATCH** /notifications | Mark multiple notifications as read
*ProjectsApi* | [**add_gallery_image**](docs/ProjectsApi.md#add_gallery_image) | **POST** /project/{id|slug}/gallery | Add a gallery image
*ProjectsApi* | [**change_project_icon**](docs/ProjectsApi.md#change_project_icon) | **PATCH** /project/{id|slug}/icon | Change project's icon
*ProjectsApi* | [**check_project_validity**](docs/ProjectsApi.md#check_project_validity) | **GET** /project/{id|slug}/check | Check project slug/ID validity
*ProjectsApi* | [**create_project**](docs/ProjectsApi.md#create_project) | **POST** /project | Create a project
*ProjectsApi* | [**delete_gallery_image**](docs/ProjectsApi.md#delete_gallery_image) | **DELETE** /project/{id|slug}/gallery | Delete a gallery image
*ProjectsApi* | [**delete_project**](docs/ProjectsApi.md#delete_project) | **DELETE** /project/{id|slug} | Delete a project
*ProjectsApi* | [**delete_project_icon**](docs/ProjectsApi.md#delete_project_icon) | **DELETE** /project/{id|slug}/icon | Delete project's icon
*ProjectsApi* | [**follow_project**](docs/ProjectsApi.md#follow_project) | **POST** /project/{id|slug}/follow | Follow a project
*ProjectsApi* | [**get_dependencies**](docs/ProjectsApi.md#get_dependencies) | **GET** /project/{id|slug}/dependencies | Get all of a project's dependencies
*ProjectsApi* | [**get_project**](docs/ProjectsApi.md#get_project) | **GET** /project/{id|slug} | Get a project
*ProjectsApi* | [**get_projects**](docs/ProjectsApi.md#get_projects) | **GET** /projects | Get multiple projects
*ProjectsApi* | [**modify_gallery_image**](docs/ProjectsApi.md#modify_gallery_image) | **PATCH** /project/{id|slug}/gallery | Modify a gallery image
*ProjectsApi* | [**modify_project**](docs/ProjectsApi.md#modify_project) | **PATCH** /project/{id|slug} | Modify a project
*ProjectsApi* | [**patch_projects**](docs/ProjectsApi.md#patch_projects) | **PATCH** /projects | Bulk-edit multiple projects
*ProjectsApi* | [**random_projects**](docs/ProjectsApi.md#random_projects) | **GET** /projects_random | Get a list of random projects
*ProjectsApi* | [**schedule_project**](docs/ProjectsApi.md#schedule_project) | **POST** /project/{id|slug}/schedule | Schedule a project
*ProjectsApi* | [**search_projects**](docs/ProjectsApi.md#search_projects) | **GET** /search | Search projects
*ProjectsApi* | [**unfollow_project**](docs/ProjectsApi.md#unfollow_project) | **DELETE** /project/{id|slug}/follow | Unfollow a project
*TagsApi* | [**category_list**](docs/TagsApi.md#category_list) | **GET** /tag/category | Get a list of categories
*TagsApi* | [**donation_platform_list**](docs/TagsApi.md#donation_platform_list) | **GET** /tag/donation_platform | Get a list of donation platforms
*TagsApi* | [**license_list**](docs/TagsApi.md#license_list) | **GET** /tag/license | Get a list of licenses
*TagsApi* | [**license_text**](docs/TagsApi.md#license_text) | **GET** /tag/license/{id} | Get the text and title of a license
*TagsApi* | [**loader_list**](docs/TagsApi.md#loader_list) | **GET** /tag/loader | Get a list of loaders
*TagsApi* | [**project_type_list**](docs/TagsApi.md#project_type_list) | **GET** /tag/project_type | Get a list of project types
*TagsApi* | [**report_type_list**](docs/TagsApi.md#report_type_list) | **GET** /tag/report_type | Get a list of report types
*TagsApi* | [**side_type_list**](docs/TagsApi.md#side_type_list) | **GET** /tag/side_type | Get a list of side types
*TagsApi* | [**version_list**](docs/TagsApi.md#version_list) | **GET** /tag/game_version | Get a list of game versions
*TeamsApi* | [**add_team_member**](docs/TeamsApi.md#add_team_member) | **POST** /team/{id}/members | Add a user to a team
*TeamsApi* | [**delete_team_member**](docs/TeamsApi.md#delete_team_member) | **DELETE** /team/{id}/members/{id|username} | Remove a member from a team
*TeamsApi* | [**get_project_team_members**](docs/TeamsApi.md#get_project_team_members) | **GET** /project/{id|slug}/members | Get a project's team members
*TeamsApi* | [**get_team_members**](docs/TeamsApi.md#get_team_members) | **GET** /team/{id}/members | Get a team's members
*TeamsApi* | [**get_teams**](docs/TeamsApi.md#get_teams) | **GET** /teams | Get the members of multiple teams
*TeamsApi* | [**join_team**](docs/TeamsApi.md#join_team) | **POST** /team/{id}/join | Join a team
*TeamsApi* | [**modify_team_member**](docs/TeamsApi.md#modify_team_member) | **PATCH** /team/{id}/members/{id|username} | Modify a team member's information
*TeamsApi* | [**transfer_team_ownership**](docs/TeamsApi.md#transfer_team_ownership) | **PATCH** /team/{id}/owner | Transfer team's ownership to another user
*ThreadsApi* | [**delete_thread_message**](docs/ThreadsApi.md#delete_thread_message) | **DELETE** /message/{id} | Delete a thread message
*ThreadsApi* | [**get_open_reports**](docs/ThreadsApi.md#get_open_reports) | **GET** /report | Get your open reports
*ThreadsApi* | [**get_report**](docs/ThreadsApi.md#get_report) | **GET** /report/{id} | Get report from ID
*ThreadsApi* | [**get_reports**](docs/ThreadsApi.md#get_reports) | **GET** /reports | Get multiple reports
*ThreadsApi* | [**get_thread**](docs/ThreadsApi.md#get_thread) | **GET** /thread/{id} | Get a thread
*ThreadsApi* | [**get_threads**](docs/ThreadsApi.md#get_threads) | **GET** /threads | Get multiple threads
*ThreadsApi* | [**modify_report**](docs/ThreadsApi.md#modify_report) | **PATCH** /report/{id} | Modify a report
*ThreadsApi* | [**send_thread_message**](docs/ThreadsApi.md#send_thread_message) | **POST** /thread/{id} | Send a text message to a thread
*ThreadsApi* | [**submit_report**](docs/ThreadsApi.md#submit_report) | **POST** /report | Report a project, user, or version
*UsersApi* | [**change_user_icon**](docs/UsersApi.md#change_user_icon) | **PATCH** /user/{id|username}/icon | Change user's avatar
*UsersApi* | [**delete_user_icon**](docs/UsersApi.md#delete_user_icon) | **DELETE** /user/{id|username}/icon | Remove user's avatar
*UsersApi* | [**get_followed_projects**](docs/UsersApi.md#get_followed_projects) | **GET** /user/{id|username}/follows | Get user's followed projects
*UsersApi* | [**get_payout_history**](docs/UsersApi.md#get_payout_history) | **GET** /user/{id|username}/payouts | Get user's payout history
*UsersApi* | [**get_user**](docs/UsersApi.md#get_user) | **GET** /user/{id|username} | Get a user
*UsersApi* | [**get_user_from_auth**](docs/UsersApi.md#get_user_from_auth) | **GET** /user | Get user from authorization header
*UsersApi* | [**get_user_projects**](docs/UsersApi.md#get_user_projects) | **GET** /user/{id|username}/projects | Get user's projects
*UsersApi* | [**get_users**](docs/UsersApi.md#get_users) | **GET** /users | Get multiple users
*UsersApi* | [**modify_user**](docs/UsersApi.md#modify_user) | **PATCH** /user/{id|username} | Modify a user
*UsersApi* | [**withdraw_payout**](docs/UsersApi.md#withdraw_payout) | **POST** /user/{id|username}/payouts | Withdraw payout balance to PayPal or Venmo
*VersionFilesApi* | [**delete_file_from_hash**](docs/VersionFilesApi.md#delete_file_from_hash) | **DELETE** /version_file/{hash} | Delete a file from its hash
*VersionFilesApi* | [**get_latest_version_from_hash**](docs/VersionFilesApi.md#get_latest_version_from_hash) | **POST** /version_file/{hash}/update | Latest version of a project from a hash, loader(s), and game version(s)
*VersionFilesApi* | [**get_latest_versions_from_hashes**](docs/VersionFilesApi.md#get_latest_versions_from_hashes) | **POST** /version_files/update | Latest versions of multiple project from hashes, loader(s), and game version(s)
*VersionFilesApi* | [**version_from_hash**](docs/VersionFilesApi.md#version_from_hash) | **GET** /version_file/{hash} | Get version from hash
*VersionFilesApi* | [**versions_from_hashes**](docs/VersionFilesApi.md#versions_from_hashes) | **POST** /version_files | Get versions from hashes
*VersionsApi* | [**add_files_to_version**](docs/VersionsApi.md#add_files_to_version) | **POST** /version/{id}/file | Add files to version
*VersionsApi* | [**create_version**](docs/VersionsApi.md#create_version) | **POST** /version | Create a version
*VersionsApi* | [**delete_version**](docs/VersionsApi.md#delete_version) | **DELETE** /version/{id} | Delete a version
*VersionsApi* | [**get_project_versions**](docs/VersionsApi.md#get_project_versions) | **GET** /project/{id|slug}/version | List project's versions
*VersionsApi* | [**get_version**](docs/VersionsApi.md#get_version) | **GET** /version/{id} | Get a version
*VersionsApi* | [**get_version_from_id_or_number**](docs/VersionsApi.md#get_version_from_id_or_number) | **GET** /project/{id|slug}/version/{id|number} | Get a version given a version number or ID
*VersionsApi* | [**get_versions**](docs/VersionsApi.md#get_versions) | **GET** /versions | Get multiple versions
*VersionsApi* | [**modify_version**](docs/VersionsApi.md#modify_version) | **PATCH** /version/{id} | Modify a version
*VersionsApi* | [**schedule_version**](docs/VersionsApi.md#schedule_version) | **POST** /version/{id}/schedule | Schedule a version


## Documentation For Models

 - [AuthError](docs/AuthError.md)
 - [BaseProject](docs/BaseProject.md)
 - [BaseVersion](docs/BaseVersion.md)
 - [CategoryTag](docs/CategoryTag.md)
 - [CreatableProject](docs/CreatableProject.md)
 - [CreatableProjectGalleryItem](docs/CreatableProjectGalleryItem.md)
 - [CreatableReport](docs/CreatableReport.md)
 - [CreatableVersion](docs/CreatableVersion.md)
 - [DonationPlatformTag](docs/DonationPlatformTag.md)
 - [EditableFileType](docs/EditableFileType.md)
 - [EditableProject](docs/EditableProject.md)
 - [EditableUser](docs/EditableUser.md)
 - [EditableVersion](docs/EditableVersion.md)
 - [FileTypeEnum](docs/FileTypeEnum.md)
 - [ForgeUpdateCheckerPromos](docs/ForgeUpdateCheckerPromos.md)
 - [ForgeUpdates](docs/ForgeUpdates.md)
 - [GalleryImage](docs/GalleryImage.md)
 - [GameVersionTag](docs/GameVersionTag.md)
 - [GetLatestVersionFromHashBody](docs/GetLatestVersionFromHashBody.md)
 - [GetLatestVersionsFromHashesBody](docs/GetLatestVersionsFromHashesBody.md)
 - [HashList](docs/HashList.md)
 - [InvalidInputError](docs/InvalidInputError.md)
 - [License](docs/License.md)
 - [LicenseTag](docs/LicenseTag.md)
 - [LoaderTag](docs/LoaderTag.md)
 - [ModeratorMessage](docs/ModeratorMessage.md)
 - [ModifiableProject](docs/ModifiableProject.md)
 - [ModifyReportRequest](docs/ModifyReportRequest.md)
 - [ModifyTeamMemberBody](docs/ModifyTeamMemberBody.md)
 - [NonSearchProject](docs/NonSearchProject.md)
 - [Notification](docs/Notification.md)
 - [NotificationAction](docs/NotificationAction.md)
 - [PatchProjectsBody](docs/PatchProjectsBody.md)
 - [Project](docs/Project.md)
 - [ProjectDependencyList](docs/ProjectDependencyList.md)
 - [ProjectDonationUrl](docs/ProjectDonationUrl.md)
 - [ProjectIdentifier](docs/ProjectIdentifier.md)
 - [ProjectLicense](docs/ProjectLicense.md)
 - [ProjectResult](docs/ProjectResult.md)
 - [Report](docs/Report.md)
 - [Schedule](docs/Schedule.md)
 - [SearchResults](docs/SearchResults.md)
 - [ServerRenderedProject](docs/ServerRenderedProject.md)
 - [Statistics](docs/Statistics.md)
 - [TeamMember](docs/TeamMember.md)
 - [Thread](docs/Thread.md)
 - [ThreadMessage](docs/ThreadMessage.md)
 - [ThreadMessageBody](docs/ThreadMessageBody.md)
 - [User](docs/User.md)
 - [UserIdentifier](docs/UserIdentifier.md)
 - [UserPayoutData](docs/UserPayoutData.md)
 - [UserPayoutHistory](docs/UserPayoutHistory.md)
 - [UserPayoutHistoryEntry](docs/UserPayoutHistoryEntry.md)
 - [Version](docs/Version.md)
 - [VersionDependency](docs/VersionDependency.md)
 - [VersionFile](docs/VersionFile.md)
 - [VersionFileHashes](docs/VersionFileHashes.md)


To get access to the crate's generated documentation, use:

```
cargo doc --open
```

## Author

support@modrinth.com

