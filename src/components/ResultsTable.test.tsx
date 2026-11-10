import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ResultsTable, filterRows } from './ResultsTable'
import type { DocumentRow } from '../types/processing'

const row = (o: Partial<DocumentRow>): DocumentRow => ({
  id: 'p1', sourcePath: '/m.pdf', originalName: 'm.pdf', sourceKind: 'MASTER_PDF', sourcePageNumber: 1,
  studentName: 'ANA UNO', extractedCode: '20260001A', effectiveCode: '20260001A', proposedName: '20260001A.pdf',
  status: 'LISTO', observation: null, correctedManually: false, destinationPath: null, ...o,
})
const rows = [
  row({}),
  row({ id: 'p2', sourcePageNumber: 2, studentName: 'LUIS DOS', effectiveCode: null, extractedCode: null, proposedName: null, status: 'SIN_CODIGO' }),
  row({ id: 'p3', sourcePageNumber: 3, studentName: null, status: 'PDF_INVALIDO' }),
]

describe('filterRows', () => {
  it('filtra por grupo de estado', () => {
    expect(filterRows(rows, 'ok', '').map(r => r.id)).toEqual(['p1'])
    expect(filterRows(rows, 'review', '').map(r => r.id)).toEqual(['p2'])
    expect(filterRows(rows, 'error', '').map(r => r.id)).toEqual(['p3'])
  })
  it('busca por alumno, código o página', () => {
    expect(filterRows(rows, 'all', 'luis').map(r => r.id)).toEqual(['p2'])
    expect(filterRows(rows, 'all', '20260001').map(r => r.id)).toContain('p1')
    expect(filterRows(rows, 'all', '3').map(r => r.id)).toContain('p3')
  })
})

describe('ResultsTable', () => {
  it('muestra estado con texto y dispara acciones de fila', async () => {
    const onPreview = vi.fn(), onEdit = vi.fn()
    render(<ResultsTable rows={rows} onPreview={onPreview} onEdit={onEdit} />)
    expect(screen.getByText('Sin código')).toBeInTheDocument()
    await userEvent.click(screen.getAllByText('Ver esta página')[1])
    expect(onPreview).toHaveBeenCalledWith(rows[1])
    await userEvent.click(screen.getAllByText('Editar código')[0])
    expect(onEdit).toHaveBeenCalledWith(rows[0])
  })
  it('el filtro Error oculta las demás filas', async () => {
    render(<ResultsTable rows={rows} onPreview={() => {}} onEdit={() => {}} />)
    await userEvent.click(screen.getByRole('button', { name: 'Error' }))
    expect(screen.getByText('PDF inválido')).toBeInTheDocument()
    expect(screen.queryByText('Sin código')).not.toBeInTheDocument()
  })
})
