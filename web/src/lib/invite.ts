export const readParams = (): URLSearchParams => {
  const merged = new URLSearchParams(window.location.search);
  const hash = window.location.hash;
  const query = hash.includes('?') ? hash.slice(hash.indexOf('?') + 1) : '';
  new URLSearchParams(query).forEach((value, key) => merged.set(key, value));
  return merged;
};

export const inviteLink = (contractId: string, accountId: string): string => {
  const query = new URLSearchParams({ c: contractId, from: accountId });
  return `${window.location.origin}${window.location.pathname}#/?${query}`;
};

export const shareText = (link: string): string =>
  `I caught $FLU. You can't buy it, you have to catch it. Catch it from me: ${link} @NEARProtocol #NEARHackathon`;

export const shareUrl = (link: string): string =>
  `https://x.com/intent/post?text=${encodeURIComponent(shareText(link))}`;
