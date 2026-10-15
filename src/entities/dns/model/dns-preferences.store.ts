import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { DnsTarget } from '../../../shared/api/dns/dns.types';

export const DEFAULT_DNS: DnsTarget = {
  ipv4: ['159.194.200.33', '193.233.112.67', '193.233.112.88', '37.230.192.51', '193.233.112.68'],
  ipv6: ['2a00:ab00:1233:26::50', '2a00:ab00:1233:26::51'],
};

interface DnsPreferencesState {
  target: DnsTarget;
  setTarget: (target: DnsTarget) => void;
  resetTarget: () => void;
}

export const useDnsPreferences = create<DnsPreferencesState>()(
  persist(
    (set) => ({
      target: DEFAULT_DNS,
      setTarget: (target) => set({ target }),
      resetTarget: () => set({ target: DEFAULT_DNS }),
    }),
    { name: 'x-route-preferences' },
  ),
);
