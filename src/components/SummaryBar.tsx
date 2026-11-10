import type { BatchSummary } from '../types/processing'

export function SummaryBar({ s }: { s: BatchSummary }) {
  const items: [string, number, string][] = [
    ['Total', s.total, 'text-navy'], ['Listos / pendientes', s.pending, 'text-teal'],
    ['Procesados', s.processed, 'text-blue'], ['Duplicados', s.duplicates, 'text-amber-700'],
    ['Errores', s.errors, 'text-danger'],
  ]
  return (
    <dl className="grid grid-cols-2 gap-3 sm:grid-cols-5" aria-label="Resumen del lote">
      {items.map(([label, n, cls]) => (
        <div key={label} className="rounded-card border border-gray/60 bg-white px-4 py-3 shadow-sm">
          <dt className="text-xs text-slate-500">{label}</dt>
          <dd className={`text-2xl font-bold ${cls}`}>{n}</dd>
        </div>
      ))}
    </dl>
  )
}
