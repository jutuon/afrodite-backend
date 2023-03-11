# API usage

Minimum viable product API

## Account creation

App opens to login screen with Google and Apple single sign on buttons.

`/account/register` is used for now and it just creates a new account.

## Initial setup

The created account is in `initial setup` state. The client will ask
user all questions and fill in user details.

All textual data will be sent with `/account/setup` that path will only be
used when account state is in initial setup.

Client initial setup will create new image moderation request with two images
using path `/media/moderation/request`.

The client initial setup will then request state transfer to `normal` state
using path `/account/complete_initial_setup`. Account server will check that all
required information is set to the account and then also check is there really
an moderation request created using internal media server API.

TODO: Remove capablity 'admin_setup_possible' from another document.

## Normal state

Client now gets the account state again using `/account/state` and updates the
client UI state accordingly.

After initial setup the client will go to the profile grid view.

Client queries about one time events like rejected image moderation requests are
handled using `/media/events`

### Profile grid view

Client will use previously get account state and check if capablity
'view_public_profiles' is visible. If that capability is not visible then client
will check `/media/moderation/request` to see is there a currently ongoing
moderation request. That info will also include current position in the
moderation queue. Client will show moderation info if images are pending
moderation. If not then client will show text "Profile is not set as public".

If capablility 'view_public_profiles' is set then update location with
`/profile/location` and start profile paging `/profile/page/next`.
Refresh is possible when using `/profile/page/reset`.

Paging info will include AccountIds and profile images. Profile images will be
downloaded on the fly using `/media/images/IMG`.

### Opened profile view

When profile is opened from the grid then it's information is get with
`/profile/profiles/ACCOUNT_ID`

### Settings

You can set profile visibility in the grid using `/account/settings/profile_visibility`.

### Image moderation

If capability 'admin_moderate_images' can be found the client displays option to
go image moderation mode. In that mode the app will fetch all images which need
moderation using `/media/admin/moderation/page/next`. Images in that request
will be downloaded using `/media/images/IMG`. It does not matter if image is
accepted or not. Moderation requests have an unique id. That id can be accepted
or not using `/media/admin/moderation/handle_request`.