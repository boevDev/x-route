import type { DnsStatus, DnsTarget } from './dns.types';

function isSameAddressSet(current: string[], target: string[]): boolean {
  if (target.length === 0) return true; // для этой семьи адресов ничего не требуем
  if (current.length !== target.length) return false;
  const currentSet = new Set(current);
  return target.every((ip) => currentSet.has(ip));
}

export function isDnsTargetActive(status: DnsStatus | undefined, target: DnsTarget): boolean {
  if (!status) return false;
  if (target.ipv4.length === 0 && target.ipv6.length === 0) return false;

  return isSameAddressSet(status.ipv4, target.ipv4) && isSameAddressSet(status.ipv6, target.ipv6);
}
