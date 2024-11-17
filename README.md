- Generate two keypairs using subkey generate

Participant 1:

Secret phrase:       position jar enforce stable collect dolphin 
logic game health lucky vehicle adult
  Network ID:        substrate
  Secret seed:       0x473a77675b8e77d90c1b6dc2dbe6ac533b0853790ea8bcadf0ee8b5da4cfbbce
  Public key (hex):  0x14a0c1a24a3e3cecf8b6c819d48d719d787c79638eca81aeb43b1de0a2e1de4c
  Account ID:        0x14a0c1a24a3e3cecf8b6c819d48d719d787c79638eca81aeb43b1de0a2e1de4c
  Public key (SS58): 5CXkZyy4S5b3w16wvKA2hUwzp5q2y7UtRPkXnW97QGvDN8Jw
  SS58 Address:      5CXkZyy4S5b3w16wvKA2hUwzp5q2y7UtRPkXnW97QGvDN8Jw

Participant 2:

Secret phrase:       eye goddess hotel merge sand lesson exclude bird shell arrive sample wise
  Network ID:        substrate
  Secret seed:       0xdb9ddbb3d6671c4de8248a4fba95f3d873dc21a0434b52951bb33730c1ac93d7
  Public key (hex):  0xd01bfaf1d2fee109029bc0999573bf2ea7af6420ab0d0c8b8e93dcfc48af3959
  Account ID:        0xd01bfaf1d2fee109029bc0999573bf2ea7af6420ab0d0c8b8e93dcfc48af3959
  Public key (SS58): 5Gma8SNsn6rkQf9reAWFQ9WKq8bwwHtSzwMYtLTdhYsGPKiy
  SS58 Address:      5Gma8SNsn6rkQf9reAWFQ9WKq8bwwHtSzwMYtLTdhYsGPKiy

- Generate a threshold public key shared between n participants
- Each participant has a share of the threshold secret key
- A threshold t-of-n of shares are needed to sign with the threshold public key 

Docker

docker build -t olaf-cli .

docker run -it --entrypoint /bin/bash olaf-cli

cd usr/local/bin/

Note: All commands use the default path "." but it can be overriden with --files "custom_path"

- Create a "recipients.json" file with the participants' public keys:

echo '[
  "5CXkZyy4S5b3w16wvKA2hUwzp5q2y7UtRPkXnW97QGvDN8Jw",
  "5Gma8SNsn6rkQf9reAWFQ9WKq8bwwHtSzwMYtLTdhYsGPKiy"
]' > recipients.json

- Generate the message of round 1 of participant 1 to be sent to all participants

- Create a "contributor_secret_key.json" file with: 

echo '"0x473a77675b8e77d90c1b6dc2dbe6ac533b0853790ea8bcadf0ee8b5da4cfbbce"' > contributor_secret_key.json

- cargo run generate-threshold-public-key-round1 --threshold 2

- Generate the message of round 1 of participant 2 to be sent to all participants

- Create a "contributor_secret_key.json" file with: 

echo '"0xdb9ddbb3d6671c4de8248a4fba95f3d873dc21a0434b52951bb33730c1ac93d7"' > contributor_secret_key.json

- cargo run generate-threshold-public-key-round1 --threshold 2

- Aggregate the messages from all participants in "all_messages.json" file

- Generate the secret signing share for each partipant 1 and the threshold public key

cargo run generate-threshold-public-key-round2 

- Generate the secret signing nonces and the corresponding public signing commitments of each participant

cargo run threshold-sign-round1 

- Fund the threshold account with, for example, a [faucet](https://faucet.polkadot.io/westend)

- Generate the public signing package for each participant
- The default values are the following:
  - url: "wss://westend-rpc.polkadot.io",
  - pallet: "System",
  - call_name: "remark",
  - call_data: "((197, 38))",
  - context: "substrate",
- Can be overriden with, for example, --url "custom_url"

cargo run threshold-sign-round2 

- Aggregate the public signing packages

cargo run aggregate-threshold-extrinsic 

- Submit the threshold extrinsic

cargo run submit-threshold-extrinsic 



