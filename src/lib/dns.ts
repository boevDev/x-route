import type { DnsStatus, DnsTarget } from '../types/dns';

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

const IPV4_RE = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/;

export function isValidIPv4(value: string): boolean {
  const match = value.match(IPV4_RE);
  if (!match) return false;
  return match.slice(1).every((octet) => Number(octet) >= 0 && Number(octet) <= 255);
}

export function isValidIPv6(value: string): boolean {
  if (!value.includes(':')) return false;
  // Мягкая проверка формата — окончательную валидацию всё равно делает Rust (IpAddr::parse)
  return /^[0-9a-fA-F:]+$/.test(value) && value.length >= 2;
}
