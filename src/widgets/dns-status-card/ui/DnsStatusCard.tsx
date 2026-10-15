import { AlertTriangle, Globe, Monitor, ShieldCheck, Router, Settings } from 'lucide-react';
import { useToggleDns } from '../../../features/toggle-dns/model/use-toggle-dns';
import { workingStageLabel, successLabel } from '../../../features/toggle-dns/model/dns-progress';

export function DnsStatusCard({ onOpenSettings }: { onOpenSettings: () => void }) {
  const {
    isActive,
    isBusy,
    toggle,
    status,
    target,
    handleToggle,
    errorMessage,
  } = useToggleDns();

  const rowTitle =
    toggle.kind === 'working'
      ? workingStageLabel(toggle.operation, toggle.stage)
      : toggle.kind === 'success'
        ? successLabel(toggle.operation)
        : isBusy
          ? 'Применение…'
          : isActive
            ? 'DNS активирован'
            : 'Автоматический DNS';

  const rowSub =
    toggle.kind === 'working'
      ? 'Обычно это занимает несколько секунд'
      : isActive
        ? 'Трафик идёт через кастомный DNS'
        : 'Нажмите, чтобы включить';

  const targetAddresses = [...target.ipv4, ...target.ipv6];
  const currentAddresses = status ? [...status.ipv4, ...status.ipv6] : [];

  return (
    <div className="panel">
      <header className="titlebar">
        <span className="app-title">X-Route</span>
        <span className={`live-dot ${isBusy ? 'live-dot--busy' : ''}`} />
        <button type="button" className="icon-btn" onClick={onOpenSettings} aria-label="Настройки">
          <Settings size={15} />
        </button>
      </header>

      <div className="diagram">
        <div className="diagram-node">
          <Monitor size={20} />
        </div>

        <div className="diagram-lanes">
          <div className={`lane ${isActive ? 'lane--active' : ''}`}>
            <span className="lane-wire" />
            <div className="lane-label">
              <ShieldCheck size={14} />
              <span>Кастомный DNS</span>
            </div>
            <span className="lane-wire" />
          </div>

          <div className={`lane ${!isActive ? 'lane--active' : ''}`}>
            <span className="lane-wire" />
            <div className="lane-label">
              <Router size={14} />
              <span>Провайдер (DHCP)</span>
            </div>
            <span className="lane-wire" />
          </div>
        </div>

        <div className="diagram-node">
          <Globe size={20} />
        </div>
      </div>

      <button
        type="button"
        className={`switch-row ${isActive ? 'switch-row--active' : ''}`}
        onClick={handleToggle}
        disabled={isBusy}
      >
        <span className="switch-row-text">
          <span className="switch-row-title">{rowTitle}</span>
          <span className="switch-row-sub">{rowSub}</span>
        </span>
        <span className="switch" aria-hidden="true">
          <span className="switch-thumb" />
        </span>
      </button>

      {errorMessage && (
        <div className="error-banner">
          <AlertTriangle size={14} />
          <span>{errorMessage}</span>
        </div>
      )}

      <div className="readout">
        <div className="readout-row">
          <span className="readout-label">Цель</span>
          <span className="readout-value">{targetAddresses.join(', ') || '—'}</span>
        </div>
        <div className="readout-row">
          <span className="readout-label">Система{status ? ` · ${status.interfaceName}` : ''}</span>
          <span className="readout-value">
            {currentAddresses.length > 0 ? currentAddresses.join(', ') : 'Авто (DHCP)'}
          </span>
        </div>
      </div>

      <p className="tray-hint">Закрытие окна сворачивает в трей — приложение продолжит работать</p>
    </div>
  );
}
