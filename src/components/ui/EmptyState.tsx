interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description: string;
  action?: {
    label: string;
    onClick: () => void;
  };
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-12 text-center">
      {icon && (
        <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-surface-100/40">
          {icon}
        </div>
      )}
      <h3 className="mb-1 text-base font-semibold text-surface-50">{title}</h3>
      <p className="text-sm text-slate-500">{description}</p>
      {action && (
        <button
          type="button"
          onClick={action.onClick}
          className="mt-4 inline-flex items-center gap-1.5 rounded-lg bg-gradient-to-r from-primary-500 to-accent-500 px-4 py-2 text-sm font-medium text-white transition-all hover:from-primary-400 hover:to-accent-400"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}
