import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ManualCorrectionDialog } from './ManualCorrectionDialog'
import type { DocumentRow } from '../types/processing'

const row = {
  id: 'p1', sourcePageNumber: 1, studentName: 'ANA', effectiveCode: null, extractedCode: null,
} as unknown as DocumentRow

describe('ManualCorrectionDialog', () => {
  it('bloquea guardar con formato inválido y acepta uno válido', async () => {
    const onSave = vi.fn().mockResolvedValue(undefined)
    render(<ManualCorrectionDialog row={row} onSave={onSave} onClose={() => {}} />)
    const save = screen.getByRole('button', { name: /guardar/i })
    expect(save).toBeDisabled()
    await userEvent.type(screen.getByRole('textbox'), '2026')
    expect(save).toBeDisabled()
    await userEvent.clear(screen.getByRole('textbox'))
    await userEvent.type(screen.getByRole('textbox'), '20267722f')
    await userEvent.click(save)
    expect(onSave).toHaveBeenCalledWith('20267722f')
  })
  it('muestra el error del backend', async () => {
    const onSave = vi.fn().mockRejectedValue('Duplicado')
    render(<ManualCorrectionDialog row={row} onSave={onSave} onClose={() => {}} />)
    await userEvent.type(screen.getByRole('textbox'), '20267722F')
    await userEvent.click(screen.getByRole('button', { name: /guardar/i }))
    expect(await screen.findByRole('alert')).toHaveTextContent('Duplicado')
  })
})
