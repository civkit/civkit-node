Trading over Lightning
======================

Civkit has experimental integration with Lightning to pay for the trades.

Buying Market Orders over Lightning
-----------------------------------

Civkit market orders accept both the Lightning standard BOLT11 invoice format andd the new Lightning standard BOLT12 format.

Those market orders can be published with the Civkit client command `sendmarketorder content board_pubkey`.

Civkit clients subscribing to market order events will receive an invoice in string format e.g:
`lnbc1pvjluezsp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygspp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmwwd5kgetjypeh2ursdae8g6twvus8g6rfwvs8qun0dq9qrsgq357wnc5r2ueh7ck6q93dj32dlqnls087fxdwk8qakdyafkq3yap9us6v52vjjsrvywa6rt52cm9r9zqt8r2t7mlcwspyetp5h2tztugp9lfyql`

This string should be enough information to be copy-pasted to a Lightning wallet and paid for trades over Lightning.
BOLT11 invoice issuers might opt for more custom BOLT11 invoice, like hold invoice where the `final_cltv_delta` is scaled up. 

This Lightning payment integration is experitmental and inter-compatibility with all Lightning wallets is still work-in-progress.

Buying Service Credentials over Lightning
-----------------------------------------

Currently the `CredentialAuthenticationResult` / `CredentialAuthenticationPayload` flow work with on-chain payment only.

Future releases of the Staking Credential framework can enable Lightning payment to be used as a scarce asset proof to redeem credentials:
https://github.com/civkit/staking-credentials-spec/blob/main/60-staking-credentials-archi.md#credentials-issuance

Lightning preimages can be added in accepted proofs in `CredentialAuthenticationResult`.
https://github.com/civkit/staking-credentials/blob/main/staking-credentials/src/common/msgs.rs#L172

Civkit Node's `CredentialGateway` credentials issuance flow can validate there has been an inbound payment paid for this
shared preimage by probing its Lightning backend. E.g Core-Lightning's `listpays`.
