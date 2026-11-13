# Contagion rules

You can't buy $FLU. You have to catch it.

This file is the spec. `core/` implements it line by line and the contract and the simulator both run `core/`, so what is written here is what runs on chain.

## Accounts

Every account is in exactly one state.

| State | Meaning |
|---|---|
| `uninfected` | Never caught it. The default for every account. |
| `contagious` | Caught it, can still spread it, owns locked `viral` tokens. |
| `recovered` | Done spreading. Tokens are plain `liquid` tokens. |
| `immune` | Vaccinated before catching it. Can never be caught. |

Transitions: `uninfected -> contagious -> recovered`, or `uninfected -> immune`. Nothing moves backwards.

Each account has two balances. `ft_balance_of` returns `liquid + viral`, but `ft_transfer` can only spend `liquid`. Infection mints land in `viral`.

## Actions

### catch(carrier), called by the target, with the fee attached

Valid when all of these hold:

- the season is open
- the target is `uninfected` and is not the carrier
- the target is a named account (64-hex and `0x` implicit accounts are rejected)
- the carrier is `contagious` and its incubation clock has not expired
- the carrier has fewer than `fanout` children
- the attached deposit covers the required fee, `max(min_fee, target_mint * fee_per_unit)` (the excess is refunded)
- the target's mint is above zero

Effects:

1. target becomes `contagious` at `gen = carrier.gen + 1`, with `parent = carrier`
2. target mint = `carrier.mint * decay_bps / 10000`, credited to the target's `viral`
3. carrier bonus = `target_mint * bonus_bps / 10000`, credited to the carrier's `viral`
4. the carrier's incubation clock restarts, `children += 1`
5. spread points: carrier +4, its parent +2, its grandparent +1

The carrier does not sign anything. It shares a link (`/?from=alice.testnet`) and the target does the call.

Patient zeros (3 accounts named at `new()`) are `contagious` at `gen = 0` with zero balance and `mint = base`.

### recover(), called by a carrier

Settles the caller's `viral`:

- if the caller's clock has not expired: burn `burn_voluntary_bps` of `viral`
- if the clock has expired, or the season is over: burn `burn_expired_bps` (no discount for acting late, and no plague doctor cut)

The rest of `viral` moves to `liquid`. The caller becomes `recovered`.

### cull(account), called by anyone

Valid when `account` is `contagious` and expired (clock ran out, or the season is over). NEAR has no cron, so this is how expired carriers get settled.

- burn `burn_expired_bps` of the account's `viral`
- the caller (the plague doctor) receives `cull_cut_bps` of that burn as `liquid`, transferred out of the burned amount, not newly minted
- the remainder of the burn is destroyed
- the rest of `viral` moves to `liquid`, and the account becomes `recovered`
- if the caller is the account itself there is no cut

### vaccinate(account), called by anyone with liquid tokens

Valid while the season is open, when `account` is `uninfected`.

- caller burns the current vaccination price from its own `liquid`: `vax_cost * (10000 + 10 * immune) / 10000`, where `immune` is the number of accounts vaccinated so far. The first vaccination costs `vax_cost` and each later one costs 0.1% of `vax_cost` more. `get_stats` returns the current price as `vax_cost`
- `account` becomes `immune`
- if `infected >= min_hosts` and `immune * 10000 >= herd_bps * infected`, the season ends now (herd immunity)

### claim(), called by an account with spread points

Valid only after the season is over. Pays `reserve_pool * points / total_points` as newly minted `liquid`, once per account, where `reserve_pool = infection_minted * reserve_bps / 10000` and `infection_minted` counts every token minted by catches (targets and carrier bonuses, not claims). The pool and the total points are frozen when the season ends because both only change on catches, so the reserve is always a share of what the outbreak actually produced.

### ft_transfer

Standard NEP-141, from `liquid` only.

### storage_deposit, storage_withdraw, storage_unregister

Standard NEP-145. Registering costs `STORAGE_MIN` (0.003 NEAR) and anything above it is refunded. No storage balance is ever withdrawable, so `storage_withdraw` only returns the balance, and a registered account cannot be unregistered because it is a node in the infection tree and the host index.

## Clocks and the season

