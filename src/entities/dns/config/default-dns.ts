import type { DnsTarget } from '../model/dns.types';

export const DEFAULT_DNS: DnsTarget = {
  ipv4: ['159.194.200.33', '193.233.112.67', '193.233.112.88', '37.230.192.51', '193.233.112.68'],
  ipv6: ['2a00:ab00:1233:26::50', '2a00:ab00:1233:26::51'],
};
