# ERC20 on Casper
Implementation of the ERC20 standard on the Casper Network.

## Testing and Development

### 1. Clone the Project

```bash

$ git clone https://github.com/FriendlyMarket/casper-erc20.git

$ cd casper-erc20

```

### 2. Install the Compiler

```bash

$ rustup toolchain install nightly

```

### 3. Install the Compilation Target

Make sure `wasm32-unknown-unknown` is installed.

```bash

$ make prepare

```

### 4. Install wasm-strip

`wasm-strip` helps reduce the compiled wasm contract's size. It can be found in the `wabt` package.

```bash

$ sudo apt-get install wabt

```

### 5. Build Contract

```bash

$ make build-contract

```

### 6. Test Contract Locally

Test logic and smart contracts.

```bash

$ make test

```
## Credits
[Casper Fungible Tokens (ERC-20 Standard)](https://github.com/casper-ecosystem/erc20)
