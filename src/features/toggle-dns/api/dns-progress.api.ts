import { listen } from '@tauri-apps/api/event';
import type { DnsProgressEvent } from '@/entities/dns';

const DNS_PROGRESS_EVENT = 'dns-progress';

export function onDnsProgress(handler: (event: DnsProgressEvent) => void): Promise<() => void> {
  return listen<DnsProgressEvent>(DNS_PROGRESS_EVENT, (event) => handler(event.payload));
}
