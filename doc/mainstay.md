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

The node will construct commitments from specified *events* () in `src/events.rs`. 

The commitment can be simply constructed from the sha256 hash of each event (encoded as a string) similar to:

```
pub fn make_commitment(data: &String) -> (String) {
    let mut data_vec = data.as_bytes().iter().cloned().collect::<Vec<u8>>();

    let commitment = sha256d::Hash::hash(&data_vec);
    return (commitment.to_string());
}
```

Will determine which events need to be attested. 

## Commitment compression

Initially assume every event will be committed to the mainstay service endpoint. 

It may be more efficient to compress several events into a single commitment and then only commit every `commitment_interval`. 

## Proof retreival

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
