# Bio-DID-Seq: Decentralized Identifiers for Biological Research Data

Bio-DID-Seq is a GDPR compliant Decentralized Identifier (DID) system designed for research data, integrating with Dataverse and powered by AI agents to showcase decentralized metadata management and user empowerment in cultural heritage and biological research data contexts.

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

## Cost Reduction Numbers for Bio-DID-Seq Identifier Creation 

- **Decentralized Storage**: Centralized cloud storage (e.g., AWS S3) for large scale research data can cost $0.023-$0.10 per GB/month for storage and $0.09-$0.12 per GB for data retrieval. Bio-DID-Seq uses IPFS for decentralized storage, reducing costs to near $0 for storage if hosted on community nodes, or ~$0.005-$0.01 per GB/month on paid pinning services (e.g., Pinata, Filebase, Publiish etc). This can lead to 70-90% savings for storing identifier metadata (e.g., DID documents, typically <1 KB each). Example: For 1 million DID documents (~1 GB total), IPFS storage costs ~$0.01/month vs. $23-$100/month on AWS S3.

- **Automation via AI (BioAgents)**: Manual creation of identifiers and metadata for research datasets can cost $20-$50 per hour for skilled labor, with an average of 1-2 minutes per identifier. For 1 million identifiers, this translates to ~16,667-33,000 hours or $333,340-$1,666,650 in labor costs. Bio-DID-Seq’s BioAgents automate metadata extraction and DID creation, reducing processing time to seconds per identifier. Assuming a server cost of $0.10-$1.50 per hour for AI processing, generating 1 million identifiers might cost ~$100-$500, a 99.9% cost reduction.

- **DID Creation Efficiency**: Traditional Identifier Systems: Proprietary systems like DOIs (Digital Object Identifiers) charge $0.01-$1 per identifier for creation and maintenance (e.g., DataCite pricing). For 1 million identifiers, this costs $10,000-$300,000. W3C compliant DIDs in Bio-DID-Seq are generated using Rust based DID Manager, with negligible computational costs (~$0.001 per 1,000 DIDs on a standard server). For 1 million DIDs, total cost is ~$1-$10, a 99.9% reduction compared to DOI systems. Decentralized DIDs require no annual fees (unlike DOIs), saving $0.50-$5 per identifier annually.

- **UCAN Authorization**: Bio-DID-Seq’s UCAN based authorization eliminates central server costs, relying on cryptographic tokens managed client side. Setup costs are minimal and operational costs are near $0, yielding 90-100% savings for access management. Publishing datasets to Dataverse manually involves labor costs of $18-$50/hour for metadata entry and validation, with 10-30 minutes per dataset. For 10,000 datasets, this costs $33,000-$100,000.

- **Bio-DID-Seq Automation**: The Dataverse Adapter automates metadata synchronization and publishing, reducing time to seconds per dataset. Assuming API call costs of $0.001-$0.01 per dataset, publishing 10,000 datasets costs $10-$100, a 99% cost reduction.

### Components

- **IPFS Cluster**: Provides redundant, decentralized storage
- **DID Manager**: Creates and manages W3C compliant DIDs
- **UCAN Auth**: Handles authorization using capability-based security
- **BioAgents Connector**: Integrates with AI powered biological data processing
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
git clone https://github.com/publiish/bio-did-seq
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
- UCAN Working Group
- IPFS and Filecoin Teams
- Harvard Dataverse Project
- BioAgents Contributors
