import type { Config } from '../types';
import { bps, longDuration, near, tokens } from './format';
import { RANKS } from './game';

export type FaqItem = { id: string; q: string; a: string };
export type FaqGroup = { id: string; title: string; items: FaqItem[] };

export const buildFaq = (config: Config | null): FaqGroup[] => {
  const c = config;
  const fanout = c ? String(c.fanout) : 'a fixed number of';
  const clock = c ? longDuration(BigInt(c.incubation_ns)) : 'a fixed time';
  const season = c ? longDuration(BigInt(c.season_max_ns)) : 'a fixed time';
  const burnLate = c ? bps(c.burn_expired_bps) : 'a large share';
  const burnTimely = c ? bps(c.burn_voluntary_bps) : 'a small share';
  const cut = c ? bps(c.cull_cut_bps) : 'a cut';
  const bonus = c ? bps(c.bonus_bps) : 'a bonus';
  const decay = c ? bps(c.decay_bps) : 'a fixed share';
  const vax = c ? `${tokens(c.vax_cost)} FLU` : 'a fixed amount of FLU';
  const herd = c ? bps(c.herd_bps) : 'a fixed share';
  const minHosts = c ? String(c.min_hosts) : 'a minimum number of';
  const reserve = c ? bps(c.reserve_bps) : 'a share';
  const feeRate = c ? `${near(BigInt(c.fee_per_unit) * 10n ** 18n, 8)} NEAR per FLU you mint` : 'a rate per FLU you mint';
  const minFee = c ? `${near(c.min_fee)} NEAR` : 'a small minimum';
  const ranks = RANKS.map((rank) => (rank.min === 0 ? rank.name : `${rank.name} (${rank.min} points)`)).join(', ');

  return [
    {
      id: 'basics',
      title: 'The basics',
      items: [
        {
          id: 'what-is',
          q: 'What is Contagion?',
          a: 'Contagion is a token game on NEAR. The token, $FLU, spreads from person to person: you can only get it by catching it from someone who already has it.',
        },
        {
          id: 'why-cant-buy',
          q: "Why can't I buy $FLU?",
          a: 'There is no sale, no airdrop and no pool. The contract only mints FLU when someone catches it, so every holder got it from another holder.',
        },
        {
          id: 'value',
          q: 'Is $FLU worth anything?',
          a: 'No. This is a testnet deployment, so the NEAR it uses and the FLU it mints have no real value.',
        },
        {
          id: 'mainnet',
          q: 'Is this on mainnet?',
          a: 'No. It runs on the NEAR testnet only. A mainnet launch would be a separate deployment.',
        },
        {
          id: 'who-controls',
          q: 'Who controls the contract?',
          a: 'Nobody. The contract has no owner, admin, upgrade or withdraw function. Whatever it does, it does for every player the same way.',
        },
        {
          id: 'rules-change',
          q: 'Can the rules change?',
          a: 'No. Every setting, such as the clock length, the burn shares, the fees and the season length, was set once when the contract started. You can read them under The rules on this page.',
        },
        {
          id: 'data-source',
          q: 'Where does this page get its data?',
          a: 'From the NEAR blockchain, through public RPC servers. There is no separate backend or database.',
        },
      ],
    },
    {
      id: 'joining',
      title: 'Joining',
      items: [
        {
          id: 'how-catch',
          q: 'How do I catch it?',
          a: 'Open an invite link from a carrier, connect a testnet wallet and press Catch it. You pay a small fee in NEAR and the contract mints your first FLU.',
        },
        {
          id: 'invite-source',
          q: 'Where do I get an invite link?',
          a: 'From anyone who has caught it: their Player card shows a link to share. If nobody sent you one, press Find a live carrier to pick one at random.',
        },
        {
          id: 'cost',
          q: 'What does catching cost?',
          a: `The fee is ${feeRate}, and never less than ${minFee}. It follows how much you mint, so earlier generations pay more and mint more.`,
        },
        {
          id: 'fee-differs',
          q: 'Why do people pay different fees?',
          a: `Each generation mints ${decay} of the one before it, and the fee follows the mint. The minimum covers the storage your account needs on chain.`,
        },
        {
          id: 'fee-back',
          q: 'Do I get the fee back?',
          a: 'No. Fees stay in the contract and no function can take them out. Anything you attach above the fee is refunded.',
        },
        {
          id: 'twice',
          q: 'Can I catch it more than once?',
          a: 'No. An account catches it once. After that it is a carrier, then recovered. An account can also be vaccinated before it ever catches it.',
        },
        {
          id: 'cannot-catch',
          q: "Why can't I catch it from a carrier?",
          a: `A carrier cannot be caught from when their clock ran out, when they already passed it on to ${fanout} people, when their mint decayed to nothing, or when the season is over. The invite preview tells you which one applies.`,
        },
        {
          id: 'live-carrier',
          q: 'What does Find a live carrier do?',
          a: 'It picks a random carrier whose clock is still running and who has room for another catch, and sets up the invite for you.',
        },
      ],
    },
    {
      id: 'spreading',
      title: 'Spreading',
      items: [
        {
          id: 'clock',
          q: 'What is my clock?',
          a: `After you catch it you have ${clock} to pass it on. Every catch you make restarts the clock.`,
        },
        {
          id: 'bonus',
          q: 'What do I earn for passing it on?',
          a: `A bonus of ${bonus} of whatever each person you catch mints, plus spread points for you, your parent and your grandparent.`,
        },
        {
          id: 'how-many',
          q: 'How many people can I pass it on to?',
          a: `${fanout} people. After that your slots are full.`,
        },
        {
          id: 'points',
          q: 'What are spread points?',
          a: 'Points you earn when you catch someone, and smaller ones when people below you catch others. They set your place on the leaderboard and your share of the reserve when the season ends.',
        },
        {
          id: 'ranks',
          q: 'What are ranks like Super-spreader?',
          a: `Ranks are labels for your spread points: ${ranks}. They are for fun and do not change what you earn.`,
        },
        {
          id: 'patient-zero',
          q: 'What is a patient zero?',
          a: 'An account the contract started with. Patient zeros did not catch it from anyone, so they sit at the root of every tree.',
        },
      ],
    },
    {
      id: 'settling',
      title: 'Settling up',
      items: [
        {
          id: 'recover',
          q: 'What does Recover do?',
          a: `It ends your time as a carrier. ${burnTimely} of your locked FLU burns and the rest becomes spendable. Do it before your clock runs out.`,
        },
        {
          id: 'miss-clock',
          q: 'What happens if I miss my clock?',
          a: `Anyone can cull you. ${burnLate} of your locked FLU burns and the rest becomes spendable, so waiting costs more than recovering in time.`,
        },
        {
          id: 'plague-doctor',
          q: 'What is a plague doctor?',
          a: `Anyone who culls an expired carrier. They keep ${cut} of the burn as their reward and the rest is destroyed.`,
        },
        {
          id: 'who-culls',
          q: 'Why would someone cull me?',
          a: 'Because they earn a share of what burns. Culling is how the game cleans up carriers who stopped playing.',
        },
        {
          id: 'nobody-culls',
          q: 'What if nobody culls an expired carrier?',
          a: 'Nothing breaks. Their FLU stays locked until someone culls them or they recover themselves.',
        },
        {
          id: 'burn-goes',
          q: 'Where does burned FLU go?',
          a: 'It is destroyed, so the total supply shrinks. It is not sent to anyone.',
        },
      ],
    },
    {
      id: 'tokens',
      title: 'Your FLU',
      items: [
        {
          id: 'locked',
          q: 'Why is part of my balance locked?',
          a: 'FLU you get from catching starts locked, so nobody can sell a free position straight away. It becomes spendable when you recover or are culled.',
        },
        {
          id: 'spend',
          q: 'When can I spend my FLU?',
          a: 'After you recover or are culled. Spendable FLU can be sent to a friend or used to vaccinate someone.',
        },
        {
          id: 'send',
          q: 'Can I send FLU to a friend?',
          a: "Yes, spendable FLU. Your friend's account has to be registered first, which takes a one-time storage deposit of about 0.003 NEAR.",
        },
        {
          id: 'wallet-show',
          q: 'Will FLU show up in my wallet?',
          a: 'Wallets that read token details show the coin and your balance once your account holds FLU. The total includes locked FLU, but only the spendable part can be sent.',
        },
        {
          id: 'register',
          q: 'What does registering mean?',
          a: 'An account needs a small storage deposit before it can hold a token. Catching it registers you automatically. Other accounts, such as a plague doctor collecting a cut, register once.',
        },
        {
          id: 'trade',
          q: 'Can I trade FLU?',
          a: 'Spendable FLU is a standard NEAR token, so it can be listed anywhere that supports NEAR tokens. This testnet version has no market.',
        },
      ],
    },
    {
      id: 'season',
      title: 'Vaccines and the season',
      items: [
        {
          id: 'vaccinate',
          q: 'What does vaccinating do?',
          a: `You burn FLU to make one account immune. The first vaccination costs ${vax} and each one after it costs a little more. An immune account can never catch it, which slows the outbreak down.`,
        },
        {
          id: 'who-vaccinate',
          q: 'Who can I vaccinate?',
          a: 'Any account that has not caught it yet, including a friend or your own second account. You need enough spendable FLU.',
        },
        {
          id: 'herd',
          q: 'What is herd immunity?',
          a: `When ${herd} of all infected accounts are immune, and at least ${minHosts} accounts have been infected, the season ends.`,
        },
        {
          id: 'season-ends',
          q: 'How does a season end?',
          a: `Two ways: herd immunity, or the clock, which runs ${season} from the start. Whichever comes first.`,
        },
        {
          id: 'after-end',
          q: 'What happens when the season ends?',
          a: `Nobody can catch it or be vaccinated any more, and every carrier counts as expired. Recovering or being culled then burns ${burnLate}. Spread points pay out the reserve.`,
        },
        {
          id: 'reserve',
          q: 'What is the reserve?',
          a: `Extra FLU minted when the season ends, worth ${reserve} of everything the outbreak minted. It is shared out by spread points.`,
        },
        {
          id: 'claim',
          q: 'How do I claim my share?',
          a: "After the season ends, open your Player card and press Claim. You can claim once. Your share is your points divided by everyone's points.",
        },
      ],
    },
    {
      id: 'trust',
      title: 'Safety and trust',
      items: [
        {
          id: 'cheat',
          q: 'Can someone cheat with lots of accounts?',
          a: 'Partly. Early spots in the tree are scarce and one person can create many accounts. The fee is the defence: every account pays the same price per FLU it mints, so extra accounts are never cheaper. That is a limit, not a guarantee.',
        },
        {
          id: 'fees-taken',
          q: 'Can the team take the fees?',
          a: 'No. Fees stay in the contract and no function can move them out.',
        },
        {
          id: 'testnet-only',
          q: 'Why is it testnet only?',
          a: 'It is a demo that shows the mechanic, so real money should not touch it.',
        },
      ],
    },
    {
      id: 'wallets',
      title: 'Wallets and problems',
      items: [
        {
          id: 'wallets',
          q: 'Which wallets work?',
          a: 'The wallets the Connect button offers. On testnet it lists only wallets that declare testnet support, so some popular wallets show up only on mainnet.',
        },
        {
          id: 'gas',
          q: 'Do I need NEAR to play?',
          a: 'Yes, testnet NEAR for the catch fee and for network gas. Testnet NEAR is free from a testnet faucet.',
        },
        {
          id: 'failed',
          q: 'My transaction failed. Why?',
          a: 'The message names the reason, for example a carrier that expired or is full, a fee below the minimum, or not enough spendable FLU. Fix that and try again. If an invite went stale, press Find a live carrier.',
        },
        {
          id: 'stale',
          q: 'Why do the numbers look out of date?',
          a: 'The page checks the chain every few seconds. A new transaction can take a moment to show up, and the page refreshes by itself.',
        },
      ],
    },
  ];
};

export const filterFaq = (groups: FaqGroup[], query: string, groupId: string): FaqGroup[] => {
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
  return groups
    .filter((group) => groupId === 'all' || group.id === groupId)
    .map((group) => ({
      ...group,
      items: group.items.filter((item) => {
        const text = `${item.q} ${item.a}`.toLowerCase();
        return terms.every((term) => text.includes(term));
      }),
    }))
    .filter((group) => group.items.length > 0);
};

export const countFaq = (groups: FaqGroup[]): number => groups.reduce((sum, group) => sum + group.items.length, 0);
