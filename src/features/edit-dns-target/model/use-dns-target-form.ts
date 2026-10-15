import { useState } from 'react';
import {
  DEFAULT_DNS,
  isValidIPv4,
  isValidIPv6,
  useDnsPreferences,
  type DnsTarget,
} from '@/entities/dns';

export interface DnsTargetForm {
  ipv4Primary: string;
  ipv4Secondary: string;
  ipv6Primary: string;
  ipv6Secondary: string;
}

export type DnsTargetFormErrors = Partial<Record<keyof DnsTargetForm, string>>;

const INVALID_IPV4 = 'Некорректный IPv4-адрес';
const INVALID_IPV6 = 'Некорректный IPv6-адрес';
const EMPTY_TARGET = 'Укажите хотя бы один адрес';

function toForm(target: DnsTarget): DnsTargetForm {
  return {
    ipv4Primary: target.ipv4[0] ?? '',
    ipv4Secondary: target.ipv4[1] ?? '',
    ipv6Primary: target.ipv6[0] ?? '',
    ipv6Secondary: target.ipv6[1] ?? '',
  };
}

function validate(form: DnsTargetForm): DnsTargetFormErrors {
  const errors: DnsTargetFormErrors = {};

  if (form.ipv4Primary && !isValidIPv4(form.ipv4Primary)) errors.ipv4Primary = INVALID_IPV4;
  if (form.ipv4Secondary && !isValidIPv4(form.ipv4Secondary)) errors.ipv4Secondary = INVALID_IPV4;
  if (form.ipv6Primary && !isValidIPv6(form.ipv6Primary)) errors.ipv6Primary = INVALID_IPV6;
  if (form.ipv6Secondary && !isValidIPv6(form.ipv6Secondary)) errors.ipv6Secondary = INVALID_IPV6;

  return errors;
}

function toTarget(form: DnsTargetForm): DnsTarget {
  return {
    ipv4: [form.ipv4Primary, form.ipv4Secondary].filter(Boolean),
    ipv6: [form.ipv6Primary, form.ipv6Secondary].filter(Boolean),
  };
}

export function useDnsTargetForm(onSaved: () => void) {
  const target = useDnsPreferences((state) => state.target);
  const setTarget = useDnsPreferences((state) => state.setTarget);

  const [form, setForm] = useState<DnsTargetForm>(() => toForm(target));
  const [errors, setErrors] = useState<DnsTargetFormErrors>({});

  const updateField = (field: keyof DnsTargetForm, value: string) => {
    setForm((prev) => ({ ...prev, [field]: value }));
    setErrors((prev) => ({ ...prev, [field]: undefined }));
  };

  const resetToDefaults = () => {
    setForm(toForm(DEFAULT_DNS));
    setErrors({});
  };

  const submit = () => {
    const nextErrors = validate(form);
    if (Object.keys(nextErrors).length > 0) {
      setErrors(nextErrors);
      return;
    }

    const nextTarget = toTarget(form);
    if (nextTarget.ipv4.length === 0 && nextTarget.ipv6.length === 0) {
      setErrors({ ipv4Primary: EMPTY_TARGET });
      return;
    }

    setTarget(nextTarget);
    onSaved();
  };

  return { form, errors, updateField, resetToDefaults, submit };
}
