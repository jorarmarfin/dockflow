import type { DocumentStatus } from '../types/processing'
import { statusGroup } from '../types/processing'

export const STATUS_LABEL: Record<DocumentStatus, string> = {
  LISTO: 'Listo', PROCESADO: 'Procesado', SIN_CODIGO: 'Sin código', FORMATO_INVALIDO: 'Formato inválido',
  CODIGO_AMBIGUO: 'Código ambiguo', DUPLICADO_LOTE: 'Duplicado', CONFLICTO_DESTINO: 'Ya existe en destino',
  PDF_SIN_TEXTO: 'Sin texto', PDF_INVALIDO: 'PDF inválido', PAGINA_NO_CONSTANCIA: 'No es constancia',
  MULTIPAGINA_AMBIGUA: 'Multipágina ambigua', ERROR_COPIA: 'Error al generar', CANCELADO: 'Cancelado',
}

// Nunca solo color: cada grupo lleva icono + texto.
export const GROUP_STYLE = {
  ok: { icon: '✓', cls: 'bg-teal/10 text-teal border-teal/30' },
  review: { icon: '!', cls: 'bg-amber/15 text-amber-700 border-amber/40' },
  error: { icon: '✕', cls: 'bg-danger/10 text-danger border-danger/30' },
} as const

export const groupOf = statusGroup
