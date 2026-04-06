interface ToggleSwitchProps {
  enabled: boolean;
  onChange: () => void;
  label: string;
  disabled?: boolean;
  variant?: 'primary' | 'success';
}

export function ToggleSwitch({ enabled, onChange, label, disabled, variant = 'primary' }: ToggleSwitchProps) {
  const enabledColor = variant === 'success' ? 'bg-success-500' : 'bg-primary-500';

  return (
    <button
      type="button"
      role="switch"
      aria-checked={enabled}
      aria-label={label}
      onClick={onChange}
      disabled={disabled}
      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
        enabled ? enabledColor : 'bg-surface-100/50'
      }`}
    >
      <span
        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
          enabled ? 'translate-x-6' : 'translate-x-1'
        }`}
      />
    </button>
  );
}
