import { useState } from 'react';
import { DnsStatusCard } from '@/widgets/dns-status-card';
import { DnsSettingsPanel } from '@/widgets/dns-settings';

export function MainPage() {
  const [settingsOpen, setSettingsOpen] = useState(false);

  return settingsOpen ? (
    <DnsSettingsPanel onClose={() => setSettingsOpen(false)} />
  ) : (
    <DnsStatusCard onOpenSettings={() => setSettingsOpen(true)} />
  );
}
