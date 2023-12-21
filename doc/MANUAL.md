Civkit Node Operator Manual
===========================

Civkit is an experimental implementation of a peer-to-peer market. The main component of the Civkit architecture is `civkitd` a custom Nostr relay, accepting, storing and flooding market orders to connected clients.

Project architecture
--------------------

The CivKit ecosystem is composed of 3 modular daemon binaries, one client binary and one admin tool
- `civkitd`: The relay binary accepting Nostr clients connections or Civkit Service registration
- `civkit-cli`: A admin utility to manage and debug `civkitd`
- `civkit-sample`: An interactive language shell Nostr client able to send and receive market orders
- `civkit-marketd`: A lightweight binary managing a market service by registering to one or more `civkitd`
- `civkit-notaryd`: A lightweight binary managing exposing a market and events notarization service by registering to one or more `civkitd`

The sample can send the following requests to the relay:
- `sendtextnote content`: send a NIP-01 EVENT kind 1 to the relay
- `setmetadata username about picture`: send a NIP-01 EVENT kind 0 to the relay
- `recommend server urlrelay`: send a NIP-01 EVENT kind 2 to the relay
- `sendmarketorder content board_pubkey`: send a market order (kind: 32500) to the relay
- `opensubscription subscriptionid kinds since until`: open a subscription to the relay
- `closesubscription subscriptionid`: close a subscription to the relay
- `submitcredentialproof merkle_block`: submit a staking credential proof to the relay
- `verifyinclusionproof`: verify the inclusion proof

The `civkit-cli` can send the following commands to the relay:
- `ping`: send a ping message
- `shutdown`: shutdown the connected CivKit node
- `publishtextnote`: send a demo NIP-01 EVENT kind 1 to all the connected clients
- `listclients: list information about connected clients
- `listsubscriptions`: list information about subscriptions
- `connectpeer`: connect to a BOLT8 peer on local port
- `disconnectclient`: disconnect from a client
- `publishnotice`: send a demo NIP-01 NOTICE to all connected clients
- `publishoffer`: send a BOLT12 offers to all the connected clients
- `publishinvoice`: send a BOLT11 invoice to all the ocnnected clients
- `list-db-events`: list DB entries
- `help`: print the help(s) of the subcommands

Running Civkit Node for Demo
----------------------------

Opening a subscription and sending basic text note.

```
./civkitd

