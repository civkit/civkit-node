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
[CIVKITD] - INIT: noise port 50011 nostr port 50021 cli_port 50031
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

./civkitd /* noise port */ 60001 /* nostr port */ 60011 cli_port 60021 (civkitd #2)

./civkit-cli connectpeer 60001

/* On logs of civkitd #1 */
[CIVKITD] - INIT: CivKit node starting up...
[CIVKITD] - INIT: noise port 50011 nostr port 50021 cli_port 50031
[CIVKITD] - NET: ready to listen tcp connection for clients !
[CIVKITD] - CONTROL: sending port to noise gateway !
[CIVKITD] - NOISE: opening outgoing noise connection!

/* On logs of civkitd #2 */
[CIVKITD] - INIT: CivKit node starting up...
[CIVKITD] - INIT: noise port 60001 nostr port 60011 cli_port 60021
[CIVKITD] - NET: ready to listen tcp connection for clients !
[CIVKITD] - NET: inbound noise connection !
```

Development Process
-------------------

The CivKit Node project embraces an open contributor model, inviting individuals to contribute 
through peer reviews, documentation, testing, and patches. 

If you're new to the project, it's advisable to begin with smaller tasks to get acquainted. 
Discussions regarding codebase enhancements take place on GitHub issues and pull requests, 
while communication regarding the development of CivKit Node primarily occurs on the CivKit Discord platform.

Building
--------

Install protobuf

```
Cargo build
```

Tagline
-------

*"Empathy with the users!"*

License
-------

CivKit Node is licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
