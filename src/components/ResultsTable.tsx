import { useMemo, useState } from 'react'
import type { DocumentRow, StatusGroup } from '../types/processing'
import { StatusBadge } from './StatusBadge'
import { groupOf } from '../lib/status'

type Filter = 'all' | StatusGroup
const FILTERS: [Filter, string][] = [['all', 'Todos'], ['ok', 'Correcto'], ['review', 'Revisar'], ['error', 'Error']]

export function filterRows(rows: DocumentRow[], filter: Filter, query: string): DocumentRow[] {
  const q = query.trim().toLowerCase()
  return rows.filter(r =>
    (filter === 'all' || groupOf(r.status) === filter) &&
    (!q || [r.studentName, r.effectiveCode, r.extractedCode, r.originalName, String(r.sourcePageNumber ?? '')]
      .some(v => v?.toLowerCase().includes(q))))
}

interface Props {
  rows: DocumentRow[]
  onPreview: (row: DocumentRow) => void
  onEdit: (row: DocumentRow) => void
}

export function ResultsTable({ rows, onPreview, onEdit }: Props) {
  const [filter, setFilter] = useState<Filter>('all')
  const [query, setQuery] = useState('')
  const shown = useMemo(() => filterRows(rows, filter, query), [rows, filter, query])

  return (
    <section className="rounded-card border border-gray/60 bg-white shadow-sm">
      <div className="flex flex-wrap items-center gap-3 border-b border-gray/60 p-3">
        <div role="group" aria-label="Filtrar por estado" className="flex gap-1">
          {FILTERS.map(([f, label]) => (
            <button key={f} onClick={() => setFilter(f)} aria-pressed={filter === f}
              className={`rounded-lg px-3 py-1 text-sm font-medium ${filter === f ? 'bg-navy text-white' : 'text-slate-600 hover:bg-offwhite'}`}>
              {label}
            </button>
          ))}
        </div>
        <input type="search" value={query} onChange={e => setQuery(e.target.value)} placeholder="Buscar alumno, código o página…"
          aria-label="Buscar" className="ml-auto w-72 rounded-lg border border-gray px-3 py-1.5 text-sm focus:outline-2 focus:outline-blue" />
        <span className="text-xs text-slate-500">{shown.length} de {rows.length}</span>
      </div>
      <div className="max-h-[48vh] overflow-auto">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-offwhite text-left text-xs uppercase text-slate-500">
            <tr>
              <th className="px-3 py-2">Pág.</th><th className="px-3 py-2">Alumno</th><th className="px-3 py-2">Código</th>
              <th className="px-3 py-2">Nombre nuevo</th><th className="px-3 py-2">Estado</th>
              <th className="px-3 py-2">Observación</th><th className="px-3 py-2 text-right">Acciones</th>
            </tr>
          </thead>
          <tbody>
            {shown.map(r => (
              <tr key={r.id} className="border-t border-gray/40 hover:bg-offwhite/70">
                <td className="px-3 py-2 tabular-nums">{r.sourcePageNumber}</td>
                <td className="px-3 py-2">{r.studentName ?? <span className="text-slate-400">—</span>}</td>
                <td className="px-3 py-2 font-mono">
                  {r.effectiveCode ?? <span className="text-slate-400">—</span>}
                  {r.correctedManually && <span title="Corregido manualmente" className="ml-1 text-blue">✎</span>}
                </td>
                <td className="px-3 py-2 font-mono text-slate-600">{r.proposedName ?? '—'}</td>
                <td className="px-3 py-2"><StatusBadge status={r.status} /></td>
                <td className="px-3 py-2 text-slate-600">{r.observation}</td>
                <td className="whitespace-nowrap px-3 py-2 text-right">
                  <button onClick={() => onPreview(r)} className="rounded px-2 py-1 text-blue hover:bg-blue/10">Ver esta página</button>
                  <button onClick={() => onEdit(r)} className="rounded px-2 py-1 text-blue hover:bg-blue/10">Editar código</button>
                </td>
              </tr>
            ))}
            {shown.length === 0 && (
              <tr><td colSpan={7} className="px-3 py-8 text-center text-slate-500">Sin resultados</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  )
}
