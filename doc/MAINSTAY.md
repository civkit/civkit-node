# Mainstay integration

All events undergo hash conversion, leading to the creation of a cumulative hash. This cumulative hash is then forwarded for commitment to the Mainstay proof-of-publication service for attestation.

The Mainstay service can be accessed through an HTTP interface or via a SOCKS5 Tor proxy. The service is operational and backed by a valid token_id for verification purposes. Funding, conducted through an LN payment, is executed in advance and separately from the subscription process (i.e., the token_id has already been processed). Mainstay proofs are stored and accessible, but the verification against bitcoind and staychain is conducted independently.

## Configuration

The node is configured with essential parameters, including the Mainstay server URL, slot index (position), authentication token, base public key, and chain code:

```
pub struct Mainstay {
    pub url: String,
    pub position: i32,
    pub token: String,
    pub base_pubkey: String,
    pub chain_code: String,
}
```

This can be added to `/src/config.rs`
