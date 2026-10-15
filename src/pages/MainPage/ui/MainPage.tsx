import { useState } from 'react';
import { DnsStatusCard } from '../../../widgets/dns-status-card/ui/DnsStatusCard';
import { DnsSettingsPanel } from '../../../widgets/dns-settings/ui/DnsSettingsPanel';

export function MainPage() {
  const [settingsOpen, setSettingsOpen] = useState(false);

  return settingsOpen ? (
    <DnsSettingsPanel onClose={() => setSettingsOpen(false)} />
  ) : (
    <DnsStatusCard onOpenSettings={() => setSettingsOpen(true)} />
  );
}
