# Submission kit

Everything needed for the NEAR HACKATHON form, plus the steps that only you can do. A submission cannot be edited directly afterward (changes go through a request the organizers review by hand, and the same repo cannot be submitted twice), so go through the checklist first.

Deadline: 14 Nov 2026, 23:59 UTC. Aim for the morning of the 14th at the latest.

## Form fields

| Field | Value |
|---|---|
| Project name | Contagion ($FLU): a memecoin you can't buy, only catch |
| Tracks | Memecoins With Mechanics (`memecoins`), Games With Real Economies (`games`), Open Innovation (`open`) |
| Repository link | https://github.com/space-rock/contagion |
| Demo post link | your X post URL (see below) |
| Website (optional) | https://space-rock.github.io/contagion/ |
| NEAR wallet address | a **mainnet** account that should receive the prize |
| Team | your name, email and size |

Track choice: the mechanic is the memecoin, the outbreak with its vaccines and plague doctors is a game loop, and `open` can be combined with any track. I left out Social Money because the project does not build on Sputnik DAO or Trezu, and the judges check the track prompts.

## Project description (paste into the form, 1,983 characters; the limit is 2,000)

```
You can't buy $FLU. You have to catch it. Contagion is a NEAR memecoin whose only supply is the outbreak: every holder received it from another holder by calling catch(carrier) and paying a fee proportional to what they mint. Rewards shrink 40% each generation, the carrier earns a bonus, and every carrier has an incubation clock. Let it run out and anyone can cull you: half of your locked tokens burn and the caller keeps a cut. Recover in time and you burn only 10%. Recovered holders can burn tokens to vaccinate other accounts, and when 30% of infected accounts are immune the season ends and spread points pay out a reserve.

The mechanic lives in the contract, not a backend. All parameters are set once at launch and readable on chain, and there is no owner, admin, upgrade or withdraw method (a test checks the wasm's exports). It implements NEP-141, 145, 148, 297 and 330 on near-contract-standards and ships its ABI, and a test replays every NEP-141 log to prove they add up to each balance on chain. Infection mints land in a locked balance that cannot be transferred, so nobody dumps a free position.

One pure Rust state machine, core::apply, runs in the contract, the 103 Rust tests and the agent-based simulator. The simulator ran 20,000 simulated people over five scenarios and changed the design three times: a fixed reserve became a share of infection mint, a flat fee that let sybil accounts buy tokens at a tenth of the honest price became a fee proportional to the mint, and a flat vaccine price that let a faction end 55% of seasons became a rising one. The README lists what still breaks (sybil accounts still take about a quarter of the mint at the posted price, a vaccine faction still ends 1 season in 20).

The repo includes a live testnet deployment with a seeded outbreak, a web app (Flu mascot, player card, outbreak map that stays smooth at 10,000 accounts, plague doctor list, on-chain event log), and a 2 minute local sandbox demo: scripts/demo.sh.
```

## X demo post

Open with the problem, show one full action, say what NEAR does, end with links and a question. Attach the video.

```
Memecoins are a price chart with a logo. What if the coin had rules?

$FLU can't be bought. You catch it from a holder, they earn a bonus, and if you don't pass it on before your clock runs out, anyone can cull you and burn half your stack.

Built at NEAR HACKATHON. The whole mechanic lives in a NEAR contract with no admin. 103 Rust tests, 20k simulated people.

Demo: https://space-rock.github.io/contagion/
Code: https://github.com/space-rock/contagion

Would you rather vaccinate or spread? @NEARProtocol #NEARHackathon
```

### 60 second video, in order

1. Open the live page. "You can't buy $FLU. You have to catch it." Scroll the outbreak map (29 accounts, three roots).
2. Open an invite link, connect a wallet, press **Catch it**. Show the new dot appear and the 600 FLU locked balance.
3. Show the plague doctor table, then cull an expired carrier (use a fresh `demo` deployment for this, its clocks are 3 minutes).
4. Cut to `scripts/demo.sh` output: the culling, the vaccination ending the season, the reserve payout, the final tree.
5. End on the How to play rules (read from the contract) and the README table of what broke.

