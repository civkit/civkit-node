Civkit Node
===========

https://civkit.org/

About Civkit Node
-----------------

The CivKit Node represents an experimental NIP-01 Rust relay, complemented by ongoing development of communication gateways for BOLT8 Noise transport and BOLT4 sphinx onion routing. Alongside this, there is a companion client binary designed specifically for development and testing purposes.

CivKit Node aims to enable a peer-to-peer electronic market system as described in the [paper](https://github.com/civkit/paper/blob/main/civ_kit_paper.pdf).

This is not production-ready software, please do not use it for real deployment for now.

Project Architecture
--------------------

There are 3 binaries available:
- `civkitd`: The relay binary accepting Nostr clients and BOLT8 peers connections
- `civkit-cli`: A utility to manage the server
- `civkit-sample`: An interactive language shell Nostr client for testing and development purposes

As of v0.0.1, the sample can send the following requests to the relay:
- `sendtextnote content`: send a NIP-01 EVENT kind 1 to the relay
- `setmetadata username about picture`: send a NIP-01 EVENT kind 0 to the relay
- `recommendserver urlrelay`: send a NIP-01 EVENT kind 2 to the relay
- `opensubscription subscriptionid kinds since until`: send a NIP-01 REQ to the relay
- `closesubscription subscriptionid`: send a NIP-01 to the relay

The `civkit-cli` is based on gRPC and can send the following requests:
- `publishnotice`: send a demo NIP-01 NOTICE to all the connected clients 
- `publishtextnote`: send a demo NIP-01 EVENT kind 1 to all the connected clients
- `connectpeer localport`: connect to a BOLT8 peer on local port

Running CivKit Node for Demo
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
> submitcredentialproof 0000002006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f441a4f6750cce9e7b80d22a314d107abd8a50bf7b9bd60cc74acba1260b4df487c584d65ffff7f20000000000100000001441a4f6750cce9e7b80d22a314d107abd8a50bf7b9bd60cc74acba1260b4df480101
> sendmarketorder lnbc1pvjluezsp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygspp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmwwd5kgetjypeh2ursdae8g6twvus8g6rfwvs8qun0dq9qrsgq357wnc5r2ueh7ck6q93dj32dlqnls087fxdwk8qakdyafkq3yap9us6v52vjjsrvywa6rt52cm9r9zqt8r2t7mlcwspyetp5h2tztugp9lfyql 03042d2018cb7d3b83cbde0b2a7ee85f68ff2f4d74d137fd1f6fe27325d318e683

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

Development Process
-------------------

The CivKit Node project embraces an open contributor model, inviting individuals to contribute 
through peer reviews, documentation, testing, and patches. 

If you're new to the project, it's advisable to begin with smaller tasks to get acquainted. 
Discussions regarding codebase enhancements take place on GitHub issues and pull requests, 
while communication regarding the development of CivKit Node primarily occurs on the CivKit Discord platform.

For more "how-to" on contributing to an open-source in the Bitcoin ecosystem, have a look on
documentation written by experienced Bitcoin protocols developers on [how to make a PR](https://github.com/jonatack/bitcoin-development/blob/master/how-to-make-bitcoin-core-prs.md)
and [how to review a PR](https://github.com/jonatack/bitcoin-development/blob/master/how-to-review-bitcoin-core-prs.md).

Building
--------

Install protobuf

```
cargo build
cd target/debug
#run commands above like ./civkitd
```

Tagline
-------

*"Empathy with the users!"*

License
-------

CivKit Node is licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
