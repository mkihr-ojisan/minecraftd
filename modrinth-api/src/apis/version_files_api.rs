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


/// struct for typed errors of method [`delete_file_from_hash`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteFileFromHashError {
    Status401(models::AuthError),
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_latest_version_from_hash`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetLatestVersionFromHashError {
    Status400(),
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_latest_versions_from_hashes`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetLatestVersionsFromHashesError {
    Status400(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`version_from_hash`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VersionFromHashError {
    Status404(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`versions_from_hashes`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VersionsFromHashesError {
    Status400(),
    UnknownValue(serde_json::Value),
}


pub async fn delete_file_from_hash(configuration: &configuration::Configuration, hash: &str, algorithm: &str, version_id: Option<&str>) -> Result<(), Error<DeleteFileFromHashError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_hash = hash;
    let p_query_algorithm = algorithm;
    let p_query_version_id = version_id;

    let uri_str = format!("{}/version_file/{hash}", configuration.base_path, hash=crate::apis::urlencode(p_path_hash));
    let mut req_builder = configuration.client.request(reqwest::Method::DELETE, &uri_str);

    req_builder = req_builder.query(&[("algorithm", &p_query_algorithm.to_string())]);
    if let Some(ref param_value) = p_query_version_id {
        req_builder = req_builder.query(&[("version_id", &param_value.to_string())]);
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
        let entity: Option<DeleteFileFromHashError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn get_latest_version_from_hash(configuration: &configuration::Configuration, hash: &str, algorithm: &str, get_latest_version_from_hash_body: Option<models::GetLatestVersionFromHashBody>) -> Result<models::Version, Error<GetLatestVersionFromHashError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_hash = hash;
    let p_query_algorithm = algorithm;
    let p_body_get_latest_version_from_hash_body = get_latest_version_from_hash_body;

    let uri_str = format!("{}/version_file/{hash}/update", configuration.base_path, hash=crate::apis::urlencode(p_path_hash));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    req_builder = req_builder.query(&[("algorithm", &p_query_algorithm.to_string())]);
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    req_builder = req_builder.json(&p_body_get_latest_version_from_hash_body);

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
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::Version`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::Version`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetLatestVersionFromHashError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// This is the same as [`/version_file/{hash}/update`](#operation/getLatestVersionFromHash) except it accepts multiple hashes.
pub async fn get_latest_versions_from_hashes(configuration: &configuration::Configuration, get_latest_versions_from_hashes_body: Option<models::GetLatestVersionsFromHashesBody>) -> Result<std::collections::HashMap<String, models::Version>, Error<GetLatestVersionsFromHashesError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_body_get_latest_versions_from_hashes_body = get_latest_versions_from_hashes_body;

    let uri_str = format!("{}/version_files/update", configuration.base_path);
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    req_builder = req_builder.json(&p_body_get_latest_versions_from_hashes_body);

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
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `std::collections::HashMap&lt;String, models::Version&gt;`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `std::collections::HashMap&lt;String, models::Version&gt;`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetLatestVersionsFromHashesError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

pub async fn version_from_hash(configuration: &configuration::Configuration, hash: &str, algorithm: &str, multiple: Option<bool>) -> Result<models::Version, Error<VersionFromHashError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_hash = hash;
    let p_query_algorithm = algorithm;
    let p_query_multiple = multiple;

    let uri_str = format!("{}/version_file/{hash}", configuration.base_path, hash=crate::apis::urlencode(p_path_hash));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    req_builder = req_builder.query(&[("algorithm", &p_query_algorithm.to_string())]);
    if let Some(ref param_value) = p_query_multiple {
        req_builder = req_builder.query(&[("multiple", &param_value.to_string())]);
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
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::Version`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::Version`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<VersionFromHashError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// This is the same as [`/version_file/{hash}`](#operation/versionFromHash) except it accepts multiple hashes.
pub async fn versions_from_hashes(configuration: &configuration::Configuration, hash_list: Option<models::HashList>) -> Result<std::collections::HashMap<String, models::Version>, Error<VersionsFromHashesError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_body_hash_list = hash_list;

    let uri_str = format!("{}/version_files", configuration.base_path);
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    req_builder = req_builder.json(&p_body_hash_list);

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
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `std::collections::HashMap&lt;String, models::Version&gt;`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `std::collections::HashMap&lt;String, models::Version&gt;`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<VersionsFromHashesError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

