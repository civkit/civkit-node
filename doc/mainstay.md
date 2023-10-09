# Mainstay integration

Options for attesting state to a mainstay proof-of-publication service. 

Assumptions: 

Mainstay service is available over http interface (or via SOCKS5 Tor proxy). 
Mainstay service is available and funded with a valid `token_id` for verifiation. 
Funding (via LN payment) is performed in advance and out of band for subscrption. (i.e. `token_id` is already performed.)
Mainstay proofs are stored and made available, but that verification against `bitcoind` and staychain occur separately. 

## Config

Node is configured with the mainstay server URL, the slot index and the authentication token:

```
pub struct MainstayConfig {
    url: String,
    position: u64,
    token: String,
}
```

This can be added to `/src/config.rs`

## Commitment function

Impliementation of a commitment function that performs a POST request to the `/commitment/send` mainstay service route, with payload:

```
payload = {
  commitment: commitment,
  position: 0,
  token: '4c8c006d-4cee-4fef-8e06-bb8112db6314',
};
```

`commitment` is a 32 byte value encoded as a 64 character hex string

This can be performed using the `Reqwest` http client library (as in mercury server), e.g. 

```
use reqwest;

pub struct Request(reqwest::blocking::RequestBuilder);

impl Request {
    //Construct a request from the give payload and config
    pub fn from(
        payload: Option<&Payload>,
        command: &String,
        config: &MainstayConfig,
        signature: Option<String>,
    ) -> Result<Self> {
        //Build request
        let client = reqwest::blocking::Client::new();
        let url = reqwest::Url::parse(&format!("{}/{}", config.url(), command))?;

        //If there is a payload this is a 'POST' request, otherwise a 'GET' request
        let req = match payload {
            Some(p) => {
                let payload_str = String::from(serde_json::to_string(&p)?);
                let payload_enc = encode(payload_str);
                let mut data = HashMap::new();
                data.insert("X-MAINSTAY-PAYLOAD", &payload_enc);
                let sig_str = match signature {
                    Some(s) => s,
                    None => String::from(""),
                };
                data.insert("X-MAINSTAY-SIGNATURE", &sig_str);
                client
                    .post(url)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .json(&data)
            }
            None => client
                .get(url)
                .header(reqwest::header::CONTENT_TYPE, "application/json"),
        };

        Ok(Self(req))
    }

    pub fn send(self) -> std::result::Result<reqwest::blocking::Response, reqwest::Error> {
        self.0.send()
    }
}
```

## Commitment construction

The node will construct commitments from specified *events* () in `src/events.rs`. The `event_id` already hashes the full payload of the event object and can be used for commitment directly. 

Initially assume all events are committed. Can add config to set commitment for specific events. 

## Commitment compression

By committing a a *cumulative* hash to the mainstay slot, and saving the cumulative hash alongside the `event_id`, then if individual commitment operations fail, or the mainstay service is temporarily unavailable, the unbroken sequence of events is verifiable as unquine up until the the latest commitment operation. 

In this approach, for each *event* that occurs (in time sequence), the `event_id` is concatenated with the previous cumulative hash and committed to mainstay. 

So, for the first event: `event_id` is used as the commitment and sent to the mainstay commitment endpoint. This is labeled `event_id[0]`. 

For the next event (`event_id[1]`), the commitment hash is computed: `comm_hash[1] = SHA256(event_id[0] || event_id[1])` and committted to the mainstay service API. 

For the next event (`event_id[2]`), the commitment hash is computed: `comm_hash[2] = SHA256(comm_hash[1] || event_id[2])` and committted to the mainstay service API. 

For the next event (`event_id[3]`), the commitment hash is computed: `comm_hash[3] = SHA256(comm_hash[2] || event_id[3])` and committted to the mainstay service API. 

And so on, committing the chain. `comm_hash[n]` does not strictly need to be saved as the chain can be reconstructed directly from the `event_id[n]` saved in the DB. 

## Proof retreival

TODO

After each commitment, retreive the slot proof from the mainstay server API (call to GET `/commitment/commitment` route with the `commitment` hex string). This will return attestion info (`TxID`) and the slot proof (Merkle proof). 

```
    pub struct Proof {
        merkle_root: Commitment,
        commitment: Commitment,
        ops: Vec<Commitment>,
        append: Vec<bool>,
        position: u64,
    }
```

This will need to be stored in a new DB table corresponding to events. 
