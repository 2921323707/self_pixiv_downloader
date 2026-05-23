const labels: Record<string, string> = {
  pending: "Pending",
  running: "Running",
  completed: "Completed",
  completed_with_errors: "Warnings",
  failed: "Failed",
  cancelled: "Cancelled"
};

export function StatusBadge({ status }: { status?: string | null }) {
  const normalized = status || "pending";

  return (
    <span className={`status-badge ${normalized}`}>
      {labels[normalized] || normalized}
    </span>
  );
}
