## Build, Deploy and Test

Let's run the test once to see what happens.

First, install dependencies:

```
$ yarn
```

Next, we will build and deploy the program via Anchor.

Get the program ID:

```
$ anchor keys list
anchor_escrow: AGtT2X117M7Lx1PeXQrknorvwApEdBSUsAiYA2R2QESd
```

Here, make sure you update your program ID in `Anchor.toml` and `lib.rs`.

Build the program:

```
$ anchor build
```

Let's deploy the program. Notice that `anchor-escrow` will be deployed on a [mainnet-fork](https://github.com/DappioWonderland/solana) test validator run by Dappio:

```
$ solana config set --url https://rpc-mainnet-fork.dappio.xyz
...
```

```
$ solana config set --ws wss://rpc-mainnet-fork.dappio.xyz/ws
...
```

```
$ anchor deploy
...

Program Id: AGtT2X117M7Lx1PeXQrknorvwApEdBSUsAiYA2R2QESd

Deploy success
```

Finally, run the test:

```
$ anchor test --skip-build --skip-deploy
```
