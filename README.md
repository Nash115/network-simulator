# network-simulator

<p align="center">
    <a href="https://github.com/Nash115/network-simulator" alt="Github Repo">
        <img src="https://img.shields.io/badge/github-repo-blue?logo=github&logoColor=white" />
    </a>
    <a href="https://rust-lang.org" alt="Rust Language">
        <img src="https://img.shields.io/badge/Rust-orange?logo=Rust&logoColor=white" />
    </a>
    <a href="https://www.docker.com" alt="Docker">
        <img src="https://img.shields.io/badge/Docker-blue?logo=Docker&logoColor=white" />
    </a>
</p>

## About

A simple network simulator written in Rust. It allows users to create virtual networks, and simulate interactions between devices.

## Launching the Simulator

### 1. Using Docker

Ensure you have Docker installed. Then clone the repository and build the Docker image:

```bash
docker build -t network-simulator .
```

Then run the container:

```bash
docker run -it --rm network-simulator
```

### 2. Building from Source

Ensure you have Rust installed. Then clone the repository and build the project:

```bash
cargo build --release
```

Run the simulator:

```bash
./target/release/network-simulator
```

---

<p align="center">
    Made with ❤️ by Nash115
</p>
