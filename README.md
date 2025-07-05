# Bio-DID-Seq: Decentralized Identifiers for Biological Research Data

Bio-DID-Seq is a GDPR-compliant Decentralized Identifier (DID) system designed as a sustainable, user centric addition to existing persistent identifier infrastructures like DOI and Handle. The system integrates with Dataverse and is demonstrated through AI applications to showcase decentralized metadata management and user empowerment in cultural heritage and biological research data contexts.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Project Overview

Bio-DID-Seq provides a solution for managing research data identifiers through decentralized technology like IPFS, DIDs and UCAN. It addresses key challenges in the scientific data management field:

- **User Data Control**: Empowering researchers with control over their metadata
- **GDPR Compliance**: Built in privacy protections aligned with European regulations
- **Cost Efficiency**: Reducing expenses for large scale identifier creation
- **Accessibility**: Enhancing data discoverability and community engagement
- **Interoperability**: Seamless integration with existing identifier systems

### Key Features

- **W3C DID Compliance**: Implementation of W3C Decentralized Identifier standards
- **UCAN Integration**: Authorization using User Controlled Authorization Networks
- **IPFS Storage**: Decentralized content-addressed storage for research data
- **BioAgents Integration**: AI agents for enhanced biological data processing
- **Harvard Dataverse Integration**: Compatibility with the Dataverse repository infrastructure

## Architecture

Bio-DID-Seq implements a layered architecture combining several technologies:

1. **Storage Layer**: IPFS based decentralized storage for data persistence
2. **Identity Layer**: W3C DID implementation for decentralized identity management
3. **Authorization Layer**: UCAN for user controlled access management
4. **Application Layer**: Actix Web API for service endpoints
5. **Integration Layer**: Connectors for BioAgents and Dataverse

### Components

- **IPFS Cluster**: Provides redundant, decentralized storage
- **DID Manager**: Creates and manages W3C-compliant DIDs
- **UCAN Auth**: Handles authorization using capability-based security
- **BioAgents Connector**: Integrates with AI-powered biological data processing
- **Dataverse Adapter**: Connects with Harvard Dataverse for data publishing

## Prerequisites

Ensure you have the following installed:

- [Rust](https://www.rust-lang.org/) (1.84.1 or later)
- [Cargo](https://doc.rust-lang.org/cargo/) (included with Rust)
- [Docker](https://www.docker.com/) and [Docker Compose](https://docs.docker.com/compose/)
- [IPFS](https://docs.ipfs.io/install/) (if running outside Docker)

## Getting Started

### Installation

1. Clone the repository:
```sh
git clone https://github.com/did-seq/bio-did-seq
cd bio-did-seq
```

2. Configure your environment:
```sh
cp .env.example .env
# Edit .env with your settings
```

3. Generate cryptographic keys:
```sh
cargo run -- generate-keys --output ./keys
```

4. Start the services using Docker Compose:
```sh
docker-compose up -d
```

### Configuration

The system is configured through environment variables in the `.env` file:

```
DATABASE_URL=mysql://user:password@localhost:3306/bio_did_seq
IPFS_NODE=http://127.0.0.1:5001
BIND_ADDRESS=127.0.0.1:8081
MAX_CONCURRENT_UPLOADS=20
RUST_LOG=info
DILITHIUM_PUBLIC_KEY=path/to/dilithium5_public.key
DILITHIUM_PRIVATE_KEY=path/to/dilithium5_secret.key
BIOAGENTS_API_URL=http://localhost:3000
DATAVERSE_API_URL=https://dataverse.harvard.edu/api
DATAVERSE_API_KEY=your_api_key
```

## API Documentation

### Core Endpoints

All endpoints are prefixed with `/api`:

- **POST** `/api/signup` - Register a new user
- **POST** `/api/signin` - Authenticate a user and receive a token
- **POST** `/api/did/create` - Create a new DID for research data
- **GET** `/api/did/{id}` - Retrieve a DID document
- **PUT** `/api/did/{id}` - Update a DID document (requires authorization)
- **POST** `/api/upload` - Upload research data (requires authorization)
- **GET** `/api/download/{cid}` - Download research data
- **POST** `/api/bioagent/process` - Process data using BioAgents
- **POST** `/api/dataverse/publish` - Publish data to Dataverse

### BioAgents Integration

Bio DID-Seq integrates with BioAgents for AI powered analysis of biological data:

- Automatic metadata extraction from research papers
- Semantic linking between related datasets
- Generation of knowledge graphs from unstructured data
- Enhanced search capabilities across biological datasets

### UCAN Authorization

The system uses UCAN (User Controlled Authorization Network) for decentralized authorization:

1. Users control who can access their data through capability based tokens
2. Fine grained permissions for specific operations
3. Delegated authorization without central authority
4. Cryptographic verification of access rights

## Deployment Options

### Local Development
```sh
cargo run
```

### Production Deployment
```sh
cargo build --release
./target/release/bio-did-seq
```

### Docker Deployment
```sh
docker-compose up -d --build
```

## Integrating with Dataverse

Bio-DID-Seq integrates with Harvard Dataverse through its API:

1. Create DIDs that reference Dataverse datasets
2. Publish datasets to Dataverse while maintaining DID linkage
3. Synchronize metadata between DID documents and Dataverse entries

## Contributing

We welcome contributions to Bio-DID-Seq! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- W3C DID and Verifiable Credentials Working Groups
- UCAN Community
- IPFS and Filecoin Teams
- Harvard Dataverse Project
- BioAgents Contributors