./civkit-sample (civkit-sample #1)
Civkit sample startup successful. Enter "help" to view available commands.
> opensubscription helloworld /*kinds*/ 1 /*since*/ 0 /*until*/ 10000000000
>
[EOSE] helloworld
> sendtextnote hola
>
[EVENT] hola
> 
/* Second sample connect and subscribes */
[EVENT] bonjour
> recommendserver https://civkit.org
> 

./civkit-sample (civkit-sample #2)
Civkit sample startup successful. Enter "help" to view available commands.
> opensubscription helloworld /*kinds*/ 1 */since*/ 0 /*until*/ 1000000000
>
[EOSE] helloworld
>
[EVENT] bonjour
>

/* On logs of civkitd  */
[CIVKITD] - INIT: CivKit node starting up...
[CIVKITD] - INIT: noise port 9735 nostr port 50021 cli_port 50031
[CIVKITD] - NET: ready to listen tcp connection for clients !
[CIVKITD] - NET: receive a tcp connection !
[CIVKITD] - NET: incoming tcp Connection from :[::1]:50422
[CIVKITD] - NET: websocket established: [::1]:50422
[CIVKITD] - NOSTR: Message received from 1!
[CIVKITD] - NOSTR: New subscription id 1
[CIVKITD] - NOSTR: Message received from 1!
[CIVKITD] - NET: receive a tcp connection !
[CIVKITD] - NET: incoming tcp Connection from :[::1]:50423
[CIVKITD] - NET: websocket established: [::1]:50423
[CIVKITD] - NOSTR: Message received from 2!
[CIVKITD] - NOSTR: New subscription id 2
[CIVKITD] - NOSTR: Message received from 2!
[CIVKITD] - NOSTR: Message received from 1!
```

Connecting to a BOLT8 peer on local.

```
./civkitd (civtkid #1)

./civkitd 
./civkitd --noise-port 60001 --nostr-port 60011 --cli-port 60021 (civkitd #2)

./civkit-cli connectpeer 60001

/* On logs of civkitd #1 */
[CIVKITD] - INIT: CivKit node starting up...
[CIVKITD] - INIT: noise port 9735 nostr port 50021 cli_port 50031
[CIVKITD] - NET: ready to listen tcp connection for clients !
[CIVKITD] - CONTROL: sending port to noise gateway !
[CIVKITD] - NOISE: opening outgoing noise connection!

/* On logs of civkitd #2 */


[CIVKITD] - INIT: CivKit node starting up...
[CIVKITD] - INIT: noise port 60001 nostr port 60011 cli_port 60021
[CIVKITD] - NET: ready to listen tcp connection for clients !
[CIVKITD] - NET: inbound noise connection !
```

Publishing a trade order (BOLT11 version).

```
./civkitd

/* On logs of civkitd */
[CIVKITD] - INIT: CivKit node starting up...
[CIVKITD] - INIT: noise port 9735 nostr port 50021 cli_port 50031
[CIVKITD] - NET: ready to listen tcp connection for clients !

./civkit-sample (civkit-sample #1)
Civkit sample startup successful. Enter "help" to view available commands

/* On prompt of civkit-sample #1 */
> sendmarketorder lnbc1pvjluezsp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygspp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmwwd5kgetjypeh2ursdae8g6twvus8g6rfwvs8qun0dq9qrsgq357wnc5r2ueh7ck6q93dj32dlqnls087fxdwk8qakdyafkq3yap9us6v52vjjsrvywa6rt52cm9r9zqt8r2t7mlcwspyetp5h2tztugp9lfyql
> 

./civkit-sample (civkitd #2)
Civkit sample startup successful. Enter "help" to view available commands

/* On prompt of civkit-sample #2 */
> opensubscription market 32500 0 100000
> 
[EVENT] new trade offer:   lnbc1pvjluezsp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygspp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmwwd5kgetjypeh2ursdae8g6twvus8g6rfwvs8qun0dq9qrsgq357wnc5r2ueh7ck6q93dj32dlqnls087fxdwk8qakdyafkq3yap9us6v52vjjsrvywa6rt52cm9r9zqt8r2t7mlcwspyetp5h2tztugp9lfyql

[EOSE] fd0b0e4a871a00de7730d07943ec2247
> 

/* On logs of civkitd */
[CIVKITD] - NET: receive a tcp connection !
[CIVKITD] - NET: incoming tcp Connection from :[::1]:49659
[CIVKITD] - NET: websocket established: [::1]:49659
[CIVKITD] - NOTE PROCESSING: Opening database for read / write new client
[CIVKITD] - NOTE PROCESSING: 0 rows were updated
[CIVKITD] - NOTE PROCESSING: 1 rows were updated
[CIVKITD] - NOTE PROCESSING: Note processor received DB requests
[CIVKITD] - NOSTR: Message received from 1!
[CIVKITD] - NOTE PROCESSING: Opening database for read / write new event
[CIVKITD] - NOTE PROCESSING: 0 rows were updated
[CIVKITD] - NOTE PROCESSING: 1 rows were updated
[CIVKITD] - NOTE PROCESSING: Note processor received DB requests
[CIVKITD] - NET: receive a tcp connection !
[CIVKITD] - NET: incoming tcp Connection from :[::1]:49660
[CIVKITD] - NET: websocket established: [::1]:49660
[CIVKITD] - NOTE PROCESSING: Opening database for read / write new client
[CIVKITD] - NOTE PROCESSING: table creation failed: table client already exists
[CIVKITD] - NOTE PROCESSING: 1 rows were updated
[CIVKITD] - NOTE PROCESSING: Note processor received DB requests
[CIVKITD] - NOSTR: Message received from 2!
[CIVKITD] - NOTE PROCESSING: Note processor received DB requests
[CIVKITD] - NOSTR: sending event for client 2
[CIVKITD] - NOSTR: sending event for client 1
```

Being Paid to host Market orders
--------------------------------

The Civkit Node relay is powered by the Staking Credential framework, a cryptographic
protocol that allows users to receive an amount of anonymous credentials in exchange
of paying an on-chain payment (off-chain payment to come). Those anonymous credentials
can be used at a later time to redeem services.

Currently, the services that can be redeemed are market order posting and relaying to
the connected Clients, assuming they have opened a subscription.

The integration with Bitcoin Core wallet for automatic workflow is work-in-progress.

For now, a Civkit Node operator can issue on-chain address manually with the following
commands: `ADDR=`bitcoin-cli getnewaddress``

This address $ADDR can be communicated out of band to the Civkit clients.

Once the address is paid and the transaction has been included in the chain, the Civkit
client can generate a transaction proof from the txid (the $TXID). Bitcoin Core with the
following command `PROOF=gettxoutproof "[\"$TXID\"]"`.

As of today, there is no cryptographic binding between the paid address and the proof,
so the address should be kept confidential between the Civkit Node and the Civkit client.

(See https://github.com/civkit/staking-credentials-spec/issues/6)

The proof can be communicated by the Civkit client with the command `submitcredentialproof
$PROOF`. Signed credentials should be automatically signed by the Civkit Node server
and share back to the Client. Those credentials are cached by the Client.

Once the credential exchange step is done, `sendmarketorder` can be send by Civkit client
to post market order and get them relayed over all Civkit clients, which have subscribed
to nostr event `32500`.

Standard documentation on how to use Bitcoin Core and its wallet is available at:
https://github.com/bitcoin/bitcoin/tree/master/doc
