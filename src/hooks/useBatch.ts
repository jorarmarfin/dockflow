import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import type { BatchView } from '../types/processing'

export function useBatch() {
  const [view, setView] = useState<BatchView | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const [report, setReport] = useState<string | null>(null)
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null)

  const run = useCallback(async <T,>(fn: () => Promise<T>): Promise<T | undefined> => {
    setError(null)
    try { return await fn() } catch (e) { setError(String(e)) }
  }, [])

  const analyze = useCallback(async (source: string) => {
    setBusy(true)
    const v = await run(() => invoke<BatchView>('analyze_batch', { source }))
    if (v) { setView(v); setReport(null) }
    setBusy(false)
  }, [run])

  const pickPdf = useCallback(async () => {
    const f = await open({ multiple: false, filters: [{ name: 'PDF', extensions: ['pdf'] }] })
    if (typeof f === 'string') await analyze(f)
  }, [analyze])

  const pickDestination = useCallback(async () => {
    const d = await open({ directory: true, multiple: false })
    if (typeof d !== 'string') return
    const v = await run(() => invoke<BatchView>('set_destination', { dest: d }))
    if (v) setView(v)
  }, [run])

  const correct = useCallback(async (rowId: string, code: string) => {
    setView(await invoke<BatchView>('correct_code', { rowId, code })) // el error lo muestra el diálogo
  }, [])

  const generate = useCallback(async () => {
    setBusy(true); setProgress({ done: 0, total: 0 })
    const v = await run(() => invoke<BatchView>('process_batch'))
    if (v) setView(v)
    setProgress(null); setBusy(false)
  }, [run])
  const cancel = useCallback(() => invoke('cancel_batch'), [])
  const exportReport = useCallback(async () => {
    const p = await run(() => invoke<string>('export_report'))
    if (p) setReport(p)
  }, [run])
  const openReport = useCallback(() => run(() => invoke('open_report')), [run])
  const openDestination = useCallback(() => run(() => invoke('open_destination')), [run])

  useEffect(() => {
    let un: (() => void) | undefined
    listen<{ done: number; total: number }>('progress', e => setProgress(e.payload)).then(u => { un = u })
    return () => un?.()
  }, [])

  const preview = useCallback((rowId: string) => run(() => invoke('preview_page', { rowId })), [run])
  const openOriginal = useCallback(() => run(() => invoke('open_original')), [run])

  // Arrastrar y soltar el PDF sobre la ventana
  useEffect(() => {
    let un: (() => void) | undefined
    getCurrentWebview().onDragDropEvent(e => {
      if (e.payload.type === 'drop') {
        const pdf = e.payload.paths.find(p => p.toLowerCase().endsWith('.pdf'))
        if (pdf) void analyze(pdf)
      }
    }).then(u => { un = u }).catch(() => {})
    return () => un?.()
  }, [analyze])

  return { view, busy, error, progress, report, exportReport, openReport, generate, cancel, openDestination, pickPdf, pickDestination, correct, preview, openOriginal }
}