- Incubation clock: a carrier's clock restarts at infection and at each catch. It expires `incubation_ns` after the last restart.
- Season ends at the first of: herd immunity (see `vaccinate`), or `season_max_ns` after `new()`.
- After the season ends, catching and vaccinating stop and every `contagious` account counts as expired. Both `recover()` and `cull()` then burn `burn_expired_bps`. The cheaper `burn_voluntary_bps` only applies to a `recover()` made before the season ends, so carriers have to settle ahead of a time end, and a herd end can catch anyone still holding `viral`.
- After settlement a `recovered` token is a plain NEP-141 balance.

## Hard constraints

- No owner, no admin, no upgrade key, no withdraw method. Parameters are set once in `new()` and readable via `get_config()`. Two rules are constants in `core/src/lib.rs`, not parameters: the spread points (4, 2, 1) and the vaccination price step (10 bps of `vax_cost` per vaccination).
- No newcomer money flows to earlier holders. The catch fee stays in the contract and there is no method that moves it out. Rewards are only the token itself.
- Every supply change is a NEP-141 event, so total supply is always `sum(mint events) - sum(burn events)`.

## Parameters

Chosen from simulation runs, not guessed. See `sim/` and the README.

| Name | Unit | Meaning |
|---|---|---|
| `base` | tokens | mint at generation 0 |
| `decay_bps` | bps | mint multiplier per generation (below 10000) |
| `bonus_bps` | bps | carrier bonus as a share of the target mint |
| `fanout` | count | max children per carrier |
| `incubation_ns` | ns | time a carrier has to spread |
| `burn_expired_bps` | bps | burn on cull, late recover or season end |
| `burn_voluntary_bps` | bps | burn on timely recover |
| `cull_cut_bps` | bps | plague doctor share of a cull burn |
| `vax_cost` | tokens | cost of the first vaccination; the price rises with each one (see `vaccinate`) |
| `herd_bps` | bps | immune per infected needed to end the season |
| `min_hosts` | count | minimum infected before herd immunity counts |
| `season_max_ns` | ns | hard season length |
| `min_fee` | yoctoNEAR | floor on the catch fee, sized to cover the storage a new account costs |
| `fee_per_unit` | yoctoNEAR per raw token unit | catch fee is at least `target_mint * fee_per_unit`, so every token is priced the same wherever it sits in the tree |
| `reserve_bps` | bps | super-spreader reserve paid by `claim`, as a share of `infection_minted` |

## One round, as text

```
alice (contagious, gen 2)         bob (uninfected)              contract
  |                                  |                             |
  | shares /?from=alice.testnet      |                             |
  |--------------------------------->|                             |
  |                                  | catch(alice) + fee         |
  |                                  |---------------------------->|
  |                                  |       checks: season open, bob uninfected + named,
  |                                  |       alice contagious + not expired + fanout left
  |                                  |       bob: contagious, gen 3, viral += mint
  |                                  |       alice: viral += bonus, clock restarts
  |                                  |       points: alice +4, parent +2, grandparent +1
  |                                  |<--- events: ft_mint x2, contagion_catch
  |                                  |
  | ... incubation_ns passes with no new catch by bob ...
  |                                  |
carol (anyone)                                                     |
  | cull(bob)                                                      |
  |--------------------------------------------------------------->|
  |        bob expired: burn burn_expired_bps of viral
  |        carol gets cull_cut_bps of that burn as liquid
  |        rest of viral becomes bob's liquid, bob is recovered
  |<-------------------------------- events: ft_transfer, ft_burn, contagion_cull
```

## Known limits

- A funded attacker who catches from a patient zero and then builds a wide tree of alt accounts at generations 2 to 4 takes a large share of infection mint (about a quarter in the `sybil` scenario with 500 alts). Position in the tree is scarce and that cannot be priced away, but the price per token can: with `fee_per_unit` the attacker pays the same per token as anyone, not a flat fee for the most valuable slots. Raising `fee_per_unit` raises the cost of that share linearly. Faster decay and a smaller `fanout` make the share larger, not smaller. The `sybil` scenario in `sim/` shows the numbers.
- Time is block timestamp. Culls depend on someone choosing to call them; the cut is the incentive.
- Vaccination needs no consent from the target: any holder can make any uninfected account immune for `vax_cost`, permanently.
- Herd immunity at the threshold is still cheap next to the outbreak it ends: `herd_bps * min_hosts / 10000` vaccinations (300 with the mainnet preset) cost about 6,900 FLU with the rising price, against 6,000 at a flat price. The step bites later: the 5,100 vaccinations needed at 17,000 infected cost about 362,000 FLU.
- `new()` rejects a zero `vax_cost` and a `burn_voluntary_bps` above `burn_expired_bps`.
