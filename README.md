Civkit Node
===========

https://civkit.org/

About Civkit Node
-----------------

The CivKit Node represents an experimental NIP-01 Rust relay, complemented by ongoing development of communication gateways for BOLT8 Noise transport and BOLT4 sphinx onion routing. Alongside this, there is a companion client binary designed specifically for development and testing purposes.

CivKit Node aims to enable a peer-to-peer electronic market system as described in the [paper](https://github.com/civkit/paper/blob/main/civ_kit_paper.pdf).

This is not production-ready software, please do not use it for real deployment for now.

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
Docker Build
------------

```
docker build -t civkit-node . 
docker run --rm civkit-node
```

Tagline
-------

*"Empathy with the users!"*

License
-------

CivKit Node is licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
