export interface DnsStatus {
  interfaceName: string;
  interfaceIndex: number;
  ipv4: string[];
  ipv6: string[];
}

export interface DnsTarget {
  ipv4: string[];
  ipv6: string[];
}
