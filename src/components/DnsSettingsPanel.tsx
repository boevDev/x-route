import { useState } from 'react';
import { ArrowLeft, RotateCcw } from 'lucide-react';
import { useDnsPreferences } from '../store';
import { isValidIPv4, isValidIPv6 } from '../lib/dns';

interface FormState {
  ipv4Primary: string;
  ipv4Secondary: string;
  ipv6Primary: string;
  ipv6Secondary: string;
}

interface FormErrors {
  ipv4Primary?: string;
  ipv4Secondary?: string;
  ipv6Primary?: string;
  ipv6Secondary?: string;
}

function buildFormState(target: { ipv4: string[]; ipv6: string[] }): FormState {
  return {
    ipv4Primary: target.ipv4[0] ?? '',
    ipv4Secondary: target.ipv4[1] ?? '',
    ipv6Primary: target.ipv6[0] ?? '',
    ipv6Secondary: target.ipv6[1] ?? '',
  };
}

export function DnsSettingsPanel({ onClose }: { onClose: () => void }) {
  const target = useDnsPreferences((state) => state.target);
  const setTarget = useDnsPreferences((state) => state.setTarget);

  const [form, setForm] = useState<FormState>(() => buildFormState(target));
  const [errors, setErrors] = useState<FormErrors>({});

  const updateField = (field: keyof FormState, value: string) => {
    setForm((prev) => ({ ...prev, [field]: value }));
    setErrors((prev) => ({ ...prev, [field]: undefined }));
  };

  const handleResetToDefaults = () => {
    setForm(
      buildFormState({
        ipv4: ['111.88.96.50', '111.88.96.51'],
        ipv6: ['2a00:ab00:1233:26::50', '2a00:ab00:1233:26::51'],
      }),
    );
    setErrors({});
  };

  const handleSave = () => {
    const nextErrors: FormErrors = {};

    if (form.ipv4Primary && !isValidIPv4(form.ipv4Primary)) {
      nextErrors.ipv4Primary = 'Некорректный IPv4-адрес';
    }
    if (form.ipv4Secondary && !isValidIPv4(form.ipv4Secondary)) {
      nextErrors.ipv4Secondary = 'Некорректный IPv4-адрес';
    }
    if (form.ipv6Primary && !isValidIPv6(form.ipv6Primary)) {
      nextErrors.ipv6Primary = 'Некорректный IPv6-адрес';
    }
    if (form.ipv6Secondary && !isValidIPv6(form.ipv6Secondary)) {
      nextErrors.ipv6Secondary = 'Некорректный IPv6-адрес';
    }

    if (Object.keys(nextErrors).length > 0) {
      setErrors(nextErrors);
      return;
    }

    const ipv4 = [form.ipv4Primary, form.ipv4Secondary].filter(Boolean);
    const ipv6 = [form.ipv6Primary, form.ipv6Secondary].filter(Boolean);

    if (ipv4.length === 0 && ipv6.length === 0) {
      setErrors({ ipv4Primary: 'Укажите хотя бы один адрес' });
      return;
    }

    setTarget({ ipv4, ipv6 });
    onClose();
  };

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

          <label className="settings-field">
            <span>Основной</span>
            <input
              value={form.ipv4Primary}
              onChange={(e) => updateField('ipv4Primary', e.target.value.trim())}
              placeholder="111.88.96.50"
              spellCheck={false}
            />
            {errors.ipv4Primary && <span className="field-error">{errors.ipv4Primary}</span>}
          </label>

          <label className="settings-field">
            <span>Дополнительный</span>
            <input
              value={form.ipv4Secondary}
              onChange={(e) => updateField('ipv4Secondary', e.target.value.trim())}
              placeholder="111.88.96.51"
              spellCheck={false}
            />
            {errors.ipv4Secondary && <span className="field-error">{errors.ipv4Secondary}</span>}
          </label>
        </fieldset>

        <fieldset className="settings-group">
          <legend>IPv6 (необязательно)</legend>

          <label className="settings-field">
            <span>Основной</span>
            <input
              value={form.ipv6Primary}
              onChange={(e) => updateField('ipv6Primary', e.target.value.trim())}
              placeholder="2a00:ab00:1233:26::50"
              spellCheck={false}
            />
            {errors.ipv6Primary && <span className="field-error">{errors.ipv6Primary}</span>}
          </label>

          <label className="settings-field">
            <span>Дополнительный</span>
            <input
              value={form.ipv6Secondary}
              onChange={(e) => updateField('ipv6Secondary', e.target.value.trim())}
              placeholder="2a00:ab00:1233:26::51"
              spellCheck={false}
            />
            {errors.ipv6Secondary && <span className="field-error">{errors.ipv6Secondary}</span>}
          </label>
        </fieldset>
      </div>

      <div className="settings-footer">
        <button type="button" className="ghost-btn" onClick={handleResetToDefaults}>
          <RotateCcw size={13} />
          <span>Сбросить</span>
        </button>
        <button type="button" className="primary-btn" onClick={handleSave}>
          Сохранить
        </button>
      </div>
    </div>
  );
}
