export type DocumentStatus =
  | 'LISTO' | 'PROCESADO' | 'SIN_CODIGO' | 'FORMATO_INVALIDO'
  | 'CODIGO_AMBIGUO' | 'DUPLICADO_LOTE' | 'CONFLICTO_DESTINO'
  | 'PDF_SIN_TEXTO' | 'PDF_INVALIDO' | 'PAGINA_NO_CONSTANCIA'
  | 'MULTIPAGINA_AMBIGUA' | 'ERROR_COPIA' | 'CANCELADO'

export type SourceKind = 'MASTER_PDF' | 'PDF_FILE' | 'ZIP_ENTRY'

export interface DocumentRow {
  id: string
  sourcePath: string
  originalName: string
  sourceKind: SourceKind
  sourcePageNumber: number | null // base 1
  studentName: string | null
  extractedCode: string | null
  effectiveCode: string | null
  proposedName: string | null
  status: DocumentStatus
  observation: string | null
  correctedManually: boolean
  destinationPath: string | null
}

export interface BatchSummary {
  total: number
  processed: number
  pending: number
  errors: number
  duplicates: number
}

/** Agrupación de UI; el estado preciso se conserva en datos y Excel. */
export type StatusGroup = 'ok' | 'review' | 'error'
export const statusGroup = (s: DocumentStatus): StatusGroup =>
  s === 'LISTO' || s === 'PROCESADO' ? 'ok'
  : ['SIN_CODIGO', 'FORMATO_INVALIDO', 'CODIGO_AMBIGUO', 'DUPLICADO_LOTE', 'CONFLICTO_DESTINO',
     'PAGINA_NO_CONSTANCIA', 'MULTIPAGINA_AMBIGUA', 'CANCELADO'].includes(s) ? 'review'
  : 'error'

export interface BatchView {
  masterPath: string
  destination: string | null
  rows: DocumentRow[]
  summary: BatchSummary
}
