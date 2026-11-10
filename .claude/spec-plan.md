# Plan de Fases: DocFlow

> Generado: 2026-11-10. Última actualización: 2026-11-10.
> Fuente: `SPEC_DocFlow_v1.4.md`. Este archivo es la fuente de verdad del progreso.

## Información General
- **Stack**: Tauri 2 + Rust + React + TypeScript + Vite + Tailwind (tokens de marca §5.1)
- **Usuarios**: secretaria de la Escuela de Posgrado UNI (no técnica), Windows 10/11 y Linux
- **Restricciones**: 100 % offline, sin telemetría, PDF maestro real (178 págs) fuera de Git, nunca modificar el maestro, sin rasterizar
- **Timeline estimado**: ~3 semanas (6 fases)

## Estado Actual
- **Fase activa**: ninguna — proyecto MVP completo
- **Progreso**: 6 de 6 fases completadas
- **Bloqueadores**: ninguno
- **Próximo paso concreto**: (opcional) carpeta/ZIP, códigos ficticios en spec/tests. Antes: probar instaladores en Windows/Linux y QA offline (tamaño de PDFs: descartado, el usuario optimiza con otro programa)

## Fase 1: Bootstrap y marca — COMPLETADA

**Duración estimada**: 2 días
**Objetivo**: proyecto Tauri 2 compilable con UI base de marca DocFlow y modelos compartidos.
**Complejidad**: Bajo

### Requisitos previos
- Rust, Node, dependencias WebKitGTK en Linux

### Entregables
#### Frontend
- [x] Scaffold Vite + React + TS + Tailwind
- [x] Tokens de color (navy/teal/blue/gray/offwhite) y fuente local
- [x] Layout: header con logo, barra de 4 pasos, insignia "100 % local"
- [x] `types/processing.ts` (DocumentRow, DocumentStatus, BatchSummary)
#### Backend
- [x] Scaffold `src-tauri`, `models.rs`, `errors.rs`, comando `ping`
- [x] Capabilities mínimas (dialog, opener; sin red)
#### Branding
- [x] Recortes PNG del tablero (`logo-full/mark`, `icon-1024`) + íconos Tauri generados (SVG vectorial pendiente, opcional)
#### Tests
- [x] `cargo check`, `npm run build` OK
#### Documentación
- [x] README con setup y build

### Posibles obstáculos
- Dependencias del sistema (webkit2gtk) → documentar

### Notas de avance
- 2026-11-10: Fase completa. `cargo check` y `npm run build` OK. Logo recortado de la imagen (PNG) en vez de rehacer SVG. Fuente Inter empaquetada local (@fontsource).

---

## Fase 2: Motor PDF maestro (PRIORITARIA) — COMPLETADA

**Duración estimada**: 4 días
**Objetivo**: leer maestro, extraer código/nombre por página, dividir sin rasterizar.
**Complejidad**: Alto

### Requisitos previos
- Fase 1

### Entregables
#### Backend
- [x] `pdf_text.rs` (trait `PdfTextExtractor`) y `parser.rs` (regex configurable, normalización)
- [x] `master_pdf.rs`: iteración por página, número base 1
- [x] `pdf_split.rs` con `lopdf`: una página → PDF nuevo, poda de recursos
- [x] `validator.rs`: duplicados, sin código, formato inválido, conflicto destino
- [x] `processor.rs`: temporal + publicación sin reemplazo, verificación post-escritura
#### Tests
- [x] Unit: casos 1–12 del spec §10
- [x] Fixture sintético en `tests/fixtures`
- [x] Integración local (opt-in, env var): maestro real → 178/178, hash SHA-256 intacto
#### Documentación
- [x] Decisión lopdf vs pdfium con resultados (ver notas)

### Notas de avance
- 2026-11-10: `pdf-extract` (Rust puro) lee 178 págs en ~1 s; `lopdf` divide sin rasterizar. 17 tests OK incl. maestro real: 178/178 códigos únicos y nombres, págs 1/2/178 correctas, SHA-256 del maestro intacto, 178 PDFs de 1 página en ~1.6 s. Render de la pág. 178 dividida vs original: idéntico pixel a pixel. **Hallazgo**: salidas ≈800 KB c/u (143 MB total): las fuentes TrueType vienen incrustadas completas (sin subset) en el maestro; evaluar subset/compresión en Fase 6. Sin firmas digitales en el maestro.

### Posibles obstáculos
- Extracción de texto en Rust vs Poppler → evaluar `pdf-extract`/`pdfium-render`
- Fidelidad de fuentes/recursos tras dividir → comparar visualmente muestra

