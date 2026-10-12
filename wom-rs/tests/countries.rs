use wom_rs::models::group::GroupDetail;
use wom_rs::models::player::Country;

/// Real captured data from WOM group 82 ("KnightSlayer"), trimmed to three
/// memberships: a `GB_SCT` player, a `GB` player, and a null-country player.
/// Before the `Country` enum was patched, the whole group failed to
/// deserialize with `unknown variant 'GB_SCT'`, breaking group sync.
const FIXTURE: &str = include_str!("mocks/group/group_detail_gb_constituent_countries.json");

#[test]
fn deserializes_gb_constituent_country_group() {
    let group: GroupDetail =
        serde_json::from_str(FIXTURE).expect("group with GB constituent-country codes must deserialize");

    let scotland = group
        .memberships
        .iter()
        .find(|m| m.player.username == "sir theomar")
        .expect("fixture should contain the GB_SCT player");
    assert_eq!(scotland.player.country, Some(Country::GB_SCT));

    assert!(
        group
            .memberships
            .iter()
            .any(|m| m.player.country == Some(Country::GB)),
        "fixture should contain a plain GB player"
    );

    assert!(
        group
            .memberships
            .iter()
            .any(|m| m.player.country.is_none()),
        "fixture should contain a null-country player"
    );
}

#[test]
fn unknown_country_code_falls_back_to_unknown() {
    let json = FIXTURE.replace("\"country\": \"GB_SCT\"", "\"country\": \"GB_LND\"");

    let group: GroupDetail = serde_json::from_str(&json)
        .expect("an unrecognized future country code must not fail deserialization");

    let player = group
        .memberships
        .iter()
        .find(|m| m.player.username == "sir theomar")
        .expect("fixture should contain the patched player");
    assert_eq!(player.player.country, Some(Country::Unknown));
}
