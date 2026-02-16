/*
 * Labrinth
 *
 * This documentation doesn't provide a way to test our API. In order to facilitate testing, we recommend the following tools:  - [cURL](https://curl.se/) (recommended, command-line) - [ReqBIN](https://reqbin.com/) (recommended, online) - [Postman](https://www.postman.com/downloads/) - [Insomnia](https://insomnia.rest/) - Your web browser, if you don't need to send headers or a request body  Once you have a working client, you can test that it works by making a `GET` request to `https://staging-api.modrinth.com/`:  ```json {   \"about\": \"Welcome traveler!\",   \"documentation\": \"https://docs.modrinth.com\",   \"name\": \"modrinth-labrinth\",   \"version\": \"2.7.0\" } ```  If you got a response similar to the one above, you can use the Modrinth API! When you want to go live using the production API, use `api.modrinth.com` instead of `staging-api.modrinth.com`.  ## Authentication This API has two options for authentication: personal access tokens and [OAuth2](https://en.wikipedia.org/wiki/OAuth). All tokens are tied to a Modrinth user and use the `Authorization` header of the request.  Example: ``` Authorization: mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic ```  You do not need a token for most requests. Generally speaking, only the following types of requests require a token: - those which create data (such as version creation) - those which modify data (such as editing a project) - those which access private data (such as draft projects, notifications, emails, and payout data)  Each request requiring authentication has a certain scope. For example, to view the email of the user being requested, the token must have the `USER_READ_EMAIL` scope. You can find the list of available scopes [on GitHub](https://github.com/modrinth/labrinth/blob/master/src/models/pats.rs#L15). Making a request with an invalid scope will return a 401 error.  Please note that certain scopes and requests cannot be completed with a personal access token or using OAuth. For example, deleting a user account can only be done through Modrinth's frontend.  A detailed guide on OAuth has been published in [Modrinth's technical documentation](https://docs.modrinth.com/guide/oauth).  ### Personal access tokens Personal access tokens (PATs) can be generated in from [the user settings](https://modrinth.com/settings/account).  ### GitHub tokens For backwards compatibility purposes, some types of GitHub tokens also work for authenticating a user with Modrinth's API, granting all scopes. **We urge any application still using GitHub tokens to start using personal access tokens for security and reliability purposes.** GitHub tokens will cease to function to authenticate with Modrinth's API as soon as version 3 of the API is made generally available.  ## Cross-Origin Resource Sharing This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with the [W3C spec](https://www.w3.org/TR/cors/). This allows for cross-domain communication from the browser. All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site.  ## Identifiers The majority of items you can interact with in the API have a unique eight-digit base62 ID. Projects, versions, users, threads, teams, and reports all use this same way of identifying themselves. Version files use the sha1 or sha512 file hashes as identifiers.  Each project and user has a friendlier way of identifying them; slugs and usernames, respectively. While unique IDs are constant, slugs and usernames can change at any moment. If you want to store something in the long term, it is recommended to use the unique ID.  ## Ratelimits The API has a ratelimit defined per IP. Limits and remaining amounts are given in the response headers. - `X-Ratelimit-Limit`: the maximum number of requests that can be made in a minute - `X-Ratelimit-Remaining`: the number of requests remaining in the current ratelimit window - `X-Ratelimit-Reset`: the time in seconds until the ratelimit window resets  Ratelimits are the same no matter whether you use a token or not. The ratelimit is currently 300 requests per minute. If you have a use case requiring a higher limit, please [contact us](mailto:admin@modrinth.com).  ## User Agents To access the Modrinth API, you **must** use provide a uniquely-identifying `User-Agent` header. Providing a user agent that only identifies your HTTP client library (such as \"okhttp/4.9.3\") increases the likelihood that we will block your traffic. It is recommended, but not required, to include contact information in your user agent. This allows us to contact you if we would like a change in your application's behavior without having to block your traffic. - Bad: `User-Agent: okhttp/4.9.3` - Good: `User-Agent: project_name` - Better: `User-Agent: github_username/project_name/1.56.0` - Best: `User-Agent: github_username/project_name/1.56.0 (launcher.com)` or `User-Agent: github_username/project_name/1.56.0 (contact@launcher.com)`  ## Versioning Modrinth follows a simple pattern for its API versioning. In the event of a breaking API change, the API version in the URL path is bumped, and migration steps will be published below.  When an API is no longer the current one, it will immediately be considered deprecated. No more support will be provided for API versions older than the current one. It will be kept for some time, but this amount of time is not certain.  We will exercise various tactics to get people to update their implementation of our API. One example is by adding something like `STOP USING THIS API` to various data returned by the API.  Once an API version is completely deprecated, it will permanently return a 410 error. Please ensure your application handles these 410 errors.  ### Migrations Inside the following spoiler, you will be able to find all changes between versions of the Modrinth API, accompanied by tips and a guide to migrate applications to newer versions.  Here, you can also find changes for [Minotaur](https://github.com/modrinth/minotaur), Modrinth's official Gradle plugin. Major versions of Minotaur directly correspond to major versions of the Modrinth API.  <details><summary>API v1 to API v2</summary>  These bullet points cover most changes in the v2 API, but please note that fields containing `mod` in most contexts have been shifted to `project`.  For example, in the search route, the field `mod_id` was renamed to `project_id`.  - The search route has been moved from `/api/v1/mod` to `/v2/search` - New project fields: `project_type` (may be `mod` or `modpack`), `moderation_message` (which has a `message` and `body`), `gallery` - New search facet: `project_type` - Alphabetical sort removed (it didn't work and is not possible due to limits in MeiliSearch) - New search fields: `project_type`, `gallery`   - The gallery field is an array of URLs to images that are part of the project's gallery - The gallery is a new feature which allows the user to upload images showcasing their mod to the CDN which will be displayed on their mod page - Internal change: Any project file uploaded to Modrinth is now validated to make sure it's a valid Minecraft mod, Modpack, etc.   - For example, a Forge 1.17 mod with a JAR not containing a mods.toml will not be allowed to be uploaded to Modrinth - In project creation, projects may not upload a mod with no versions to review, however they can be saved as a draft   - Similarly, for version creation, a version may not be uploaded without any files - Donation URLs have been enabled - New project status: `archived`. Projects with this status do not appear in search - Tags (such as categories, loaders) now have icons (SVGs) and specific project types attached - Dependencies have been wiped and replaced with a new system - Notifications now have a `type` field, such as `project_update`  Along with this, project subroutes (such as `/v2/project/{id}/version`) now allow the slug to be used as the ID. This is also the case with user routes.  </details><details><summary>Minotaur v1 to Minotaur v2</summary>  Minotaur 2.x introduced a few breaking changes to how your buildscript is formatted.  First, instead of registering your own `publishModrinth` task, Minotaur now automatically creates a `modrinth` task. As such, you can replace the `task publishModrinth(type: TaskModrinthUpload) {` line with just `modrinth {`.  To declare supported Minecraft versions and mod loaders, the `gameVersions` and `loaders` arrays must now be used. The syntax for these are pretty self-explanatory.  Instead of using `releaseType`, you must now use `versionType`. This was actually changed in v1.2.0, but very few buildscripts have moved on from v1.1.0.  Dependencies have been changed to a special DSL. Create a `dependencies` block within the `modrinth` block, and then use `scope.type(\"project/version\")`. For example, `required.project(\"fabric-api\")` adds a required project dependency on Fabric API.  You may now use the slug anywhere that a project ID was previously required.  </details> 
 *
 * The version of the OpenAPI document: v2.7.0/366f528
 * Contact: support@modrinth.com
 * Generated by: https://openapi-generator.tech
 */


