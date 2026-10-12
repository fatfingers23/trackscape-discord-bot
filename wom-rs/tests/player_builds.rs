use wom_rs::models::group::GroupDetail;
use wom_rs::models::player::PlayerBuild;

/// Real captured data from WOM group 7253 ("Akhis Clan"), trimmed to three
/// memberships: an `f2p_lvl3` player, an `lvl3` player, and a `main` player.
/// Before the `PlayerBuild` enum was fixed to snake case, the whole group
/// failed to deserialize with `unknown variant 'f2p_lvl3'`, breaking group
/// sync for every clan containing one.
const FIXTURE: &str = include_str!("mocks/group/group_detail_f2p_lvl3_build.json");

#[test]
fn deserializes_f2p_lvl3_build_group() {
    let group: GroupDetail =
        serde_json::from_str(FIXTURE).expect("group with an f2p_lvl3 player must deserialize");

    let f2p_lvl3 = group
        .memberships
        .iter()
        .find(|m| m.player.username == "v t m n t s")
        .expect("fixture should contain the f2p_lvl3 player");
    assert_eq!(f2p_lvl3.player.build, PlayerBuild::F2pLvl3);

    assert!(
        group
            .memberships
            .iter()
            .any(|m| m.player.build == PlayerBuild::Lvl3),
        "fixture should contain a lvl3 player"
    );

    assert!(
        group
            .memberships
            .iter()
            .any(|m| m.player.build == PlayerBuild::Main),
        "fixture should contain a main player"
    );
}

#[test]
fn unknown_build_falls_back_to_unknown() {
    let json = FIXTURE.replace("\"build\": \"f2p_lvl3\"", "\"build\": \"future_build\"");

    let group: GroupDetail = serde_json::from_str(&json)
        .expect("an unrecognized future build must not fail deserialization");

    let player = group
        .memberships
        .iter()
        .find(|m| m.player.username == "v t m n t s")
        .expect("fixture should contain the patched player");
    assert_eq!(player.player.build, PlayerBuild::Unknown);
}

#[test]
fn player_build_serializes_back_to_wom_wire_format() {
    assert_eq!(
        serde_json::to_string(&PlayerBuild::F2pLvl3).expect("serialize"),
        "\"f2p_lvl3\""
    );
    assert_eq!(
        serde_json::to_string(&PlayerBuild::Main).expect("serialize"),
        "\"main\""
    );
}
