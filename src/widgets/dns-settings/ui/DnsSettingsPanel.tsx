import { ArrowLeft, RotateCcw } from 'lucide-react';
import { useDnsTargetForm } from '@/features/edit-dns-target';
import { DnsAddressField } from './DnsAddressField';

export function DnsSettingsPanel({ onClose }: { onClose: () => void }) {
  const { form, errors, updateField, resetToDefaults, submit } = useDnsTargetForm(onClose);

  return (
    <div className="panel">
      <header className="titlebar">
        <button type="button" className="icon-btn" onClick={onClose} aria-label="Назад">
          <ArrowLeft size={16} />
        </button>
        <span className="app-title">Настройки DNS</span>
      </header>

      <div className="settings-body">
        <fieldset className="settings-group">
          <legend>IPv4</legend>

          <DnsAddressField
            label="Основной"
            value={form.ipv4Primary}
            placeholder="111.88.96.50"
            error={errors.ipv4Primary}
            onChange={(value) => updateField('ipv4Primary', value)}
          />
          <DnsAddressField
            label="Дополнительный"
            value={form.ipv4Secondary}
            placeholder="111.88.96.51"
            error={errors.ipv4Secondary}
            onChange={(value) => updateField('ipv4Secondary', value)}
          />
        </fieldset>

        <fieldset className="settings-group">
          <legend>IPv6 (необязательно)</legend>

          <DnsAddressField
            label="Основной"
            value={form.ipv6Primary}
            placeholder="2a00:ab00:1233:26::50"
            error={errors.ipv6Primary}
            onChange={(value) => updateField('ipv6Primary', value)}
          />
          <DnsAddressField
            label="Дополнительный"
            value={form.ipv6Secondary}
            placeholder="2a00:ab00:1233:26::51"
            error={errors.ipv6Secondary}
            onChange={(value) => updateField('ipv6Secondary', value)}
          />
        </fieldset>
      </div>

      <div className="settings-footer">
        <button type="button" className="ghost-btn" onClick={resetToDefaults}>
          <RotateCcw size={13} />
          <span>Сбросить</span>
        </button>
        <button type="button" className="primary-btn" onClick={submit}>
          Сохранить
        </button>
      </div>
    </div>
  );
}
