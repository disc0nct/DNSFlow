export function SkeletonBlock({ className = '' }: { className?: string }) {
  return <div className={`animate-shimmer rounded ${className}`} />;
}
