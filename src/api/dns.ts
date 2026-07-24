import { invoke } from '@tauri-apps/api/core';
import type { DnsStatus, DnsTarget } from '../types/dns';

export const dnsApi = {
  getStatus: () => invoke<DnsStatus>('get_dns_status'),

  setDns: (enable: boolean, target: DnsTarget) =>
    invoke<DnsStatus>('set_dns', {
      enable,
      ipv4: target.ipv4,
      ipv6: target.ipv6,
    }),
};
