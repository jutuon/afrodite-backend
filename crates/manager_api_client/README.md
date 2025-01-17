# Rust API client for manager_api_client

Afrodite manager API


## Overview

This API client was generated by the [OpenAPI Generator](https://openapi-generator.tech) project.  By using the [openapi-spec](https://openapis.org) from a remote server, you can easily generate an API client.

- API version: 0.1.0
- Package version: 0.1.0
- Generator version: 7.10.0
- Build package: `org.openapitools.codegen.languages.RustClientCodegen`

## Installation

Put the package under your project folder in a directory named `manager_api_client` and add the following to `Cargo.toml` under `[dependencies]`:

```
manager_api_client = { path = "./manager_api_client" }
```

## Documentation for API Endpoints

All URIs are relative to *http://localhost*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*ManagerApi* | [**get_encryption_key**](docs/ManagerApi.md#get_encryption_key) | **GET** /manager_api/encryption_key/{server} | Get encryption key for some server
*ManagerApi* | [**get_latest_software**](docs/ManagerApi.md#get_latest_software) | **GET** /manager_api/latest_software | Download latest software.
*ManagerApi* | [**get_software_info**](docs/ManagerApi.md#get_software_info) | **GET** /manager_api/software_info | Get current software info about currently installed backend and manager.
*ManagerApi* | [**get_system_info**](docs/ManagerApi.md#get_system_info) | **GET** /manager_api/system_info | Get system info about current operating system, hardware and software.
*ManagerApi* | [**get_system_info_all**](docs/ManagerApi.md#get_system_info_all) | **GET** /manager_api/system_info_all | Get system info about current operating system, hardware and software.
*ManagerApi* | [**post_request_restart_or_reset_backend**](docs/ManagerApi.md#post_request_restart_or_reset_backend) | **POST** /manager_api/request_restart_or_reset_backend | Restart or reset backend.
*ManagerApi* | [**post_request_software_update**](docs/ManagerApi.md#post_request_software_update) | **POST** /manager_api/request_software_update | Request software update.


## Documentation For Models

 - [BuildInfo](docs/BuildInfo.md)
 - [CommandOutput](docs/CommandOutput.md)
 - [DataEncryptionKey](docs/DataEncryptionKey.md)
 - [DownloadType](docs/DownloadType.md)
 - [DownloadTypeQueryParam](docs/DownloadTypeQueryParam.md)
 - [RebootQueryParam](docs/RebootQueryParam.md)
 - [ResetDataQueryParam](docs/ResetDataQueryParam.md)
 - [ServerNameText](docs/ServerNameText.md)
 - [SoftwareInfo](docs/SoftwareInfo.md)
 - [SoftwareOptions](docs/SoftwareOptions.md)
 - [SoftwareOptionsQueryParam](docs/SoftwareOptionsQueryParam.md)
 - [SystemInfo](docs/SystemInfo.md)
 - [SystemInfoList](docs/SystemInfoList.md)


To get access to the crate's generated documentation, use:

```
cargo doc --open
```

## Author



