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
