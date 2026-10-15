interface DnsAddressFieldProps {
  label: string;
  value: string;
  placeholder: string;
  error?: string;
  onChange: (value: string) => void;
}

export function DnsAddressField({
  label,
  value,
  placeholder,
  error,
  onChange,
}: DnsAddressFieldProps) {
  return (
    <label className="settings-field">
      <span>{label}</span>
      <input
        value={value}
        onChange={(e) => onChange(e.target.value.trim())}
        placeholder={placeholder}
        spellCheck={false}
      />
      {error && <span className="field-error">{error}</span>}
    </label>
  );
}
