# \MediaAdminApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_security_image_info**](MediaAdminApi.md#get_security_image_info) | **GET** /media_api/security_image_info/{account_id} | Get current security image for selected profile. Only for admins.
[**patch_moderation_request_list**](MediaAdminApi.md#patch_moderation_request_list) | **PATCH** /media_api/admin/moderation/page/next | Get current list of moderation requests in my moderation queue.
[**post_handle_moderation_request**](MediaAdminApi.md#post_handle_moderation_request) | **POST** /media_api/admin/moderation/handle_request/{account_id} | Handle moderation request of some account.



## get_security_image_info

> crate::models::SecurityImage get_security_image_info(account_id)
Get current security image for selected profile. Only for admins.

Get current security image for selected profile. Only for admins.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::SecurityImage**](SecurityImage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## patch_moderation_request_list

> crate::models::ModerationList patch_moderation_request_list()
Get current list of moderation requests in my moderation queue.

Get current list of moderation requests in my moderation queue. Additional requests will be added to my queue if necessary.  ## Access  Account with `admin_moderate_images` capability is required to access this route. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ModerationList**](ModerationList.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_handle_moderation_request

> post_handle_moderation_request(account_id, handle_moderation_request)
Handle moderation request of some account.

Handle moderation request of some account.  ## Access  Account with `admin_moderate_images` capability is required to access this route. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**handle_moderation_request** | [**HandleModerationRequest**](HandleModerationRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
