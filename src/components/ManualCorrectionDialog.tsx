import { useState } from 'react'
import type { DocumentRow } from '../types/processing'

const FORMAT = /^[0-9]{8}[A-Za-z]$/

interface Props {
  row: DocumentRow
  onSave: (code: string) => Promise<void>
  onClose: () => void
}

export function ManualCorrectionDialog({ row, onSave, onClose }: Props) {
  const [code, setCode] = useState(row.effectiveCode ?? row.extractedCode ?? '')
  const [error, setError] = useState<string | null>(null)
  const valid = FORMAT.test(code.trim())

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!valid) return
    try { await onSave(code.trim()) } catch (err) { setError(String(err)) }
  }

  return (
    <div role="dialog" aria-modal="true" aria-label="Editar código" className="fixed inset-0 z-10 grid place-items-center bg-navy/50 p-4">
      <form onSubmit={submit} className="w-full max-w-md rounded-card bg-white p-6 shadow-xl">
        <h2 className="text-lg font-bold">Editar código — página {row.sourcePageNumber}</h2>
        <p className="mt-1 text-sm text-slate-600">
          Solo cambia el nombre del archivo de salida; el PDF no se modifica. Alumno: {row.studentName ?? 'sin nombre'}.
        </p>
        <label className="mt-4 block text-sm font-medium" htmlFor="code">Código (8 dígitos + letra)</label>
        <input id="code" autoFocus value={code} onChange={e => { setCode(e.target.value); setError(null) }}
          className="mt-1 w-full rounded-lg border border-gray px-3 py-2 font-mono uppercase focus:outline-2 focus:outline-blue" />
        {!valid && code !== '' && <p className="mt-1 text-xs text-danger">Formato esperado: 20267722F</p>}
        {error && <p role="alert" className="mt-1 text-xs text-danger">{error}</p>}
        <div className="mt-5 flex justify-end gap-2">
          <button type="button" onClick={onClose} className="rounded-xl px-4 py-2 text-slate-600 hover:bg-offwhite">Cancelar</button>
          <button type="submit" disabled={!valid} className="rounded-xl bg-blue px-4 py-2 font-semibold text-white disabled:opacity-50">Guardar y revalidar</button>
        </div>
      </form>
    </div>
  )
}
