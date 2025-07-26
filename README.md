# Colosseum

Colosseum is a 3D physics simulation platform built with Rust, utilizing [kiss3d](https://github.com/sebcrozet/kiss3d) for real-time 3D rendering and [rapier3d](https://rapier.rs/) for physics simulation. The project also integrates [Zenoh](https://zenoh.io/) for distributed command and control, enabling remote interaction with the simulation.

## Features

- Real-time 3D visualization using kiss3d
- Physics simulation powered by rapier3d
- Remote command execution via Zenoh
- Modular architecture for adding new simulation objects and commands

## Getting Started

### Prerequisites

- Rust (edition 2021)
- Cargos
- A recent version of Zenoh (for distributed communication)

### Build and Run

```sh
cargo run
```
This will launch the simulation window and initialize the physics environment

## Usage
- The simulation starts with a ground plane and simulation objects.
- Remote commands can be sent via Zenoh to interact with simulation objects (e.g., applying force to the ball).