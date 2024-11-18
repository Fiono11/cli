# Threshold Signature CLI

The Threshold Signature CLI allows multiple participants to collaboratively generate a threshold signature scheme where:

- A **threshold public key** is shared among **n** participants.
- Each participant holds a share of the **threshold secret key**.
- A threshold of **t-of-n** shares is required to sign with the threshold public key.

This tool is useful for scenarios where trust and security need to be distributed among multiple parties, such as in multi-signature wallets or collaborative decision-making processes.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Setup](#setup)
- [Tutorial](#tutorial)
  - [Step 1: Generate Key Pairs for Participants](#step-1-generate-key-pairs-for-participants)
  - [Step 2: Prepare the Threshold Signature CLI Environment](#step-2-prepare-the-threshold-signature-cli-environment)
  - [Step 3: Run the Threshold Key Generation Protocol](#step-3-run-the-threshold-key-generation-protocol)
    - [Step 3.1: Generate the Threshold Public Key](#step-31-generate-the-threshold-public-key)
      - [Step 3.1.1: Create the `recipients.json` File](#step-311-create-the-recipientsjson-file)
      - [Step 3.1.2: Generate Round 1 Messages](#step-312-generate-round-1-messages)
      - [Step 3.1.3: Exchange Round 1 Messages](#step-313-exchange-round-1-messages)
      - [Step 3.1.4: Aggregate Round 1 Messages](#step-314-aggregate-round-1-messages)
      - [Step 3.1.5: Generate Secret Signing Shares and Threshold Public Key](#step-315-generate-secret-signing-shares-and-threshold-public-key)
    - [Step 3.2: Generate the Threshold Signature](#step-32-generate-the-threshold-signature)
      - [Step 3.2.1: Generate Signing Nonces and Commitments](#step-321-generate-signing-nonces-and-commitments)
      - [Step 3.2.2: Exchange Signing Commitments](#step-322-exchange-signing-commitments)
      - [Step 3.2.3: Aggregate Signing Commitments](#step-323-aggregate-signing-commitments)
      - [Step 3.2.4: Generate Partial Signatures](#step-324-generate-partial-signatures)
      - [Step 3.2.5: Exchange Partial Signatures](#step-325-exchange-partial-signatures)
      - [Step 3.2.6: Aggregate Partial Signatures](#step-326-aggregate-partial-signatures)
    - [Step 3.3: Submit the Threshold Signature](#step-33-submit-the-threshold-signature)
      - [Step 3.3.1: Fund the Threshold Account](#step-331-fund-the-threshold-account)
      - [Step 3.3.2: Submit the Transaction](#step-332-submit-the-transaction)
- [Additional Notes and Best Practices](#additional-notes-and-best-practices)
- [Summary of Steps for Each Participant](#summary-of-steps-for-each-participant)
- [License](#license)

## Overview

In a threshold signature scheme, multiple participants collaborate to generate a signature that requires a minimum number of participants (threshold) to sign transactions. This enhances security by distributing trust and control.

This CLI tool implements the protocol for threshold key generation and signing, allowing participants to:

- Generate individual key pairs.
- Collaboratively generate a threshold public key.
- Produce threshold signatures for transactions.

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

**Note:** Each CLI command must be run individually by each participant. Outputs must be manually gathered and shared between participants using out-of-band communication or by running each participant's steps on the same machine.

### Step 1: Generate Key Pairs for Participants

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

**Important:** Keep your **Secret phrase** and **Secret seed** confidential.

### Step 2: Prepare the Threshold Signature CLI Environment

Ensure you have built and are inside the Docker container as per the [Setup](#setup) section.

### Step 3: Run the Threshold Key Generation Protocol

This protocol consists of multiple rounds where participants generate and exchange data to collaboratively create a threshold public key and sign transactions.

#### Step 3.1: Generate the Threshold Public Key

##### Step 3.1.1: Create the `recipients.json` File

Each participant needs a `recipients.json` file containing the SS58 public keys of all participants.

**Command:**

```bash
echo '[
  "5CXkZyy4S5b3w16wvKA2hUwzp5q2y7UtRPkXnW97QGvDN8Jw",
  "5Gma8SNsn6rkQf9reAWFQ9WKq8bwwHtSzwMYtLTdhYsGPKiy"
]' > recipients.json
```

**Note:** Replace the public keys with those of your actual participants.

##### Step 3.1.2: Generate Round 1 Messages

For each participant:

1. **Create `contributor_secret_key.json`:**

   ```bash
   echo '"YOUR_SECRET_SEED_IN_HEX"' > contributor_secret_key.json
   ```

   **Participant 1:**

   ```bash
   echo '"0x473a77675b8e77d90c1b6dc2dbe6ac533b0853790ea8bcadf0ee8b5da4cfbbce"' > contributor_secret_key.json
   ```

   **Participant 2:**

   ```bash
   echo '"0xdb9ddbb3d6671c4de8248a4fba95f3d873dc21a0434b52951bb33730c1ac93d7"' > contributor_secret_key.json
   ```

2. **Run `generate-threshold-public-key-round1`:**

   ```bash
   cargo run generate-threshold-public-key-round1 --threshold 2
   ```

   This command generates a `all_messages.json` file that needs to be shared with all other participants.

##### Step 3.1.3: Exchange Round 1 Messages

Participants share their `all_messages.json` files with each other using secure, out-of-band communication methods.

##### Step 3.1.4: Aggregate Round 1 Messages

Each participant collects each partipant's `all_messages.json` files (including their own) and aggregates them manually into `all_messages.json`.

Ensure that `all_messages.json` contains messages from all participants.

##### Step 3.1.5: Generate Secret Signing Shares and Threshold Public Key

Each participant runs:

```bash
cargo run generate-threshold-public-key-round2
```

This command uses:

- `contributor_secret_key.json`
- `recipients.json`
- `all_messages.json`

It generates:

- **Signing share** (`signing_share.json`).
- **Generation output** (`generation_output.json`).
- **Threshold public key** (`threshold_public_key.json`).

**Note:** The `threshold_public_key.json` should be the same for all participants if the steps are followed correctly.

#### Step 3.2: Generate the Threshold Signature

This process allows the participants to collaboratively sign a transaction using the threshold key.

##### Step 3.2.1: Generate Signing Nonces and Commitments

Each participant runs:

```bash
cargo run threshold-sign-round1
```

This generates:

- **Secret signing nonces** (`signing_nonces.json`).
- **Public signing commitments** (`signing_commitments.json`).

##### Step 3.2.2: Exchange Signing Commitments

Participants share their `signing_commitments.json` files with each other.

##### Step 3.2.3: Aggregate Signing Commitments

Each participant aggregates each participant's `signing_commitments.json` files manually into `signing_commitments.json`.

##### Step 3.2.4: Generate Signing Packages

**Default Values:**

- **URL:** `wss://westend-rpc.polkadot.io`
- **Pallet:** `System`
- **Call Name:** `remark`
- **Call Data:** `((197, 38))`
- **Context:** `substrate`

**Note:** You can override these defaults using flags, e.g., `--url "custom_url"`.

Each participant runs:

```bash
cargo run threshold-sign-round2
```

This command generates the participant's **signing_package** (`signing_packages.json`).

##### Step 3.2.5: Exchange Signing Packages

Participants share their `signing_packages.json` files with each other.

##### Step 3.2.6: Aggregate Signing Packages

One participant (or each participant individually) manually aggregates the signing packages to create the final threshold signature.

**Command:**

```bash
cargo run aggregate-threshold-extrinsic
```

This produces the final threshold signature (`threshold_signature.json`), ready for submission.

#### Step 3.3: Submit the Threshold Signature

##### Step 3.3.1: Fund the Threshold Account

Ensure the threshold account (identified by `threshold_public_key.json`) has sufficient funds. You can use a faucet or transfer from existing accounts.

For example, on the Westend network, use the [Westend Faucet](https://matrix.to/#/#westend_faucet:matrix.org).

##### Step 3.3.2: Submit the Threshold Extrinsic

Run:

```bash
cargo run submit-threshold-extrinsic
```

This submits the threshold-signed extrinsic to the network.

## Summary of Steps for Each Participant

1. **Generate Key Pair:**

   ```bash
   subkey generate
   ```

2. **Create `recipients.json`:**

   Contains all participants' SS58 public keys.

3. **Create `contributor_secret_key.json`:**

   Contains your secret seed in hex format.

4. **Run Round 1 of Key Generation:**

   ```bash
   cargo run generate-threshold-public-key-round1 --threshold 2
   ```

5. **Share `message.json`:**

   Exchange with other participants.

6. **Aggregate Messages:**

   Create `all_messages.json` containing all participants' messages.

7. **Run Round 2 of Key Generation:**

   ```bash
   cargo run generate-threshold-public-key-round2
   ```

8. **Run Round 1 of Signing:**

   ```bash
   cargo run threshold-sign-round1
   ```

9. **Share `signing_commitments.json`:**

   Exchange with other participants.

10. **Aggregate Signing Commitments:**

    Create `all_signing_commitments.json`.

11. **Run Round 2 of Signing:**

    ```bash
    cargo run threshold-sign-round2
    ```

12. **Share `signing_packages.json`:**

    Exchange with other participants.

13. **Aggregate Signing Packages:**

    Run:

    ```bash
    cargo run aggregate-threshold-extrinsic
    ```

14. **Submit the Threshold Extrinsic:**

    ```bash
    cargo run submit-threshold-extrinsic
    ```


