# DocFlow

**Automatización documental local y segura.** Un PDF maestro de constancias de admisión entra; PDFs individuales `<CODIGO>.pdf` y un Excel de trazabilidad salen. Todo en el mismo equipo, sin red, sin IA en la nube, sin telemetría.

Spec: [`SPEC_DocFlow_v1.4.md`](SPEC_DocFlow_v1.4.md) · Plan y estado: [`.claude/spec-plan.md`](.claude/spec-plan.md)

## Qué hace

1. Seleccionas (o arrastras) el PDF maestro, p. ej. `FIIS CONST MAEST 2026-2.pdf` (178 páginas).
2. Eliges la carpeta destino. DocFlow lee cada página, extrae **alumno** y **código** (`código de alumno/a de posgrado: 20267722F`) y propone el nombre `20267722F.pdf`.
3. Detecta duplicados, páginas sin código, páginas que no son constancia y destinos ocupados.
4. Puedes ver una página concreta en tu visor de PDF y corregir un código (solo cambia el nombre de salida).
5. Divide el PDF **sin rasterizar** (texto y fuentes originales) y deja una constancia por archivo.

El PDF maestro nunca se modifica, mueve ni sobrescribe. El nombre final sale siempre del contenido de la página, nunca de la posición ni del nombre del archivo.

## Estado

| Fase | Contenido | Estado |
|---|---|---|
| 1 | Bootstrap Tauri + React + marca | Completa |
| 2 | Motor: lectura, extracción, división, validación | Completa |
| 3 | Interfaz de revisión (tabla, filtros, vista de página, corrección) | Completa |
| 4 | Botón Generar PDFs, progreso, cancelación | Completa (carpeta/ZIP: futuro) |
| 5 | Reporte Excel (`Detalle`, `Resumen`, `Incidencias`) | Completa |
| 6 | QA final, empaquetado y verificación offline | Completa |

La app ya **analiza, revisa y genera** los PDFs (botón *Generar PDFs* con confirmación, progreso y cancelación). El botón *Exportar Excel* guarda `reporte_procesamiento_YYYYMMDD_HHMMSS.xlsx` en la carpeta destino (hojas `Detalle`, `Resumen` e `Incidencias`; una fila por página, con número de página original, y celdas de texto protegidas contra fórmulas).

## Requisitos

- Node 20+ y Rust 1.90+
- **Linux:** `sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev build-essential libssl-dev librsvg2-dev`
- **Windows 10/11:** WebView2 (incluido en el sistema) y Visual Studio Build Tools (C++)

## Uso rápido

| Acción | Linux / Git Bash | Windows (PowerShell) |
|---|---|---|
| Levantar la app en desarrollo | `./run.sh` | `.\run.ps1` |
| Solo interfaz en el navegador (sin motor) | `./run.sh web` | `.\run.ps1 web` |
| Ejecutar pruebas | `./run.sh test` | `.\run.ps1 test` |
| Compilar instaladores | `./build.sh` | `.\build.ps1` |

La compilación es **nativa en cada sistema** (no se compila Windows desde Linux):

- Linux: `.deb` y `.AppImage`
- Windows: instalador **Inno Setup** `DocFlow-Setup-<versión>.exe` (con acceso directo en el escritorio y logo; requiere [Inno Setup 6](https://jrsoftware.org/isdl.php) instalado para compilar)

Los paquetes quedan en `dist-release/` junto con `SHA256SUMS.txt`. 

## GitHub Actions

`.github/workflows/build.yml` compila en Ubuntu 24.04 y Windows **solo** al subir un tag `v*` (release) o al ejecutarlo a mano (*Actions → Build → Run workflow*). No corre en cada push. Corre los tests, empaqueta con `build.sh` y sube los instaladores como artefactos (`docflow-linux`, `docflow-windows`).

Para publicar una release con los instaladores adjuntos:

```bash
git tag v0.1.0 && git push origin v0.1.0
```

El CI nunca usa PDFs reales: la prueba del maestro real solo corre si defines `DOCFLOW_REAL_MASTER`.

## Pruebas

```bash
cd src-tauri && cargo test --release   # 25 tests del motor (fixtures sintéticos)
npm test                               # 6 tests de interfaz (vitest)
```

Prueba contra el maestro real (solo local, nunca en Git):

```bash
DOCFLOW_REAL_MASTER="/ruta/FIIS CONST MAEST 2026-2.pdf" \
  cargo test --release real -- --nocapture
# DOCFLOW_KEEP_OUT=/carpeta  conserva las 178 salidas para inspección visual
```

Resultados verificados con el maestro real: 178 páginas, 178 códigos únicos y 178 nombres; página 1 → `20267722F`, 2 → `20267731E`, 178 → `20267672I`; 178 PDFs de una página en ~1,6 s; SHA-256 del maestro sin cambios; página dividida idéntica píxel a píxel a la original.

## Estructura

```
src/                 React + TypeScript + Tailwind (components, hooks, types)
src-tauri/src/
  parser.rs          regex configurable, normalización, nombre del alumno
  pdf_text.rs        extracción de texto (trait, backend pdf-extract)
  pdf_split.rs       una página → PDF nuevo (lopdf, sin rasterizar)
  master_pdf.rs      análisis página por página
  validator.rs       duplicados, conflictos de destino, corrección manual
  processor.rs       temporal → verificación → publicación sin reemplazo
  batch.rs           estado del lote en el backend
  commands.rs        comandos Tauri expuestos a React
branding/            tablero, logo, ícono (recortes del tablero de marca)
run.sh / build.sh    scripts Linux (y Git Bash); .ps1 equivalentes para Windows
```

## Notas conocidas

- Cada PDF generado pesa ~800 KB (≈143 MB el lote): el maestro trae las fuentes TrueType incrustadas completas y cada página se las lleva. Es un tema de tamaño, no de calidad; se optimizan con un programa externo.
- Dividir un PDF puede invalidar firmas digitales. El maestro actual no tiene firmas; si aparecen, hay que validar con los responsables.
- La regex del código (8 dígitos + letra) está validada con este lote; es configurable para otras plantillas.

## Privacidad

100 % local: sin APIs, telemetría ni servicios externos; permisos de Tauri mínimos (diálogos y abrir archivos locales). Las vistas previas se escriben en un temporal que se borra al re-analizar y al cerrar. Los PDF reales están en `.gitignore`; el repositorio usa solo fixtures sintéticos.

## Marca

Paleta: Navy `#0F172A`, Teal `#0EA5A4`, Blue `#2563EB`, Gray `#CBD5E1`, Off-white `#F8FAFC`. Íconos de la app: `npx tauri icon branding/icon-1024.png`.
