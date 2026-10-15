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

export type DnsToggleOperation = 'enable' | 'disable';

export type DnsProgressStage =
  'resolving' | 'applying' | 'resetting' | 'clearing' | 'verifying' | 'done' | 'error';

export interface DnsProgressEvent {
  stage: DnsProgressStage;
  operation: DnsToggleOperation;
  detail?: string | null;
}
