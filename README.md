# UniBlockchain

An educational blockchain implementation for managing student records, built with 🦀 **Rust**.
![Untitled](https://github.com/user-attachments/assets/33bd36ab-3680-4cd1-bf1b-e04009c5e8d3)

---

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Generating RSA Keys](#generating-rsa-keys)
- [Usage](#usage)
  - [Running the Authority Node](#running-the-authority-node)
  - [Running a Non-Authority Node](#running-a-non-authority-node)
- [Commands](#commands)
- [Configuration](#configuration)
- [Dependencies](#dependencies)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

---

## Introduction

**UniBlockchain** is a decentralized application designed to securely manage and store student academic records using blockchain technology. Developed in Rust, it leverages the language's performance and safety features to ensure a robust and secure system.

---

## Features

- 🔒 **Secure Student Record Management**: Safely store and retrieve student information, academic periods, courses, and grades.
- 🌐 **Decentralized Network**: Nodes communicate using the libp2p library, ensuring data consistency across the network.
- 🛡️ **Blockchain Integrity**: Blocks are signed and verified using RSA cryptography, maintaining the integrity of the chain.
- 🎓 **Authority and Non-Authority Nodes**: Different roles within the network, with authority nodes capable of creating and signing new blocks.
- ⚡ **High Performance**: Built with Rust for maximum efficiency and reliability.

---

## Getting Started

### Prerequisites

Ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021 or later)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [OpenSSL](https://www.openssl.org/) (for generating RSA keys)
- Access to a terminal or command-line interface

### Installation

Clone the repository:

```bash
git clone https://github.com/eduardozborowski/uniblockchain.git
cd uniblockchain
```

### Build the project:

bash
Copy code
cargo build
Generating RSA Keys
To participate as an authority node, you need to generate an RSA key pair. Follow these steps using OpenSSL:

### Generate a 2048-bit Private Key:

```bash
openssl genpkey -algorithm RSA -out chaves_privadas/autoridade_1.pem -pkeyopt rsa_keygen_bits:2048
```
Extract the Public Key:

```bash
openssl rsa -pubout -in chaves_privadas/autoridade_1.pem -out chaves_publicas/autoridade_1.pem
```
Copy the Public Key into config.toml:

Open chaves_publicas/autoridade_1.pem and copy its contents into the config.toml file as shown below.

### Commands
Once the application is running, you can interact using the following commands:

transacao: Create a new transaction (student record).
criar_bloco: (Authority only) Create a new block with pending transactions.
exibir_blockchain: Display the current state of the blockchain.

### Dependencies
The project relies on several Rust crates to function properly. Below is the list of dependencies along with brief explanations:
- libp2p: A modular networking stack for peer-to-peer applications. Used for node communication.
- rsa: Provides RSA encryption and decryption functionalities for signing and verifying blocks.
- chrono: Date and time library for Rust, used for timestamps.
- serde and serde_json: Frameworks for serializing and deserializing Rust data structures.
- sha2: Cryptographic hashing algorithms (SHA-2) for hashing blocks.
- rand: Generates random numbers, used in key generation and other randomness needs.
- futures: Asynchronous programming support.
- tokio: An asynchronous runtime for Rust, enabling concurrency.
- base64: Encoding and decoding of base64 data.
- thiserror: Simplifies error handling by deriving standard error implementations.
- parking_lot: Provides faster and more efficient implementations of synchronization primitives.
- config: Layered configuration system for Rust applications.
- signature: Trait definitions for cryptographic signatures.
- toml: Parser and encoder for TOML configuration files.


### Acknowledgements
We would like to extend our heartfelt thanks to:

-  💻 The Web3 Community: For pioneering decentralized technologies and inspiring projects like this.
-  🦀 The Brazilian Rust Community: For their support, knowledge sharing, and contributions to the Rust ecosystem.
-  � High Order Company: For inspiring me in the beginning of this journey. Obrigado Savio e Bonatto.
