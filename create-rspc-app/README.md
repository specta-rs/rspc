<p align="center">
<img width="75" height="75" src="https://raw.githubusercontent.com/oscartbeaumont/rspc/main/docs/public/logo.png" alt="Logo">
<h1 align="center">create-rspc-app</h1>
</p>


<div align="center">
  <a href="https://discord.gg/4V9M5sksw8"><img src="https://img.shields.io/discord/1011665225809924136?style=flat-square" alt="Discord"></a>
  <a href="https://crates.io/crates/create-rspc-app">
    <img src="https://img.shields.io/crates/v/create-rspc-app.svg?style=flat-square"
    alt="crates.io" />
  </a>
  <a href="https://crates.io/crates/create-rspc-app">
    <img src="https://img.shields.io/crates/d/create-rspc-app.svg?style=flat-square"
      alt="download count badge" />
  </a>

  <strong><a href="https://rspc.dev">Website</a></strong>
</div>

<p align="center">Interactive CLI to create a new app using rspc.</p>
<p align="center">
<strong>
Get started by running `cargo install create-rspc-app` and then `create-rspc-app` in your terminal.
</strong>
</p>


## Stack

The stack is based on the following technologies:
- Languages
  - [Rust](https://www.rust-lang.org/)
  - [TypeScript](https://www.typescriptlang.org/)
- Frameworks
  - [React](https://reactjs.org/)
  - [Solid](https://www.solidjs.com/)
- Database ORMs
  - [prisma-client-rust](https://prisma.brendonovich.dev/introduction)
- [RSPC](https://rspc.dev)

## Getting Started

```sh
# Install the CLI
cargo install create-rspc-app
```

```sh
# Create a new app
create-rspc-app
```

Additionally you can `create-rspc-app <my-directory>` to create a new app in a specific directory.

`create-rspc-app` will ask for the following information:
- `Name`: The name of your app.
- `Backend Framwork (Tauri, Axum)`: The backend framework you want to use.
- `Database ORM (Prisma, None):` The database ORM you want to use.
- `Frontend Framework (React, Solid)`: The frontend framework you want to use.
- `Package Manager (npm, yarn, pnpm)`: The package manager you want to use.

If you use prisma then create-rspc-app will automatically run `prisma generate`

## Issues

If you find any issue with the CLI please open an issue in the [rspc issues](https://github.com/oscartbeaumont/rspc/issues).