### Notas de avance
-

---

## Fase 3: Vista previa y tabla de revisión — COMPLETADA

**Duración estimada**: 3 días
**Objetivo**: UI completa de análisis, estados, corrección manual y vista de página.
**Complejidad**: Medio

### Entregables
- [x] Comandos `analyze_batch`, `validate_correction`, `open_original`
- [x] `ResultsTable`, `SummaryBar`, `ManualCorrectionDialog`, filtros/búsqueda
- [x] Vista previa de página concreta
- [x] Tests de componentes y de revalidación de duplicados

### Notas de avance
- 2026-11-10: Comandos `analyze_batch`, `set_destination`, `correct_code`, `preview_page` (PDF temporal de 1 página en el visor del sistema; se borra al re-analizar y al salir), `open_original`. Estado del lote en `batch.rs`. UI: tabla con filtro/búsqueda, resumen, diálogo de corrección, arrastrar y soltar. 22 tests Rust + 6 tests vitest OK. `./run.sh` levanta la app (vite + binario). Pendiente: probarlo a mano en tu Linux.

---

## Fase 4: Generación, progreso y cancelación — COMPLETADA (carpeta/ZIP diferido)

**Duración estimada**: 3 días
**Objetivo**: `process_batch` con eventos de progreso, cancelación segura, modos carpeta/ZIP.
**Complejidad**: Medio

### Entregables
- [ ] `process_batch`, `cancel_batch`, eventos de progreso
- [ ] `scanner.rs` carpeta y ZIP seguro (anti `../`, zip bomb)
- [ ] Reproceso individual de filas fallidas
- [ ] Tests: cancelación sin archivos incompletos, ZIP malicioso

### Notas de avance
- 2026-11-10: `process_batch` (eventos `progress`), `cancel_batch`, `open_destination`; botón Generar con confirmación, barra de progreso, Cancelar y reintento de pendientes. Carpeta/ZIP (`scanner.rs`) diferido: no es necesario para el flujo del maestro.

---

## Fase 5: Reporte XLSX — COMPLETADA

**Duración estimada**: 2 días
**Objetivo**: Excel `Detalle`, `Resumen`, `Incidencias` con protección de inyección de fórmulas.
**Complejidad**: Bajo

### Entregables
- [x] `excel_report.rs` con `rust_xlsxwriter`
- [x] Formato (autofiltro, freeze, colores por estado)
- [x] Tests: una fila por página, sin desfase

### Notas de avance
- 2026-11-10: `excel_report.rs` (rust_xlsxwriter), comandos `export_report`/`open_report`, botón Exportar Excel + Abrir Excel. Test lee el xlsx con calamine: 3 hojas, 1 fila por página sin desfase, fórmulas neutralizadas. Falta abrirlo en Microsoft Excel real (QA Fase 6).

---

## Fase 6: QA, empaquetado y distribución — COMPLETADA

**Duración estimada**: 3 días
**Objetivo**: instaladores Windows (Inno Setup, ícono en escritorio) y Linux (.deb/.AppImage), CI, verificación offline.
**Complejidad**: Medio

### Entregables
- [x] GitHub Actions matriz Linux/Windows (`.github/workflows/build.yml`), agregado tras validar en Linux del usuario
- [x] Checksums SHA-256, README final
- [x] QA con maestro real en entorno local, revisión offline (sin tráfico)

### Notas de avance
- 2026-11-10: Release v0.1.0 publicada vía GitHub Actions (tags v*). Instalador Inno Setup probado por el usuario en Windows: todo OK. Pruebas del maestro real 178/178 y Excel verificadas. Instalación Linux y prueba sin red: a criterio del usuario.

---

## Timeline Total
- F1 2d · F2 4d · F3 3d · F4 3d · F5 2d · F6 3d
**TOTAL: ~3 semanas**

## Riesgos Identificados
1. Fidelidad al dividir PDF con lopdf → plan B pdfium-render, comparar muestra
2. Extracción de texto robusta en Rust → trait abstracto, plan B Poppler empaquetado
3. Firmas digitales → el maestro real no tiene; advertir si aparecen
4. Dependencias WebKitGTK en Linux → documentar

## Historial de Cambios
- 2026-11-10: Plan creado con 6 fases desde SPEC v1.4.
- 2026-11-10: Fases 1 y 2 completadas.
- 2026-11-10: Fases 4, 5 y 6 completadas; release v0.1.0.
- 2026-11-10: Fase 3 completada. Sin GitHub Actions (decisión del usuario); scripts run/build locales.
