import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { DnsTarget } from './types/dns';

export const DEFAULT_XBOX_DNS: DnsTarget = {
  ipv4: ['111.88.96.50', '111.88.96.51'],
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
      target: DEFAULT_XBOX_DNS,
      setTarget: (target) => set({ target }),
      resetTarget: () => set({ target: DEFAULT_XBOX_DNS }),
    }),
    { name: 'x-route-preferences' },
  ),
);
