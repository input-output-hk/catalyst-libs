---
icon: material/server-network
---

# Deployment View

<!-- See: https://docs.arc42.org/section-7/ -->

## Infrastructure Level 1: Library Distribution

**Overview**:  
Catalyst-libs libraries are distributed as reusable components that applications integrate into their own deployment infrastructure.

**Motivation**:  
Libraries are not deployed as standalone services but are integrated into applications. This section describes how libraries are distributed and how applications deploy them.

**Quality and Performance Features**:
- Libraries are lightweight and have minimal runtime dependencies
- Can be statically linked or used as dynamic libraries
- WebAssembly builds for web deployment
- Cross-platform support

**Mapping of Building Blocks to Infrastructure**:
- Rust crates: Distributed via git tags or (potentially) crates.io
- Flutter/Dart packages: Distributed via pub.dev or git
- Python bindings: Distributed via PyPI or git
- Documentation: Hosted on GitHub Pages

## Infrastructure Level 2: Distribution Channels

### Rust Crates Distribution

**Infrastructure Element**: Git-based Distribution

**Description**:  
Rust crates are distributed via git tags in the catalyst-libs repository. Applications reference specific versions using git dependencies.

**Deployment**:
- Each crate is versioned independently
- Git tags follow format: `crate-name/vX.Y.Z`
- Applications specify versions in `Cargo.toml`:
  ```toml
  cardano-chain-follower = { 
    version = "0.0.19", 
    git = "https://github.com/input-output-hk/catalyst-libs.git", 
    tag = "cardano-chain-follower/v0.0.19" 
  }
  ```

**Alternative**: Potential future distribution via crates.io

**Requirements**:
- Git repository access
- Tag-based versioning
- Cargo dependency resolution

### Flutter/Dart Packages Distribution

**Infrastructure Element**: Pub Package Distribution

**Description**:  
Flutter/Dart packages can be distributed via pub.dev or directly from git.

**Deployment**:
- Packages defined in `pubspec.yaml`
- Can be published to pub.dev
- Or referenced from git repository
- Versioned independently

**Requirements**:
- Dart/Flutter SDK
- Pub package manager
- Git access (if not using pub.dev)

### Python Bindings Distribution

**Infrastructure Element**: Python Package Distribution

**Description**:  
Python bindings via catalyst-python package, distributed via PyPI or git.

**Deployment**:
- Python package with FFI bindings
- Can be installed via pip
- Or built from source

**Requirements**:
- Python 3.x
- Rust toolchain (for building from source)
- FFI compatibility

### WebAssembly Distribution

**Infrastructure Element**: WebAssembly Builds

**Description**:  
Rust crates can be compiled to WebAssembly for web applications.

**Deployment**:
- Compile crates to wasm32 target
- Bundle with web application
- Use from JavaScript/TypeScript

**Requirements**:
- wasm32-unknown-unknown target
- wasm-bindgen or similar
- Web application build system

## Infrastructure Level 3: Application Deployment

### cat-gateway Service Deployment

**Infrastructure Element**: Docker Container Deployment

**Description**:  
The cat-gateway service (in catalyst-voices repository) uses catalyst-libs crates and is deployed as a Docker container.

**Deployment Architecture**:
- **Container**: cat-gateway Docker image
- **Dependencies**: 
  - PostgreSQL (event-db)
  - ScyllaDB (index-db)
  - Cardano node access
- **Configuration**: Environment variables
- **Networking**: Exposes HTTP API

**catalyst-libs Integration**:
- Uses cardano-chain-follower for blockchain sync
- Uses rbac-registration for RBAC handling
- Uses catalyst-signed-doc for document processing
- Uses c509-certificate for certificate validation

**Deployment Files**:
- `docker-compose.yml`: Local development
- `Earthfile`: Build configuration
- `blueprint.cue`: Deployment configuration

### Catalyst Voices Application Deployment

**Infrastructure Element**: Multi-Platform Application

**Description**:  
Catalyst Voices Flutter application uses catalyst-libs packages and deploys to multiple platforms.

**Deployment Targets**:
- **Web**: Static hosting or web server
- **iOS**: App Store distribution
- **Android**: Google Play or APK distribution
- **Desktop**: Platform-specific installers

**catalyst-libs Integration**:
- Uses Flutter/Dart packages
- May use Rust crates via FFI
- Shares types and protocols

### Standalone Library Usage

**Infrastructure Element**: Direct Integration

**Description**:  
Applications can directly integrate catalyst-libs crates without additional infrastructure.

**Deployment**:
- Add dependencies to application's `Cargo.toml` or `pubspec.yaml`
- Build with application
- No separate deployment needed

**Examples**:
- CLI tools using catalyst-libs
- Other Rust applications
- Integration into existing systems

## Network Requirements

### Cardano Node Access

**Requirement**:  
Applications using blockchain-related crates need access to Cardano nodes.

**Options**:
- Direct node connection (Node-to-Node protocol)
- Mithril snapshot access
- Third-party API (if available)

**Networks**:
- Mainnet: Production Cardano network
- Preprod: Pre-production test network
- Preview: Preview test network

### IPFS Network Access

**Requirement**:  
Applications using hermes-ipfs need IPFS network access.

**Options**:
- Local IPFS node
- IPFS gateway
- IPFS network participation

## Build and Deployment Tools

### Earthly

**Purpose**: Reproducible builds

**Usage**: Build Docker images, run tests, generate documentation

### Cargo

**Purpose**: Rust package management and building

**Usage**: Build crates, run tests, manage dependencies

### MkDocs

**Purpose**: Documentation generation

**Usage**: Build documentation site, deploy to GitHub Pages

## Version Compatibility

### Rust Version

- **Required**: Rust edition 2024
- **Toolchain**: Specified in `rust-toolchain.toml`

### Cardano Protocol

- **Support**: Multiple protocol versions
- **Compatibility**: Handled by Pallas library

### Platform Support

- **Linux**: Full support
- **macOS**: Full support
- **Windows**: Full support
- **WebAssembly**: Partial support (wasm32 target)
