import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { DEFAULT_DNS } from '../config/default-dns';
import type { DnsTarget } from './dns.types';

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
