<p align="center"><img src="brand/flu-contagious.svg" width="120" alt="Flu, the Contagion mascot"></p>

# Contagion ($FLU)

[![ci](https://github.com/space-rock/contagion/actions/workflows/ci.yml/badge.svg)](https://github.com/space-rock/contagion/actions/workflows/ci.yml)

## You can't buy $FLU. You have to catch it.

Contagion is a NEAR memecoin that spreads like a flu. There is no sale, no airdrop and no pool. The only way to get $FLU is to catch it from someone who already has it, so every holder got theirs from another holder. Each generation mints 40% less than the one before it, carriers who stop spreading lose tokens, and enough vaccinated accounts can end the whole outbreak.

Built for the NEAR Hackathon, tracks Memecoins With Mechanics and Games With Real Economies.

- Play it: [space-rock.github.io/contagion](https://space-rock.github.io/contagion/)
- Live testnet contract: [`contagion-live.testnet`](https://testnet.nearblocks.io/address/contagion-live.testnet) ([verified source](https://testnet.nearblocks.io/address/contagion-live.testnet/contract/code))
- The full rules: [RULES.md](RULES.md)

This is an unaudited demo that runs on testnet only. The NEAR and the FLU have no real value, so please don't put any into it.

## What this explores

Contagion explores whether on-chain rules can turn a memecoin into a social game. Spreading, recovery, vaccination and a reserve payout give each season its gameplay loop.

## How to play

1. Catch it. Open an invite link from a carrier, connect a testnet wallet and pay a small fee. The contract mints your first FLU into a locked balance. If you catch it straight from a patient zero, that is 600 FLU.
2. Spread it. Share your own invite link. Each person you catch earns you a 30% bonus on what they mint and restarts your clock. You can pass it on to 5 people.
3. Settle up. Recover before your clock runs out and 10% of your locked FLU burns. Miss the clock and anyone can cull you, which burns 50%.
4. Vaccinate and win. Burn FLU to make other accounts immune. When enough infected accounts are immune, the season ends and the spread points you earned pay out a share of a reserve.

## The rules

These are the values on the live testnet contract. Every one is set once when the contract starts and can be read with `get_config`.

| Rule | Value | What it means |
|---|---|---|
| Mint per generation | 600 FLU, then 60% of the one before | Patient zeros are generation 0 and mint 1,000. Their first catch mints 600, the next generation 360, and so on. |
| Carrier bonus | 30% | Extra FLU for the carrier on top of every mint they cause. |
| Fanout | 5 people | How many accounts one carrier can pass it on to. |
| Incubation clock | 21 days | Restarts with every catch. Run it out and you can be culled. |
| Recover in time | burn 10% | Settle before the clock ends and the rest becomes spendable. |
| Late or culled | burn 50% | The plague doctor who calls `cull` keeps 10% of that burn. |
| Vaccine | from 20 FLU | Burned to make one account immune. Each vaccination costs 0.1% more than the last. |
| Herd immunity | 30% | The season ends when this share of infected accounts is immune, from 1,000 infected. |
| Season | 30 days | The longest a season can run if herd immunity never comes. |
| Catch fee | 0.00005 NEAR per FLU minted | Never less than 0.01 NEAR, which pays for the storage a new account needs. The fee stays in the contract. |
| Reserve | 20% | Paid out to spread points after the season, as a share of everything the outbreak minted. |

The 72 hour clock and the higher fee of 0.0005 NEAR per FLU belong to the mainnet preset in `deploy/presets/mainnet.json`. The live testnet instance stretches the clock to 21 days so it stays playable.

## Gameplay

### Locked and spendable FLU

FLU you get from catching starts out locked. Your balance includes it, but only spendable FLU can be sent or used to vaccinate. Locked FLU turns spendable when you recover or get culled. That keeps anyone from selling a free position before they have settled.

Once it is spendable, FLU is a standard NEAR token. You can send it to a friend (their account needs a one-time storage deposit of about 0.003 NEAR first) or list it anywhere that supports NEAR tokens. The project doesn't deploy a market.

### Spread points and ranks

A catch earns 4 points for the carrier, 2 for the carrier's parent and 1 for its grandparent. Points set your place on the leaderboard and your share of the reserve. Ranks are only labels: Rookie, Spreader (8 points), Super-spreader (24) and Legend (60).

### Plague doctors

NEAR has no scheduler, so someone has to settle carriers whose clock ran out. Anyone can call `cull` on them and keep 10% of the burn as a reward. The rest is destroyed. If nobody culls an expired carrier, nothing breaks and their FLU just stays locked.

### Vaccines and the end of the season

The first vaccination costs 20 FLU and each one after it costs a little more, so a group racing to herd immunity has to pay more as it gets closer. A vaccinated account can never catch $FLU. Any account that hasn't caught it yet can be vaccinated, and nobody has to agree.

A season ends at 30% immunity among the infected (once at least 1,000 accounts were infected) or after 30 days, whichever comes first. After that, nobody can catch it or be vaccinated, every carrier counts as expired, and the reserve is shared out by spread points. Open your player card and press Claim. You can claim once.

### Cheating with many accounts

It partly works. Early places in the tree are scarce and one person can open many accounts. The catch fee follows the mint, so every account pays the same price per FLU and extra accounts are never cheaper. That is a limit and not a guarantee.

## The contract

The contract has no owner, admin, upgrade or withdraw method, and a test checks the wasm's exported functions against an allowlist. The rules can't change after launch and fees can't be taken out.

The mechanic lives in one pure Rust crate, `core/`. The contract, the test suite and the agent-based simulator in `sim/` all run that same code, so what [RULES.md](RULES.md) describes is what happens on chain. The live instance, `contagion-live.testnet`, was seeded with three patient zeros and was built reproducibly so explorers can verify its source.

## Run it

You need Rust (any recent stable), Node 22.18+ and pnpm. Nothing here touches anything outside the project folder.

```bash
scripts/setup-toolchain.sh   # project-local rustup with the wasm target and cargo-near (one time, several minutes)
scripts/test.sh              # every test, then builds the wasm and the web app
scripts/demo.sh              # the whole mechanic against a local NEAR sandbox, no keys, ~2 minutes
```

`scripts/demo.sh` starts a real sandbox node, deploys the contract, runs a 22 account outbreak, lets clocks expire, culls, vaccinates until herd immunity ends the season, pays out the reserve and prints the final tree.

The web app reads the live testnet contract:

```bash
cd web
pnpm install
pnpm dev        # http://localhost:5173
```

Open `http://localhost:5173/#/?from=a6.contagion-zero1.testnet` to see an invite page. Wallets connect through [near-connect](https://github.com/hot-dao/near-selector). On testnet it offers the wallets that declare testnet support, so MyNearWallet and HOT show up on mainnet only.

To look at the signed-in screens without a wallet, run `pnpm dev` and add `?as=<account>.testnet` to the URL (development builds only, and it can't sign anything). Add `#/outbreak?synthetic=10000` to load ten thousand fake accounts and see how the map copes.

You can also play from the NEAR CLI:

```bash
near contract call-function as-read-only contagion-live.testnet quote_catch \
  json-args '{"carrier":"a6.contagion-zero1.testnet"}' network-config testnet now

near contract call-function as-transaction contagion-live.testnet catch \
  json-args '{"carrier":"a6.contagion-zero1.testnet"}' prepaid-gas '60 Tgas' \
  attached-deposit '0.018 NEAR' sign-as YOU.testnet network-config testnet sign-with-keychain send
```

### Try it

| Do | See |
|---|---|
| Open the [live app](https://space-rock.github.io/contagion/) | The Play page with the season stats, the leaderboard and the latest on-chain events. |
| Open [`#/outbreak`](https://space-rock.github.io/contagion/#/outbreak) | The infection tree of the seeded outbreak and the plague doctor list. |
| Open [`#/?from=a6.contagion-zero1.testnet`](https://space-rock.github.io/contagion/#/?from=a6.contagion-zero1.testnet) | An invite card with the generation, the mint and the fee quoted by the contract. |
| Connect a testnet wallet there and press **Catch it** | Your player card with 360 FLU locked, a running clock and your own invite link. |
| Run `scripts/demo.sh` | A full season in a local sandbox. |
| Run `scripts/test.sh` | Every suite passes, then `all checks passed`. |

## Deploying

```bash
scripts/testnet-account.sh my-flu                  # faucet account, keys saved to .testnet/ (gitignored)
scripts/testnet-account.sh my-flu-zero1
scripts/deploy-testnet.sh live my-flu my-flu-zero1 # presets: live, demo
scripts/seed-testnet.sh my-flu my-flu-zero1        # grow a 24 host tree, 3 recoveries, 2 vaccinations
scripts/redeploy-testnet.sh my-flu                 # code-only update, state kept, needs the deployer key
scripts/burn-keys.sh my-flu                        # irreversible, asks you to type the name
```

The token icon stored in the contract is `brand/coin.svg`. `scripts/build-icon.py` embeds it as a base64 data URI in `contract/src/icon.txt`, and a contract test fails if the two ever differ. `scripts/redeploy-testnet.sh` rebuilds that, redeploys the code to an existing instance without calling `new`, and aborts unless the hosts, supply and event count are identical afterwards. It only works while the deployer key exists, so run it before `burn-keys.sh`.

To make the source verifiable on explorers, build reproducibly: commit, push, start Docker and run `scripts/build-reproducible.sh` (`contract/Cargo.toml` already names this repository). It refuses to run on a dirty tree or an unpushed commit, builds in the pinned `sourcescan/cargo-near` image and fills the commit link in `contract_source_metadata`. Deploy that wasm with `scripts/redeploy-testnet.sh my-flu --reproducible`. For a normal rebuild, `e2e/examples/compat.rs` loads the previous wasm's state under the new code and compares every view (`cargo run -p contagion-e2e --example compat -- previous.wasm`).

The `demo` preset has 3 minute clocks so you can show the whole cycle on camera. Its patient zero expires minutes after deployment by design, so deploy it right before recording.

A deployed contract can be replaced by whoever holds its account key, so the last step of a launch is `scripts/burn-keys.sh <contract>`. It deletes every access key and checks that none remain. All access keys of `contagion-live.testnet` have been burned, so its code can never change.

## Contract methods

| Method | Who | Notes |
|---|---|---|
| `new(config, patient_zeros)` | once | Validates the config and starts the season clock. |
| `catch(carrier)` | payable | Attach at least the fee from `quote_catch`. The excess is refunded. |
| `recover()`, `claim()` | host | |
| `cull(account)` | anyone | Register with `storage_deposit` first to receive the cut. |
| `vaccinate(account)` | payable | Attach the storage minimum if the target isn't registered. |
| `ft_*`, `ft_transfer_call` | NEP-141 | Receivers must be registered. Locked balance can't move. |
| `ft_metadata` | NEP-148 | The icon is `brand/coin.svg`. |
| `storage_deposit`, `storage_balance_of`, `storage_balance_bounds` | NEP-145 | Registration costs 0.003 NEAR. |
| `storage_withdraw`, `storage_unregister` | NEP-145 | Nothing is withdrawable, and registered accounts can't leave because they are nodes in the infection tree. `storage_unregister` returns `false` for an account that was never registered. |
| `contract_source_metadata` | NEP-330 | Version, the declared standards and, for a reproducible build, the source commit. |
| `get_config`, `get_stats`, `get_host`, `get_hosts`, `get_events`, `quote_catch` | views | `get_events` is an on-chain log, so the web app needs no indexer. |

Events follow NEP-297. Every balance change logs a NEP-141 event (`ft_mint`, `ft_burn`, `ft_transfer`), so an indexer that replays them gets `ft_balance_of` for every account. The game events `catch`, `recover`, `cull`, `vaccinate`, `claim` and `season_end` use the `contagion` standard. Plain transfers appear in the NEP-141 logs only. The on-chain `get_events` list skips them so nobody can grow contract storage with one-unit transfers.

## Standards and ABI

| Standard | Version | Where |
|---|---|---|
| NEP-141 fungible token | 1.0.0 | `FungibleTokenCore` and `FungibleTokenResolver` from `near-contract-standards` |
| NEP-145 storage management | 1.0.0 | `StorageManagement` |
| NEP-148 metadata | 1.0.0 | `FungibleTokenMetadataProvider` |
| NEP-297 events | 1.0.0 | `ft_*` through the standards crate, game events in the same envelope |
| NEP-330 source metadata | 1.3.0 | `contract_source_metadata`, declared standards listed |

`scripts/build-contract.sh` builds with `cargo-near` 0.23.0 (pinned by `scripts/setup-toolchain.sh`), embeds the ABI in the wasm (view `__contract_abi`) and writes `deploy/abi.json`. Two tests keep it honest: the ABI lists exactly the methods the wasm exports, and the embedded ABI equals `deploy/abi.json`.

## Built with

- Contract: [near-sdk](https://github.com/near/near-sdk-rs) and near-contract-standards 5.30, built with cargo-near 0.23.
- Tests and tools: near-workspaces (sandbox and testnet), serde, borsh, schemars.
- Web: React 19, React Router, Vite, Vitest, near-api-js, [near-connect](https://github.com/hot-dao/near-selector), d3 (hierarchy, quadtree, zoom), lucide-react and the Fredoka and Plus Jakarta Sans fonts.
- AI: the code, tests and documentation were written with AI coding assistance (Claude Code) under the author's direction and review.

## License

MIT, see [LICENSE](LICENSE).

## Other NEAR Hackathon submission

[Tilll](https://github.com/space-rock/tilll) is a self-hosted crypto checkout on NEAR Intents.
