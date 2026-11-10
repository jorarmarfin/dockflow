import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import logoMark from './assets/logo-mark.png'
import { useBatch } from './hooks/useBatch'
import { SummaryBar } from './components/SummaryBar'
import { ResultsTable } from './components/ResultsTable'
import { ManualCorrectionDialog } from './components/ManualCorrectionDialog'
import type { DocumentRow } from './types/processing'

const STEPS = ['Divide PDFs', 'Extrae códigos', 'Renombra archivos', 'Protege la privacidad']
const btn = 'rounded-xl px-5 py-2.5 font-semibold disabled:opacity-50'

export default function App() {
  const [engine, setEngine] = useState('…')
  const [editing, setEditing] = useState<DocumentRow | null>(null)
  const { view, busy, error, progress, report, exportReport, openReport, generate, cancel, openDestination, pickPdf, pickDestination, correct, preview, openOriginal } = useBatch()

  useEffect(() => {
    invoke<string>('ping').then(setEngine).catch(() => setEngine('sin motor (modo web)'))
  }, [])

  const [confirming, setConfirming] = useState(false)
  const processed = view?.summary.processed ?? 0
  const pending = view?.summary.pending ?? 0
  const step = !view ? 0 : processed > 0 ? 3 : view.destination ? 2 : 1

  return (
    <div className="mx-auto max-w-6xl space-y-5 p-6">
      <header className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <img src={logoMark} alt="" className="h-12 w-auto" />
          <div>
            <h1 className="text-2xl font-bold leading-none">Doc<span className="text-blue">Flow</span></h1>
            <p className="text-sm text-slate-600">Automatización documental local y segura</p>
          </div>
        </div>
        <span className="rounded-full border border-teal/40 bg-white px-3 py-1 text-sm font-medium text-teal">
          🔒 100 % local · sin conexión
        </span>
      </header>

      <ol className="grid grid-cols-4 gap-3 rounded-card border border-gray/60 bg-white/70 p-4">
        {STEPS.map((s, i) => (
          <li key={s} aria-current={i === step ? 'step' : undefined} className="flex items-center gap-2 text-sm font-semibold">
            <span className={`grid h-7 w-7 place-items-center rounded-full ${i <= step ? 'bg-navy text-white' : 'bg-gray/60 text-slate-600'}`}>{i + 1}</span>
            {s}
          </li>
        ))}
      </ol>

      <section className="flex flex-wrap items-center gap-3 rounded-card border border-gray/60 bg-white p-4 shadow-sm">
        <button onClick={pickPdf} disabled={busy} className={`${btn} bg-blue text-white`}>
          {busy ? 'Analizando…' : view ? 'Cambiar PDF' : 'Seleccionar PDF'}
        </button>
        <button onClick={pickDestination} disabled={!view} className={`${btn} border border-teal text-teal`}>Seleccionar destino</button>
        <button onClick={() => setConfirming(true)} disabled={!view?.destination || pending === 0 || busy}
          title={!view?.destination ? 'Primero elige la carpeta destino' : undefined}
          className={`${btn} bg-navy text-white`}>
          Generar PDFs{pending > 0 ? ` (${pending})` : ''}
        </button>
        <button onClick={exportReport} disabled={!view?.destination || busy}
          title={!view?.destination ? 'Primero elige la carpeta destino' : 'Guarda el reporte .xlsx en la carpeta destino'}
          className={`${btn} border border-navy text-navy`}>
          Exportar Excel
        </button>
        {progress && <button onClick={cancel} className={`${btn} border border-danger text-danger`}>Cancelar</button>}
        <div className="min-w-0 flex-1 text-sm text-slate-600">
          {view ? (
            <>
              <p className="truncate"><b>Origen:</b> {view.masterPath}{' '}
                <button onClick={openOriginal} className="text-blue underline">abrir</button></p>
              <p className="truncate"><b>Destino:</b> {view.destination ?? 'sin elegir'}</p>
            </>
          ) : 'Selecciona o arrastra aquí el PDF maestro de constancias.'}
        </div>
      </section>

      {progress && (
        <div role="progressbar" aria-valuenow={progress.done} aria-valuemax={progress.total}
          className="rounded-card border border-gray/60 bg-white p-4 shadow-sm">
          <p className="mb-2 text-sm font-medium">Generando… {progress.done} de {progress.total || pending}</p>
          <div className="h-2 overflow-hidden rounded-full bg-gray/40">
            <div className="h-full bg-teal transition-all" style={{ width: `${progress.total ? (100 * progress.done) / progress.total : 0}%` }} />
          </div>
        </div>
      )}

      {!busy && processed > 0 && (
        <div className="flex flex-wrap items-center gap-3 rounded-card border border-teal/40 bg-teal/10 p-4">
          <span className="font-semibold text-teal">✓ {processed} PDFs generados</span>
          {pending > 0 && <span className="text-sm text-amber-700">{pending} pendientes (puedes volver a generar)</span>}
          <button onClick={openDestination} className="ml-auto rounded-xl bg-teal px-4 py-2 font-semibold text-white">Abrir carpeta destino</button>
        </div>
      )}

      {report && (
        <div className="flex flex-wrap items-center gap-3 rounded-card border border-blue/30 bg-blue/5 p-4 text-sm">
          <span className="font-semibold text-blue">✓ Reporte exportado</span>
          <span className="min-w-0 flex-1 truncate text-slate-600">{report}</span>
          <button onClick={openReport} className="rounded-xl bg-blue px-4 py-2 font-semibold text-white">Abrir Excel</button>
        </div>
      )}

      {error && <p role="alert" className="rounded-xl border border-danger/30 bg-danger/10 px-4 py-2 text-sm text-danger">{error}</p>}

      {view && (
        <>
          <SummaryBar s={view.summary} />
          <ResultsTable rows={view.rows} onPreview={r => void preview(r.id)} onEdit={setEditing} />
        </>
      )}

      {confirming && view && (
        <div role="dialog" aria-modal="true" aria-label="Confirmar generación" className="fixed inset-0 z-10 grid place-items-center bg-navy/50 p-4">
          <div className="w-full max-w-md rounded-card bg-white p-6 shadow-xl">
            <h2 className="text-lg font-bold">¿Generar {pending} PDFs?</h2>
            <p className="mt-2 break-all text-sm text-slate-600">
              Se crearán archivos <code>CÓDIGO.pdf</code> de una página en <b>{view.destination}</b>.
              No se modifica el PDF maestro ni se sobrescribe nada existente.
            </p>
            <div className="mt-5 flex justify-end gap-2">
              <button onClick={() => setConfirming(false)} className="rounded-xl px-4 py-2 text-slate-600 hover:bg-offwhite">Cancelar</button>
              <button onClick={() => { setConfirming(false); void generate() }} className="rounded-xl bg-navy px-4 py-2 font-semibold text-white">Generar</button>
            </div>
          </div>
        </div>
      )}

      {editing && (
        <ManualCorrectionDialog row={editing} onClose={() => setEditing(null)}
          onSave={async code => { await correct(editing.id, code); setEditing(null) }} />
      )}

      <footer className="text-center text-xs text-slate-500">Motor: {engine}</footer>
    </div>
  )
}
