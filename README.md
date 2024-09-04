# Blockfrost Platform

`blockfrost-platform` is a software that serves the Blockfrost API within the Blockfrost network. It is designed to decentralize Blockfrost by allowing Stake Pool Operators (SPOs) and other node operators to run Blockfrost instances, which will handle customer traffic.

## Key Features

- **Decentralization:** Enables the decentralization of Blockfrost services by distributing traffic across multiple instances.
- **SPO Engagement:** Involves SPOs and other node operators in the process, enhancing the robustness of the network.
- **Rust-based CLI:** Built with Rust, offering performance and security.

## Prerequisites

- A running Cardano node with a socket available.
- Rust environment for building from source (if not using pre-built binaries).

## Installation

To install `blockfrost-platform`, you can download the pre-built binaries from the [releases page](#) or build from source:

```bash
# Clone the repository
git clone https://github.com/blockfrost/blockfrost-instance.git

# Navigate to the project directory
cd blockfrost-instance

# Build the project
cargo build --release
