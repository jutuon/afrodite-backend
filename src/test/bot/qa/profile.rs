


use api_client::models::Location;

use crate::test::bot::actions::{BotAction, ActionArray, TO_NORMAL_STATE, AssertEquals, profile::{UpdateLocation, GetProfileList, ResetProfileIterator}, RunActions, ModifyTaskState, SleepUntil, AssertEqualsFn, account::SetProfileVisibility};

use super::{
    super::actions::{
        account::{Login, Register},
        media::SendImageToSlot,
        AssertFailure,
    },
    SingleTest,
};

use crate::test;

const LOCATION_LAT_LON_10: Location = Location { latitude: 10.0, longitude: 10.0 };

pub const PROFILE_TESTS: &[SingleTest] = &[
    test!(
        "Update location works",
        [
            RunActions(TO_NORMAL_STATE),
            UpdateLocation(LOCATION_LAT_LON_10),
            SetProfileVisibility(true),
            ModifyTaskState(|s| s.bot_count_update_location_to_lat_lon_10 += 1),
        ]
    ),
    test!(
        "Get profile changes when visiblity changes",
        [
            RunActions(TO_NORMAL_STATE),
            UpdateLocation(LOCATION_LAT_LON_10),
            SetProfileVisibility(true),
            ModifyTaskState(|s| s.bot_count_update_location_to_lat_lon_10 += 1),
            SleepUntil(|s| s.bot_count_update_location_to_lat_lon_10 >= 2),
            AssertEqualsFn(|v, _| v.profile_count(), 2, &GetProfileList),
            SetProfileVisibility(false),
            ResetProfileIterator,
            AssertEqualsFn(|v, _| v.profile_count(), 1, &GetProfileList),
        ]
    )
];
