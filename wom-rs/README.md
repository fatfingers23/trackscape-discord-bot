<div align="center">
    <h1>wom_rs</h1>
</div>

A asynchronous rust wrapper for interacting with [Wise Old Man API](https://docs.wiseoldman.net/).

The goal of wom_rs is to provide a 1:1 in rust to their API. This is still a work in progress and can find below in [Endpoints Progress](#endpoints-progress) the current status of the endpoints. If you do find any issues or have any suggestions please feel free to open an issue.  


## Getting Started

add `wom_rs` to your `Cargo.toml` file:

```toml
[dependencies]
wom_rs = "0.1.0"
```

To get a client for interacting with the API, use the `WomClient` struct:

```rust
use wom_rs::WomClient;

//Default client
let wom_client = WomClient::new();

//Client with an api key
let api_key = std::env::var("WOM_API_KEY").unwrap();
let wom_client = WomClient::new_with_key(api_key);
```

## Example Web Call
```rust
use wom_rs::WomClient;
let wom_client = WomClient::new();

let details = wom_client.player_client.get_details("IFat Fingers".to_string()).await;
println!("{:?}", details);
```



## Endpoints Progress
Some may have cross over completion. For example you can get player name changes by username, but cannot submit request for a name change.
* [x] [Players API](https://docs.wiseoldman.net/players-api/player-endpoints)
* [ ] [Groups API](https://docs.wiseoldman.net/groups-api/group-endpoints)
* [ ] [Competitions API](https://docs.wiseoldman.net/competitions-api/competition-endpoints)
* [ ] [Records API](https://docs.wiseoldman.net/records-api/record-endpoints)
* [ ] [Deltas API](https://docs.wiseoldman.net/deltas-api/delta-endpoints)
* [ ] [Name Change API](https://docs.wiseoldman.net/names-api/name-endpoints)
* [ ] [Efficiency API](https://docs.wiseoldman.net/efficiency-api/efficiency-endpoints)
* [ ] Full handle of errors
  * [x] Not found should be properly handled
  * ... More will appear as I find them

## Project Layout

### [Clients](./src/clients)
Contains the clients for interacting with the API. Each of the above endpoints will have a client accessed from [WomClient](./src/lib.rs).

### [Models](./src/models)
Various models used to represent the data returned from the API. You will find that many of the responses handled in [models](./src/models) are very similar to each other. In the WOM API many responses are built off of each other.
I did not find a good way to do this in rust. So I kept them separate for clarity.
  
Some of the models that may not be clear are:
  - [error](./src/models/error.rs): Holds models for error responses from the API.
  - [snapshot](./src/models/snapshot.rs): Holds models for the snapshot data returned from the API. This mimics very closely to [PlayerGain](./src/models/player.rs) but kept separate for clarity.

### [Mock Responses](./test/mocks)
Contains the mock responses for the tests. These are used to test the deserialization of the responses from the API. All examples are either from the real API or the examples provided in WOM fantastic API documentation.

## Special Thanks
None of this would be possible with out the wonderful team at [Wise Old Man](https://github.com/wise-old-man). Would not need a wrapper if it was not for their fantastic website. Not to mention their amazing [documentation](https://docs.wiseoldman.net) made creating this a breeze. 