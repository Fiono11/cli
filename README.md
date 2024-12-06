# Threshold Signature CLI

The Threshold Signature CLI allows multiple participants to collaboratively generate a threshold signature scheme where:

- A **threshold public key** is shared among **n** participants.
- Each participant holds a share of the **threshold secret key**.
- A threshold of **t-of-n** shares is required to sign with the threshold public key.

This tool is useful for scenarios where trust and security need to be distributed among multiple parties, such as in multi-signature wallets or collaborative decision-making processes.

## Prerequisites

- [Subkey](https://docs.substrate.io/reference/command-line-tools/subkey/) installed for key pair generation.
- [Docker](https://docs.docker.com/get-docker/) installed for building and running the Threshold Signature CLI Docker image or [Rust](https://www.rust-lang.org) installed.

## Setup

### Build and Run the Docker Image

1. **Build the Docker image:**

   ```bash
   docker build -t olaf-cli .
   ```

2. **Run the Docker container:**

   ```bash
   docker run -it --entrypoint /bin/bash olaf-cli
   ```

3. **Navigate to the CLI directory:**

   ```bash
   cd /usr/local/bin/
   ```

## Tutorial

This tutorial demonstrates how to set up a threshold signature scheme with **2 participants** and a **threshold of 2** (both participants are required to sign).

**Note:** Each step of the protocol and the corresponding CLI command must be run individually by each participant, except the last two steps (aggregation and submission). Outputs of other participants must be shared using out-of-band communication and manually created if run on different machines. But can also be run automatically on the same machine.

### Step 1: Generate Key Pairs 

Each participant generates their own key pair using `subkey`.

**Run the following command:**

```bash
subkey generate
```

**Example Outputs:**

#### Participant 1

```
Secret phrase:     position jar enforce stable collect dolphin logic game health lucky vehicle adult
Network ID:        substrate
Secret seed:       0x473a77675b8e77d90c1b6dc2dbe6ac533b0853790ea8bcadf0ee8b5da4cfbbce
Public key (hex):  0x14a0c1a24a3e3cecf8b6c819d48d719d787c79638eca81aeb43b1de0a2e1de4c
Account ID:        0x14a0c1a24a3e3cecf8b6c819d48d719d787c79638eca81aeb43b1de0a2e1de4c
Public key (SS58): 5CXkZyy4S5b3w16wvKA2hUwzp5q2y7UtRPkXnW97QGvDN8Jw
SS58 Address:      5CXkZyy4S5b3w16wvKA2hUwzp5q2y7UtRPkXnW97QGvDN8Jw
```

#### Participant 2

```
Secret phrase:     eye goddess hotel merge sand lesson exclude bird shell arrive sample wise
Network ID:        substrate
Secret seed:       0xdb9ddbb3d6671c4de8248a4fba95f3d873dc21a0434b52951bb33730c1ac93d7
Public key (hex):  0xd01bfaf1d2fee109029bc0999573bf2ea7af6420ab0d0c8b8e93dcfc48af3959
Account ID:        0xd01bfaf1d2fee109029bc0999573bf2ea7af6420ab0d0c8b8e93dcfc48af3959
Public key (SS58): 5Gma8SNsn6rkQf9reAWFQ9WKq8bwwHtSzwMYtLTdhYsGPKiy
SS58 Address:      5Gma8SNsn6rkQf9reAWFQ9WKq8bwwHtSzwMYtLTdhYsGPKiy
```

### Step 2: Prepare the Threshold Signature CLI Environment

Ensure you have built and are inside the Docker container as per the [Setup](#setup) section.

### Step 3: Run the Threshold Key Generation Protocol

This protocol consists of multiple rounds where participants generate and exchange data to collaboratively create a threshold public key and sign transactions.

#### Step 3.1: Generate the Threshold Public Key

##### Step 3.1.1: Create the Recipients and Secret Key files

Each participant needs a `recipients.json` file containing the SS58 public keys of all participants.

**Command:**

```bash
echo '[
  "5CXkZyy4S5b3w16wvKA2hUwzp5q2y7UtRPkXnW97QGvDN8Jw",
  "5Gma8SNsn6rkQf9reAWFQ9WKq8bwwHtSzwMYtLTdhYsGPKiy"
]' > recipients.json
```

Each participant runs the corresponding command:

   ```bash
   echo '"0x473a77675b8e77d90c1b6dc2dbe6ac533b0853790ea8bcadf0ee8b5da4cfbbce"' > contributor_secret_key1.json
   ```

   **Participant 2:**

   ```bash
   echo '"0xdb9ddbb3d6671c4de8248a4fba95f3d873dc21a0434b52951bb33730c1ac93d7"' > contributor_secret_key2.json
   ```

##### Step 3.1.2: Generate Round 1 Messages

Each participant runs the corresponding command:

   ```bash
   ./olaf_cli generate-threshold-public-key-round1 --threshold 2 --participant 1
   ```

   ```bash
   ./olaf_cli generate-threshold-public-key-round1 --threshold 2 --participant 2
   ```

##### Step 3.1.2: Generate the Secret Signing Shares and the Threshold Public Key

Each participant runs the corresponding command:

```bash
./olaf_cli generate-threshold-public-key-round2 --participant 1
```

```bash
./olaf_cli generate-threshold-public-key-round2 --participant 2
```

**Note:** The `threshold_public_key.json` should be the same for all participants if the steps are followed correctly.

#### Step 3.2: Generate the Threshold Signature

This process allows the participants to collaboratively sign a transaction using the threshold key.

##### Step 3.2.1: Generate the secret Signing Nonces and the public Signing Commitments

Each participant runs the corresponding command:

```bash
./olaf_cli threshold-sign-round1 --participant 1
```

```bash
./olaf_cli threshold-sign-round1 --participant 2
```

##### Step 3.2.2: Generate the Signing Packages

**Default Values:**

- **URL:** `wss://westend-rpc.polkadot.io`
- **Pallet:** `System`
- **Call Name:** `remark`
- **Call Data:** `((197, 38))`
- **Context:** `substrate`

**Note:** You can override these defaults using flags, e.g., `--url "custom_url"`.

Each participant runs the corresponding command:

```bash
./olaf_cli threshold-sign-round2 --participant 1
```

```bash
./olaf_cli threshold-sign-round2 --participant 2
```

##### Step 3.2.3: Aggregate the Signing Packages

Only one participant needs to run the following command:

**Command:**

```bash
./olaf_cli aggregate-threshold-extrinsic
```

This produces the final threshold signature (`threshold_signature.json`), ready for submission.

#### Step 3.3: Submit the Threshold Signature

##### Step 3.3.1: Fund the Threshold Account

Ensure the threshold account (identified by `threshold_public_key.json`) has sufficient funds. You can use a faucet or transfer from existing accounts.

For example, on the Westend network, use the [Westend Faucet](https://matrix.to/#/#westend_faucet:matrix.org).

##### Step 3.3.2: Submit the Threshold Extrinsic

Only one participant needs to run the following command:

Run:

```bash
cargo run submit-threshold-extrinsic
```

This submits the threshold-signed extrinsic to the network.