## Checklist (things only you can do)

- [ ] Run `scripts/test.sh` on a clean checkout. It runs every suite, builds the wasm and the web app.
- [ ] Push to https://github.com/space-rock/contagion (the `origin` remote is set) and make sure the repository is **public**. The LICENSE (MIT) and README are already in place.
- [ ] Enable GitHub Pages with source "GitHub Actions". The `pages` workflow builds `web/` and publishes it.
- [x] Verified source: the live contract was rebuilt reproducibly from commit `5aabf57`, redeployed (state kept) and verified on 12 Nov 2026: https://testnet.nearblocks.io/address/contagion-live.testnet/contract/code. If the contract source changes again, repeat `scripts/redeploy-testnet.sh contagion-live --reproducible` (on Apple silicon, turn on Rosetta emulation in Docker Desktop first) and verify again. Do it before burning the keys.
- [ ] Decide on immutability. The testnet instances were deployed with the deployer key still present so you could test. When you are happy, run `scripts/burn-keys.sh contagion-live` (irreversible) and set `keys_burned` to `true` in `deploy/testnet.json`. If you find a bug first, run `scripts/redeploy-testnet.sh contagion-live` to update the code in place (state is kept and checked), or deploy new accounts with `scripts/deploy-testnet.sh` and update `web/.env.production`. The new Flu coin is already the on-chain token icon.
- [ ] Redeploy the demo instance right before recording: `scripts/testnet-account.sh <name>` for a contract and a patient zero, then `scripts/deploy-testnet.sh demo <contract> <zero>` and `scripts/seed-testnet.sh <contract> <zero>`. Use `?c=<contract>` on the web app to point at it.
- [x] Wallet flow tested end to end through near-connect with Intear Wallet on 12 Nov 2026, every step approved in the wallet popup and confirmed on chain: connect and `catch` on `contagion-live.testnet`; `storage_deposit` + `cull` on `flu-demo-nov12.testnet`; `catch`, `recover` in time (10% burn), six `vaccinate` calls at a rising price (20.04 to 20.14 FLU) that ended the season by herd immunity, and `claim`, on `flu-demo2-nov12.testnet` and again on `flu-demo3-nov12.testnet` for the video; a late `recover` after the clock ran out (50% burn, `expired: true`) on `flu-demo4-nov12.testnet`. Not tried: Meteor (its web wallet refuses automated browsers) and the other listed wallets.
- [ ] The Pages build uses https://space-rock.github.io/contagion for the social card image. Only if the site ends up at another address (a custom domain), set a repository variable `SITE_URL` to it before the Pages build.
- [x] Contract code redeployed to `contagion-live.testnet` on 12 Nov 2026 (state kept, hashes in `deploy/testnet.json` match the chain). If the contract source changes again, run `scripts/redeploy-testnet.sh contagion-live` and update the hashes.
- [ ] Keep the Pages site and the testnet instance up until at least 25 Nov 2026 (judging).
- [ ] Post the video on X with the tags above and copy the post URL. The video is recorded: `media/contagion-demo.mp4` (98 seconds, 1280x720, not committed; X allows 140 seconds). Every transaction in it is a real wallet approval on testnet: connect and `catch` on the live contract, `cull`, `recover`, `vaccinate` up to herd immunity and `claim` on demo instances. It also shows the verified source and ABI methods on NearBlocks, and the rules page.
- [ ] One submission per project: if you also enter a second project, each README must name the other.
- [ ] Submit the form with the repo link, the post link and a mainnet payout account.

## Known gaps to be honest about in the post

- Testnet only, unaudited.
- Sybil accounts still take about a quarter of the mint; they pay the posted price for it.
- The simulator's agents do not react to prices.
