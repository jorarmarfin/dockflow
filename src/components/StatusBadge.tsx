import type { DocumentStatus } from '../types/processing'
import { GROUP_STYLE, STATUS_LABEL, groupOf } from '../lib/status'

export function StatusBadge({ status }: { status: DocumentStatus }) {
  const g = GROUP_STYLE[groupOf(status)]
  return (
    <span className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-semibold ${g.cls}`}>
      <span aria-hidden>{g.icon}</span>
      {STATUS_LABEL[status]}
    </span>
  )
}
