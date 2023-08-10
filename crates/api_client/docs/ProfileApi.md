# \ProfileApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_profile**](ProfileApi.md#get_profile) | **GET** /profile_api/profile/{account_id} | Get account's current profile.
[**get_profile_from_database_debug_mode_benchmark**](ProfileApi.md#get_profile_from_database_debug_mode_benchmark) | **GET** /profile_api/benchmark/profile/{account_id} | Get account's current profile from database. Debug mode must be enabled
[**post_get_next_profile_page**](ProfileApi.md#post_get_next_profile_page) | **POST** /profile_api/page/next | Post (updates iterator) to get next page of profile list.
[**post_profile**](ProfileApi.md#post_profile) | **POST** /profile_api/profile | Update profile information.
[**post_profile_to_database_debug_mode_benchmark**](ProfileApi.md#post_profile_to_database_debug_mode_benchmark) | **POST** /profile_api/benchmark/profile | Post account's current profile directly to database. Debug mode must be enabled
[**post_reset_profile_paging**](ProfileApi.md#post_reset_profile_paging) | **POST** /profile_api/page/reset | Reset profile paging.
[**put_location**](ProfileApi.md#put_location) | **PUT** /profile_api/location | Update location



## get_profile

> crate::models::Profile get_profile(account_id)
Get account's current profile.

Get account's current profile.  Profile can include version UUID which can be used for caching.  # Access Public profile access requires `view_public_profiles` capability. Public and private profile access requires `admin_view_all_profiles` capablility.  # Microservice notes If account feature is set as external service then cached capability information from account service is used for access checks.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::Profile**](Profile.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile_from_database_debug_mode_benchmark

> crate::models::Profile get_profile_from_database_debug_mode_benchmark(account_id)
Get account's current profile from database. Debug mode must be enabled

Get account's current profile from database. Debug mode must be enabled that route can be used.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::Profile**](Profile.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_get_next_profile_page

> crate::models::ProfilePage post_get_next_profile_page()
Post (updates iterator) to get next page of profile list.

Post (updates iterator) to get next page of profile list.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ProfilePage**](ProfilePage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_profile

> post_profile(profile_update)
Update profile information.

Update profile information.  Writes the profile to the database only if it is changed.  TODO: string lenght validation, limit saving new profiles

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**profile_update** | [**ProfileUpdate**](ProfileUpdate.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_profile_to_database_debug_mode_benchmark

> post_profile_to_database_debug_mode_benchmark(profile_update)
Post account's current profile directly to database. Debug mode must be enabled

Post account's current profile directly to database. Debug mode must be enabled that route can be used.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**profile_update** | [**ProfileUpdate**](ProfileUpdate.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_reset_profile_paging

> post_reset_profile_paging()
Reset profile paging.

Reset profile paging.  After this request getting next profiles will continue from the nearest profiles.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_location

> put_location(location)
Update location

Update location

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**location** | [**Location**](Location.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