use reqwest;
use serde::{Deserialize, Serialize, de::Error as _};
use crate::{apis::ResponseContent, models};
use super::{Error, configuration, ContentType};
use tokio::fs::File as TokioFile;
use tokio_util::codec::{BytesCodec, FramedRead};


/// struct for typed errors of method [`add_gallery_image`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddGalleryImageError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`change_project_icon`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChangeProjectIconError {
    Status400(models::InvalidInputError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`check_project_validity`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CheckProjectValidityError {
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`create_project`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateProjectError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_gallery_image`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteGalleryImageError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_project`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteProjectError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_project_icon`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteProjectIconError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`follow_project`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FollowProjectError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_dependencies`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetDependenciesError {
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_project`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetProjectError {
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_projects`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetProjectsError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`modify_gallery_image`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModifyGalleryImageError {
    Status401(models::AuthError),
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`modify_project`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModifyProjectError {
    Status401(models::AuthError),
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`patch_projects`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PatchProjectsError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`random_projects`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RandomProjectsError {
    Status400(models::InvalidInputError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`schedule_project`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScheduleProjectError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`search_projects`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchProjectsError {
    Status400(models::InvalidInputError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`unfollow_project`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnfollowProjectError {
    Status400(models::InvalidInputError),
    Status401(models::AuthError),
    UnknownValue(serde_json::Value),
}


/// Modrinth allows you to upload files of up to 5MiB to a project's gallery.
pub async fn add_gallery_image(configuration: &configuration::Configuration, id_pipe_slug: &str, ext: &str, featured: bool, title: Option<&str>, description: Option<&str>, ordering: Option<i32>, body: Option<std::path::PathBuf>) -> Result<(), Error<AddGalleryImageError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;
    let p_query_ext = ext;
    let p_query_featured = featured;
    let p_query_title = title;
    let p_query_description = description;
    let p_query_ordering = ordering;
    let p_body_body = body;

    let uri_str = format!("{}/project/{id_slug}/gallery", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    req_builder = req_builder.query(&[("ext", &p_query_ext.to_string())]);
    req_builder = req_builder.query(&[("featured", &p_query_featured.to_string())]);
    if let Some(ref param_value) = p_query_title {
        req_builder = req_builder.query(&[("title", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_description {
        req_builder = req_builder.query(&[("description", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_ordering {
        req_builder = req_builder.query(&[("ordering", &param_value.to_string())]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };
    if let Some(param_value) = p_body_body {
        let file = TokioFile::open(param_value).await?;
        let stream = FramedRead::new(file, BytesCodec::new());
        req_builder = req_builder.body(reqwest::Body::wrap_stream(stream));
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<AddGalleryImageError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// The new icon may be up to 256KiB in size.
pub async fn change_project_icon(configuration: &configuration::Configuration, id_pipe_slug: &str, ext: &str, body: Option<std::path::PathBuf>) -> Result<(), Error<ChangeProjectIconError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;
    let p_query_ext = ext;
    let p_body_body = body;

    let uri_str = format!("{}/project/{id_slug}/icon", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::PATCH, &uri_str);

    req_builder = req_builder.query(&[("ext", &p_query_ext.to_string())]);
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };
    if let Some(param_value) = p_body_body {
        let file = TokioFile::open(param_value).await?;
        let stream = FramedRead::new(file, BytesCodec::new());
        req_builder = req_builder.body(reqwest::Body::wrap_stream(stream));
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<ChangeProjectIconError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn check_project_validity(configuration: &configuration::Configuration, id_pipe_slug: &str) -> Result<models::ProjectIdentifier, Error<CheckProjectValidityError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;

    let uri_str = format!("{}/project/{id_slug}/check", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::ProjectIdentifier`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::ProjectIdentifier`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<CheckProjectValidityError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn create_project(configuration: &configuration::Configuration, data: models::CreatableProject, icon: Option<std::path::PathBuf>) -> Result<models::Project, Error<CreateProjectError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_form_data = data;
    let p_form_icon = icon;

    let uri_str = format!("{}/project", configuration.base_path);
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };
    let mut multipart_form = reqwest::multipart::Form::new();
    multipart_form = multipart_form.text("data", serde_json::to_string(&p_form_data)?);
    if let Some(ref param_value) = p_form_icon {
                let file = TokioFile::open(param_value).await?;
                let stream = FramedRead::new(file, BytesCodec::new());
                let file_name = param_value.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let file_part = reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(stream)).file_name(file_name);
                multipart_form = multipart_form.part("icon", file_part);
    }
    req_builder = req_builder.multipart(multipart_form);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::Project`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::Project`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<CreateProjectError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn delete_gallery_image(configuration: &configuration::Configuration, id_pipe_slug: &str, url: &str) -> Result<(), Error<DeleteGalleryImageError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;
    let p_query_url = url;

    let uri_str = format!("{}/project/{id_slug}/gallery", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::DELETE, &uri_str);

    req_builder = req_builder.query(&[("url", &p_query_url.to_string())]);
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<DeleteGalleryImageError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn delete_project(configuration: &configuration::Configuration, id_pipe_slug: &str) -> Result<(), Error<DeleteProjectError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;

    let uri_str = format!("{}/project/{id_slug}", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::DELETE, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<DeleteProjectError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn delete_project_icon(configuration: &configuration::Configuration, id_pipe_slug: &str) -> Result<(), Error<DeleteProjectIconError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;

    let uri_str = format!("{}/project/{id_slug}/icon", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::DELETE, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<DeleteProjectIconError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn follow_project(configuration: &configuration::Configuration, id_pipe_slug: &str) -> Result<(), Error<FollowProjectError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;

    let uri_str = format!("{}/project/{id_slug}/follow", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<FollowProjectError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn get_dependencies(configuration: &configuration::Configuration, id_pipe_slug: &str) -> Result<models::ProjectDependencyList, Error<GetDependenciesError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;

    let uri_str = format!("{}/project/{id_slug}/dependencies", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::ProjectDependencyList`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::ProjectDependencyList`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetDependenciesError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn get_project(configuration: &configuration::Configuration, id_pipe_slug: &str) -> Result<models::Project, Error<GetProjectError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;

    let uri_str = format!("{}/project/{id_slug}", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::Project`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::Project`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetProjectError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn get_projects(configuration: &configuration::Configuration, ids: &str) -> Result<Vec<models::Project>, Error<GetProjectsError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_query_ids = ids;

    let uri_str = format!("{}/projects", configuration.base_path);
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    req_builder = req_builder.query(&[("ids", &p_query_ids.to_string())]);
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `Vec&lt;models::Project&gt;`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `Vec&lt;models::Project&gt;`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetProjectsError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn modify_gallery_image(configuration: &configuration::Configuration, id_pipe_slug: &str, url: &str, featured: Option<bool>, title: Option<&str>, description: Option<&str>, ordering: Option<i32>) -> Result<(), Error<ModifyGalleryImageError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;
    let p_query_url = url;
    let p_query_featured = featured;
    let p_query_title = title;
    let p_query_description = description;
    let p_query_ordering = ordering;

    let uri_str = format!("{}/project/{id_slug}/gallery", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::PATCH, &uri_str);

    req_builder = req_builder.query(&[("url", &p_query_url.to_string())]);
    if let Some(ref param_value) = p_query_featured {
        req_builder = req_builder.query(&[("featured", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_title {
        req_builder = req_builder.query(&[("title", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_description {
        req_builder = req_builder.query(&[("description", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_ordering {
        req_builder = req_builder.query(&[("ordering", &param_value.to_string())]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<ModifyGalleryImageError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn modify_project(configuration: &configuration::Configuration, id_pipe_slug: &str, editable_project: Option<models::EditableProject>) -> Result<(), Error<ModifyProjectError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;
    let p_body_editable_project = editable_project;

    let uri_str = format!("{}/project/{id_slug}", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::PATCH, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };
    req_builder = req_builder.json(&p_body_editable_project);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<ModifyProjectError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn patch_projects(configuration: &configuration::Configuration, ids: &str, patch_projects_body: Option<models::PatchProjectsBody>) -> Result<(), Error<PatchProjectsError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_query_ids = ids;
    let p_body_patch_projects_body = patch_projects_body;

    let uri_str = format!("{}/projects", configuration.base_path);
    let mut req_builder = configuration.client.request(reqwest::Method::PATCH, &uri_str);

    req_builder = req_builder.query(&[("ids", &p_query_ids.to_string())]);
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };
    req_builder = req_builder.json(&p_body_patch_projects_body);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<PatchProjectsError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn random_projects(configuration: &configuration::Configuration, count: i32) -> Result<Vec<models::Project>, Error<RandomProjectsError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_query_count = count;

    let uri_str = format!("{}/projects_random", configuration.base_path);
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    req_builder = req_builder.query(&[("count", &p_query_count.to_string())]);
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `Vec&lt;models::Project&gt;`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `Vec&lt;models::Project&gt;`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<RandomProjectsError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn schedule_project(configuration: &configuration::Configuration, id_pipe_slug: &str, schedule: Option<models::Schedule>) -> Result<(), Error<ScheduleProjectError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;
    let p_body_schedule = schedule;

    let uri_str = format!("{}/project/{id_slug}/schedule", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };
    req_builder = req_builder.json(&p_body_schedule);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<ScheduleProjectError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn search_projects(configuration: &configuration::Configuration, query: Option<&str>, facets: Option<&str>, index: Option<&str>, offset: Option<i32>, limit: Option<i32>) -> Result<models::SearchResults, Error<SearchProjectsError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_query_query = query;
    let p_query_facets = facets;
    let p_query_index = index;
    let p_query_offset = offset;
    let p_query_limit = limit;

    let uri_str = format!("{}/search", configuration.base_path);
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref param_value) = p_query_query {
        req_builder = req_builder.query(&[("query", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_facets {
        req_builder = req_builder.query(&[("facets", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_index {
        req_builder = req_builder.query(&[("index", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_offset {
        req_builder = req_builder.query(&[("offset", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_limit {
        req_builder = req_builder.query(&[("limit", &param_value.to_string())]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::SearchResults`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::SearchResults`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<SearchProjectsError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn unfollow_project(configuration: &configuration::Configuration, id_pipe_slug: &str) -> Result<(), Error<UnfollowProjectError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_id_pipe_slug = id_pipe_slug;

    let uri_str = format!("{}/project/{id_slug}/follow", configuration.base_path, id_slug=crate::apis::urlencode(p_path_id_pipe_slug));
    let mut req_builder = configuration.client.request(reqwest::Method::DELETE, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.header("Authorization", value);
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<UnfollowProjectError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

