# SPEC — DocFlow: automatización documental local

**Versión:** 1.4 · **Fecha:** 2026-11-10 · **Estado:** listo para implementación del MVP  
**Plataformas objetivo:** Windows 10/11 x64 y Linux x86_64 (Ubuntu/Debian y AppImage), aplicación local y offline  
**Stack propuesto:** Tauri 2 + Rust + React + TypeScript + Vite  
**Producto:** DocFlow, herramienta de escritorio para procesar constancias de admisión exportadas desde el SGD.

## 1. Contexto y problema

La secretaria recibe del sistema externo **un único PDF multipágina** (ejemplo real `FIIS CONST MAEST 2026-2.pdf` de 178 páginas). Actualmente carga ese documento en servicios externos de IA para **dividirlo en constancias individuales y renombrarlas**; esto expone datos personales de estudiantes a terceros. Las capturas de archivos `FIIS CONST MAEST 2026-2-1.pdf`, `...-2.pdf`, etc. reflejan una etapa posterior de separación, no necesariamente la descarga original. Se requiere automatizar **ambas operaciones en el mismo equipo**, preservando el PDF maestro y con trazabilidad página → archivo de salida.

**Meta del MVP (prioridad máxima):** seleccionar **un PDF multipágina**, analizar cada página, extraer nombre y código, dividir el PDF original sin rasterizar las páginas, generar un PDF por constancia nombrado `<CODIGO>.pdf`, permitir correcciones y exportar un `.xlsx` de auditoría. **Compatibilidad secundaria:** mantener como modos adicionales la importación de carpeta de PDF individuales o ZIP.

**Fuera del MVP:** conexión directa al SGD, autenticación institucional, nube, OCR obligatorio, edición del contenido de los PDF y operaciones sobre el servidor Laravel existente.

## 2. Evidencia y patrón de documentos

Se revisaron **dos PDF individuales** y posteriormente el **PDF maestro de 178 páginas** proporcionado por el usuario:

| Muestra | Nombre de alumno en el documento | Texto clave | Código extraído |
|---|---|---|---|
| `20267672I.pdf` | (nombre omitido) | `con código de alumno de posgrado: 20267672I` | `20267672I` |
| `20267676D.pdf` | (nombre omitido) | `con código de alumna de posgrado: 20267676D` | `20267676D` |

En ambas muestras, el texto es extraíble sin OCR. Se confirmó mediante `pdftotext` que la frase y los códigos son legibles. Los archivos proporcionados **ya están nombrados por código**; los nombres correlativos usados en pantallas y pruebas son ejemplos de cómo llegaría el lote del SGD.

**Regla de extracción inicial:** buscar, de forma insensible a mayúsculas y con espacios y saltos de línea flexibles, la expresión `código de alumno de posgrado:` o `código de alumna de posgrado:`, seguida del identificador.

Ejemplo de regex orientativa (ajustar tras pruebas del extractor):

```regex
(?i)c[oó]digo\s+de\s+alumn[oa]\s+de\s+posgrado\s*:\s*([0-9]{8}[A-Za-z])\b
```

La regex anterior **es una hipótesis de validación basada solo en dos ejemplos**, no una norma institucional confirmada. Centralizarla en una configuración editable y no asumir que todos los códigos son siempre exactamente ocho dígitos y una letra sin validar el lote real. No confundir el periodo académico `2026-2`, la fecha ni otros números del documento con el código de alumno.

### 2.0. Validación del PDF maestro real (10/11/2026)

Archivo: `FIIS CONST MAEST 2026-2.pdf`. Origen: *Microsoft Print To PDF*, sin cifrado, sin firmas digitales, sin adjuntos. Encabezado de cada página: **Universidad Nacional de Ingeniería — Escuela de Posgrado — Constancia de Admisión 2026-2** (la facultad/maestría varía según constancia).

- **178 páginas**, todas con texto extraíble. Todas tienen tamaño de página 612 × 792 puntos.
- En las **178 páginas**, el patrón contextual `código de alumno/a de posgrado:` permitió reconocer **exactamente un código válido por página**.
- **178 códigos únicos**, **0 duplicados**, **0 páginas sin código** usando la expresión regular del apartado 2.
- El patrón contextual `Que, Don/Doña NOMBRE ... ha sido admitido/a en` permite extraer el **nombre en 178 de 178 páginas** (con normalización de espacios/saltos).
- Ejemplos confirmados: **página 1 → `20267722F.pdf`**, **página 2 → `20267731E.pdf`**, **página 178 → `20267672I.pdf`**. No inferir códigos por posición ni correlativo.
- El lote concreto confirma la regla de **ocho dígitos más letra**, pero esta regla sigue siendo configurable para otros formatos y años.
- Estos resultados validan la **extracción**, no constituyen una prueba de extremo a extremo del motor Rust ni de la preservación de elementos interactivos al dividir el PDF. El agente debe ejecutar las pruebas de salida descritas abajo.

**Privacidad de fixtures:** mantener el PDF real fuera del repositorio público, fixtures publicados, artefactos de CI y logs. Para pruebas automatizadas en Git, construir un PDF sintético con nombres y códigos ficticios del mismo patrón. El original puede usarse en un entorno autorizado de QA local.

### 2.1. Evidencia adicional: nombres de entrada y salida (capturas)

La captura de **origen** muestra, entre otros, estos nombres literales:

```text
FIIS CONST MAEST 2026-2-1.pdf
FIIS CONST MAEST 2026-2-2.pdf
FIIS CONST MAEST 2026-2-3.pdf
...
FIIS CONST MAEST 2026-2-10.pdf
FIIS CONST MAEST 2026-2-11.pdf
```

La captura de **resultado** muestra nombres en el formato:

```text
20267570A.pdf
20267571H.pdf
20267572D.pdf
20267573K.pdf
20267574G.pdf
20267575C.pdf
...
```

**Advertencia de trazabilidad:** las capturas muestran el *formato* antes y después, pero **no permiten comprobar** que la primera fila visible de la primera imagen corresponde a la primera fila visible de la segunda. Por tanto, **nunca inferir** `...-1.pdf -> 20267570A.pdf` por posición o correlativo. La única fuente para decidir el nuevo nombre debe ser el código leído dentro de **ese mismo PDF** (o una corrección manual registrada y validada).

**Requisito adicional de auditoría:** mantener relación uno a uno `ruta de origen -> código detectado -> nombre final -> estado` y exportarla al Excel, aunque el análisis encuentre archivos en orden alfabético, natural, por fecha u otro. Mostrar el correlativo original solo como dato informativo. **No** fabricar ni incrementar códigos consecutivos.

**Orden visual opcional:** ordenar por número al final del nombre (orden natural: `...-2.pdf` antes de `...-10.pdf`), únicamente para comodidad de la secretaria. El orden no forma parte de la lógica de extracción, validación ni renombrado.

**Casos de prueba de nombres:** espacios, guiones, prefijo `FIIS CONST MAEST`, periodo `2026-2`, correlativos de uno o varios dígitos, y archivos con nombres diferentes dentro del mismo lote. No construir el destino reemplazando el último número del nombre de origen: se construye **exclusivamente** con el código del contenido del PDF.

**Normalización:** eliminar espacios alrededor del código, convertir letra final a mayúscula, preservar ceros a la izquierda y almacenar el resultado como **cadena**, nunca como número. El nombre final esperado en el MVP es `<CODIGO>.pdf`.

## 3. Flujo principal de usuario — modo PDF multipágina (predeterminado)

1. La secretaria abre DocFlow en Windows o Linux, **sin conexión a servicios externos**.
2. Selecciona o arrastra el archivo maestro `FIIS CONST MAEST 2026-2.pdf` (o cualquier PDF compatible).
3. Elige una **carpeta destino** distinta de la ubicación del archivo maestro.
4. Pulsa **Analizar**. La app abre el archivo en modo lectura y recorre **cada página individualmente**; obtiene número de página, nombre y código, sin crear archivos todavía.
5. Muestra una tabla `Página | Archivo original | Alumno | Código | Salida propuesta | Estado | Observación`. Se detectan códigos duplicados, códigos ausentes, destinos ocupados, páginas vacías y posibles documentos de más de una página.
6. La secretaria puede **previsualizar la página específica** en el visor PDF local, corregir un código mal impreso (solo para el nombre del archivo; no se cambia el contenido) y revalidar.
7. Pulsa **Generar PDFs**. Por cada fila válida se extrae la página correspondiente a **un PDF nuevo**, sin convertirla a imagen, y se guarda como `<CODIGO>.pdf` en el destino.
8. El motor verifica cada salida (PDF legible, número de páginas esperado = 1, código que corresponde al registro, integridad razonable) y registra los resultados.
9. Se genera un `reporte_procesamiento_YYYYMMDD_HHMMSS.xlsx` con detalle, resumen e incidencias por **número de página original**. Se ofrecen acciones `Abrir carpeta` y `Abrir Excel`.

**IMPORTANTE — supuesto documental del lote:** las 178 páginas observadas corresponden a una constancia de una página por estudiante. En el MVP, el modo multipágina funciona con **1 página = 1 constancia**, pero debe comprobar indicadores de inicio/estructura. Si se detecta un documento de varias páginas, páginas sin constancia o incertidumbre, **no combinar ni partir automáticamente con suposiciones**: marcar para revisión y ofrecer una futura modalidad de rangos manuales.

**Entradas secundarias (mantener):** carpeta de PDFs y ZIP, con el flujo preexistente de analizar cada archivo individual y copiarlo renombrado, sin reescribir su contenido. Detectar modo por tipo de origen y permitir al usuario elegir explícitamente el modo si es ambiguo.

**Regla de seguridad:** nunca eliminar, mover ni sobrescribir el PDF maestro ni los archivos fuente. Si el destino contiene el nombre propuesto, bloquear la salida (`CONFLICTO_DESTINO`). No generar `pagina_42.pdf` como salida definitiva de una página sin código; marcar incidencia para corregir.

## 4. Requisitos funcionales

| ID | Requisito | Prioridad |
|---|---|---|
| RF-01 | Elegir **PDF maestro multipágina** con selector o arrastrar y soltar; carpeta/ZIP como modos alternativos | P0 |
| RF-02 | Elegir carpeta destino y validar que exista o crearla con autorización | P0 |
| RF-03 | Enumerar **páginas** del PDF maestro; en modos alternativos, enumerar PDFs individuales | P0 |
| RF-04 | Leer texto de PDF sin OCR para documentos con capa de texto | P0 |
| RF-05 | Extraer código con patrón configurable y validar formato | P0 |
| RF-06 | Mostrar vista previa sin modificar archivos | P0 |
| RF-07 | Detectar código duplicado en lote y colisión de nombre en destino | P0 |
| RF-08 | Abrir PDF original mediante visor predeterminado del sistema operativo | P0 |
| RF-09 | Corregir manualmente código, registrar corrección y revalidar fila | P0 |
| RF-10 | **Extraer cada página a un PDF independiente**, nombrar `<CODIGO>.pdf`, preservar el PDF maestro; en modo carpeta/ZIP, copiar/renombrar | P0 |
| RF-11 | Exportar XLSX con detalle, resumen, errores y motivos | P0 |
| RF-12 | Progreso por archivo y opción de cancelar sin corromper salida | P1 |
| RF-13 | Filtro por estado y búsqueda por archivo/código/alumno | P1 |
| RF-14 | Exportar solo errores como hoja independiente | P1 |
| RF-15 | Recordar última carpeta usada, sin almacenar contenido personal | P2 |
| RF-16 | OCR local opcional para PDFs escaneados | Futuro |
| RF-17 | Mostrar el correlativo original solo como referencia, y asociar cada nombre de salida al PDF fuente realmente leído | P0 |
| RF-18 | Orden natural opcional de los archivos originales en modos carpeta/ZIP | P1 |
| RF-19 | Conservar el **número de página original (base 1)** en UI, modelo y Excel | P0 |
| RF-20 | Validar que cada página corresponde a una constancia y que cada PDF exportado tenga 1 página correcta | P0 |
| RF-21 | Separar páginas **sin rasterizar**, conservar nitidez, fuentes y disposición; advertir si hay firmas digitales u otros elementos que requieren validación | P0 |
| RF-22 | Permitir vista previa local de la **página concreta**, no solo del PDF completo | P0 |
| RF-23 | Procesar completamente offline; no enviar PDF, códigos ni nombres a API/IA/telemetría | P0 |

### Estados propuestos

- `LISTO`: código único, válido y destino disponible; pendiente de procesar.
- `PROCESADO`: copia creada y verificada.
- `SIN_CODIGO`: no se encontró la frase/código esperado.
- `FORMATO_INVALIDO`: se encontró candidato, pero no cumple regla configurada.
- `CODIGO_AMBIGUO`: se detectaron varios códigos plausibles incompatibles.
- `DUPLICADO_LOTE`: dos o más archivos apuntan al mismo nombre final.
- `CONFLICTO_DESTINO`: ya existe el archivo de salida.
- `PDF_SIN_TEXTO`: no hay texto extraíble (candidato a OCR futuro).
- `PDF_INVALIDO`: el documento está corrupto, cifrado o no puede leerse.
- `PAGINA_NO_CONSTANCIA`: página sin estructura esperada de constancia.
- `MULTIPAGINA_AMBIGUA`: no es seguro asumir una constancia de una sola página.
- `ERROR_COPIA`: falla de permisos, disco, ruta u otra operación de escritura.
- `CANCELADO`: fila aún no procesada cuando el usuario canceló.

La interfaz puede agrupar esos códigos en **Correcto**, **Revisar** y **Error**, pero conservar el motivo preciso en datos y Excel.

## 5. Interfaz React

**Pantalla única del MVP**, con estas áreas:

- Encabezado: logo + “DocFlow — Automatización documental local y segura” (ver §5.1).
- Origen: opción principal `Seleccionar PDF` (también arrastrar y soltar), más opciones secundarias `Seleccionar ZIP`, `Seleccionar carpeta`.
- Destino: campo + botón `Seleccionar destino`.
- Acciones: `Analizar`, `Generar PDFs`, `Cancelar`, `Exportar Excel`.
- Indicadores: total, listos, procesados, pendientes, errores, duplicados.
- Tabla: `Página de origen | Archivo maestro | Alumno | Código | Nombre nuevo | Estado | Observación | Acciones`. En modo carpeta/ZIP, mostrar archivo fuente individual.
- Acciones de fila: `Ver esta página`, `Editar código`, `Revalidar`.
- Panel final con enlaces `Abrir destino` y `Abrir reporte`.

**UX:** deshabilitar `Procesar` antes del análisis; no bloquear toda la ventana durante lotes; avisar cuando destino = origen o destino está dentro de origen de manera que podría autoingestarse; mostrar confirmación antes de escribir; mantener las filas fallidas para reprocesarlas individualmente.

## 5.1. Identidad visual (fuente: `Tablero de marca DocFlow moderno y seguro.png`)

**Marca:** DocFlow — *Automatización documental local y segura*. Logotipo: "Doc" en Navy + "Flow" en Blue, con ícono de documento + flecha teal→azul + escudo con candado. Flujo de comunicación: **Divide PDFs → Extrae códigos → Renombra archivos → Protege la privacidad**.

| Token | Hex | Uso |
|---|---|---|
| `--navy` | `#0F172A` | Principal: texto, encabezado, fondo del ícono |
| `--teal` | `#0EA5A4` | Secundario: estado Correcto, acentos de flujo, botón secundario |
| `--blue` | `#2563EB` | Acento: botón primario, enlaces, foco |
| `--gray` | `#CBD5E1` | Soporte: bordes, separadores, deshabilitados |
| `--offwhite` | `#F8FAFC` | Fondo de la app |

Estados (adicionales a la paleta, solo para semántica): Revisar = ámbar `#F59E0B`, Error = rojo `#DC2626` (PDF del ícono del tablero). Contraste mínimo WCAG AA; nunca comunicar estado solo por color (icono + texto).

**Estilo:** limpio, tarjetas con radio 12–16 px y sombra suave, fondo off-white con degradado teal muy tenue en esquinas, tipografía sans geométrica (Inter o similar, empaquetada localmente; sin CDN/Google Fonts por requisito offline), títulos en peso 700 navy. Barra de pasos superior reutilizando los 4 pictogramas del tablero (Divide → Extrae → Renombra → Protege) como indicador de progreso. Insignia fija "100 % local · sin conexión" con candado.

**Assets:** `branding/logo.svg`, `branding/icon.svg` (cuadrado redondeado navy con documento/flecha/escudo), `branding/board.png` (referencia). Generar íconos Tauri (`.ico`, `.png`, `.icns`) desde `icon.svg`. Inicialmente se rehace el logo en SVG simplificado; el arte final puede sustituirse sin cambiar código.

## 6. Arquitectura recomendada

```text
src/                         # React + TypeScript
  components/
    SourcePicker.tsx
    DestinationPicker.tsx
    ResultsTable.tsx
    SummaryBar.tsx
    ManualCorrectionDialog.tsx
  hooks/useBatchProcessor.ts
  types/processing.ts
  App.tsx

src-tauri/
  src/
    main.rs
    lib.rs
    commands.rs                # API invocada desde React
    scanner.rs                 # Enumeración carpeta / lectura ZIP (modos secundarios)
    master_pdf.rs              # Iteración sobre páginas de un PDF multipágina
    pdf_split.rs               # Creación de PDF de una página desde el maestro
    pdf_text.rs                # Abstracción de extracción de texto
    parser.rs                  # Regex, normalización, datos opcionales
    validator.rs               # Reglas y detección de conflictos
    processor.rs               # Copias seguras y estado por fila
    excel_report.rs            # XLSX y resumen
    models.rs                  # Tipos serializables
    errors.rs
  capabilities/                # Permisos mínimos de Tauri
  tauri.conf.json
```

### Motor de lectura PDF

Usar una abstracción Rust `PdfTextExtractor` para no acoplar la app a una sola librería. Evaluar una biblioteca Rust de extracción textual compatible con Windows; comprobar el orden de lectura y caracteres acentuados con las dos muestras. **No asumir que `lopdf` por sí solo resuelve todos los PDFs**: es útil para estructura, pero la extracción robusta de texto puede requerir un motor adicional. Si se decide integrar un ejecutable local como Poppler/`pdftotext`, empaquetarlo explícitamente con el instalador de Windows y validar licencias y distribución. No depender de que la secretaria tenga herramientas CLI instaladas.

**Separación PDF:** elegir y comprobar una biblioteca capaz de extraer páginas a documentos independientes preservando su estructura (no hacer capturas, ni crear PDFs a partir de PNG/JPEG, ni reconstruir texto por OCR). Evaluar compatibilidad y licencias de bibliotecas tipo PDFium u otras implementaciones adecuadas para Rust/Windows; si se usa una biblioteca con DLL, empaquetarla y validar arquitectura/versiones. Verificar que la exportación mantenga gráficos, sellos y cualquier objeto visual del original. **Firmas digitales:** separar páginas puede invalidar firmas del documento original o impedir su verificación. Detectar firmas, avisar y requerir validación funcional/legal de las salidas; no prometer preservación criptográfica de firmas al dividir.

Para XLSX, evaluar `rust_xlsxwriter` u otra biblioteca Rust mantenida. Para ZIP, biblioteca Rust con protección frente a entradas peligrosas. Registrar versiones concretas y licencias en el repositorio.

### Contrato inicial Tauri (orientativo)

```ts
type DocumentStatus =
  | 'LISTO' | 'PROCESADO' | 'SIN_CODIGO' | 'FORMATO_INVALIDO'
  | 'CODIGO_AMBIGUO' | 'DUPLICADO_LOTE' | 'CONFLICTO_DESTINO'
  | 'PDF_SIN_TEXTO' | 'PDF_INVALIDO' | 'PAGINA_NO_CONSTANCIA'
  | 'MULTIPAGINA_AMBIGUA' | 'ERROR_COPIA' | 'CANCELADO';

type DocumentRow = {
  id: string;
  sourcePath: string;
  originalName: string;
  sourceKind: 'MASTER_PDF' | 'PDF_FILE' | 'ZIP_ENTRY';
  sourcePageNumber: number | null; // 1-based, obligatorio en modo MASTER_PDF
  studentName: string | null;
  extractedCode: string | null;
  effectiveCode: string | null;
  proposedName: string | null;
  status: DocumentStatus;
  observation: string | null;
  correctedManually: boolean;
  destinationPath: string | null;
};

type BatchSummary = {
  total: number;
  processed: number;
  pending: number;
  errors: number;
  duplicates: number;
};
```

**Comandos sugeridos:** `analyze_batch(source)` (incluye `sourceType` y `mode`), `validate_correction(batch_id, row_id, code)`, `process_batch(batch_id, destination)`, `export_report(batch_id, output_path)`, `open_original(row_id)`, `cancel_batch(batch_id)`. Mantener el estado del lote en el backend y exponer eventos de progreso; no confiar en rutas de archivo proporcionadas directamente por componentes sin validarlas.

## 7. Algoritmo de análisis y generación

```text
ANALIZAR(origen, modo):
  validar fuente, permisos, tipo y límites razonables
  si modo == MASTER_PDF:
    abrir PDF maestro en lectura
    para page_index desde 0 hasta page_count-1:
      extraer texto EXCLUSIVAMENTE de esta página
      detectar estructura de constancia y candidatos a código
      extraer nombre si es inequívoco; validar código
      registrar sourcePageNumber = page_index + 1
      proponer nombre <CODIGO>.pdf
  si modo == FOLDER o ZIP:
    enumerar PDFs de forma segura
    analizar texto de cada archivo individual
  detectar códigos repetidos en TODO el lote
  validar carpeta destino y conflictos preexistentes
  mostrar previa de páginas / archivos y estados, sin escribir aún

GENERAR(destino):
  validar rutas, no-sobrescritura, espacio, acceso y conflictos otra vez
  para cada fila LISTO:
    si MASTER_PDF:
      copiar la página exacta (índice sourcePageNumber-1)
      a un nuevo documento PDF de una página SIN RASTERIZAR
    si FOLDER/ZIP:
      copiar PDF individual sin alterar su contenido
    escribir archivo temporal privado y validar salida PDF
    publicar con nombre <CODIGO>.pdf mediante operación sin reemplazo
    registrar página original, archivo resultante y estado
  generar XLSX por página/archivo con resumen e incidencias
```

**Comprobación de calidad:** cada salida del maestro debe abrir como PDF independiente, contener una sola página, conservar el tamaño y contenido visual esperado y coincidir con el código previsto para la página fuente. El resultado debe ser **178 PDFs de una página** para el lote real si todas las validaciones pasan; no crear PDFs definitivos para filas fallidas.

**Nota de firmas y anotaciones:** detectar advertencias sobre firmas, formularios, anotaciones, enlaces o recursos especiales al copiar páginas y revisar equivalencia; la extracción de páginas puede invalidar firmas criptográficas, incluso aunque el aspecto se conserve. Priorizar la exactitud documental frente a la velocidad.

**Nota ZIP:** preferir leer o extraer a un directorio temporal controlado; rechazar `../`, rutas absolutas, enlaces simbólicos peligrosos, bombas ZIP y entradas demasiado grandes. Limpiar temporales al cerrar/cancelar. No permitir que un ZIP sobrescriba archivos arbitrarios del equipo.

**Correcciones manuales:** modificar solo el código efectivo para el nombre de salida, no el contenido del PDF maestro. Registrar el código leído y el ingresado para auditoría local. Si una corrección crea duplicado, bloquear el procesamiento de esas filas hasta resolver el conflicto. En modo PDF maestro, localizar y mostrar la **página original exacta** al corregir.

## 8. Especificación de Excel

Nombre sugerido: `reporte_procesamiento_YYYYMMDD_HHMMSS.xlsx`.

**Hoja `Detalle`:**

| Columna | Contenido |
|---|---|
| N° | Secuencia del lote |
| Archivo original | Nombre del PDF maestro o PDF individual de origen |
| Página del maestro | Número de página de origen (1..N), obligatorio en modo PDF multipágina |
| Tipo de origen | `MASTER_PDF`, `PDF_FILE` o `ZIP_ENTRY` |
| Ruta relativa de origen | Subcarpeta o entrada ZIP |
| Alumno | Nombre extraído si es confiable; en otro caso vacío |
| Código extraído | Valor detectado automáticamente |
| Código final | Valor definitivo, incluida corrección manual |
| Archivo destino | Nombre `<CODIGO>.pdf` si se generó |
| Estado | Estado final del procesamiento |
| Observación | Motivo concreto del error o advertencia |
| Corrección manual | Sí/No |
| Fecha y hora | Marca de procesamiento local |

**Hoja `Resumen`:** modo de entrada, número total de páginas de origen, fecha, origen (sin información sensible innecesaria), destino, total detectados, procesados, sin código, formato inválido, duplicados, conflictos, PDFs ilegibles y errores de copia.

**Hoja `Incidencias`:** filas que no terminaron en `PROCESADO`, con el motivo y acción recomendada (`Revisar PDF`, `Corregir código`, `Cambiar destino`, etc.).

Formato: autofiltro, encabezados visibles, anchos legibles, inmovilizar primera fila, colores sobrios según estado. **Protección contra fórmula inyectada:** escribir los campos provenientes de PDF/ZIP como texto y neutralizar cadenas que Excel podría interpretar como fórmulas. No incluir contenido completo de los PDF en el Excel.

## 9. Seguridad y privacidad

- Funcionamiento **100 % local**; sin telemetría, API web ni subida de documentos. **No usar servicios de IA web, PDF en línea, APIs de OCR remotas, modelos que transmitan documentos ni analíticas con datos.**
- **El PDF maestro contiene 178 constancias con datos personales identificables**. Probar siempre que el equipo no transmite estos datos; no añadir servicios externos para análisis, logging o diagnóstico.
- Los PDF incluyen datos personales; permisos mínimos, nada de logs con texto completo, nombres o códigos salvo cuando sea necesario para un reporte local autorizado.
- El origen es siempre de solo lectura; exportar páginas a un destino seleccionado; nunca alterar el maestro.
- No enviar el PDF maestro a repositorios públicos, CI remotos ni plataformas externas de depuración. Usar pruebas sintéticas en el repositorio.
- Evitar path traversal y sobreescritura tanto en ZIP como en la exportación.
- Validar nombres de archivo y rutas según restricciones de Windows y Linux.
- Los temporales deben limpiarse; no retener copias en directorios públicos.
- No ejecutar scripts, macros, links ni archivos contenidos en los ZIP.
- Proporcionar mensajes comprensibles y registrar errores técnicos sin exponer datos sensibles.

## 10. Pruebas y criterios de aceptación

### Pruebas unitarias mínimas

1. Extraer `20267672I` de la frase `código de alumno de posgrado: 20267672I`.
2. Extraer `20267676D` de `código de alumna de posgrado: 20267676D`.
2a. Aceptar como entrada `FIIS CONST MAEST 2026-2-1.pdf` y `FIIS CONST MAEST 2026-2-10.pdf` sin confundir el periodo ni el correlativo con el código.
2b. Al intercambiar intencionalmente los contenidos de dos archivos correlativos de prueba, los nombres de salida deben seguir al **contenido**, no a la posición o al nombre original.
3. Aceptar saltos de línea/espacios y variantes de mayúsculas en la frase.
4. No confundir `2026-2` con código de alumno.
5. Marcar error cuando no exista frase o código, sin inventar valores.
6. Detectar duplicados en el lote antes de copiar.
7. No sobrescribir destino preexistente.
8. Corregir código manualmente y revalidar duplicados.
9. Manejar PDF cifrado, sin texto y corrupto sin detener el lote.
10. Rechazar ZIP con entradas `../` y nombres no seguros.
11. Emitir XLSX con filas, conteos y estados correctos.
12. Cancelar sin dejar archivos finales incompletos.

### Pruebas específicas del documento maestro real

1. Leer `FIIS CONST MAEST 2026-2.pdf` y confirmar **178 páginas** sin crear archivos durante `Analizar`.
2. Extraer **178 códigos, todos únicos**, uno por página, y **178 nombres** con las reglas observadas. Cualquier discrepancia deberá registrarse como incidencia y detener la aceptación del lote.
3. Comprobar correspondencia exacta: página 1 → `20267722F.pdf`, página 2 → `20267731E.pdf`, página 178 → `20267672I.pdf`.
4. Generar **178 archivos PDF independientes**, cada uno de **1 página**, sin perder legibilidad ni rasterizar el contenido.
5. Reabrir cada PDF generado, extraer su texto y confirmar que el código interior coincide con el nombre final. Comparar tamaño y contenido visible de una muestra representativa con páginas originales.
6. Comprobar que el archivo maestro mantiene el mismo hash SHA-256 antes y después del procesamiento.
7. Insertar (en un **fixture sintético**, no modificar el original): página sin código, página con código duplicado, página escaneada, dos códigos, página no constancia y contenido con saltos de línea; verificar incidencias por página sin desalinear las restantes.
8. Probar vista previa/edición de una sola página y reprocesamiento aislado, conservando sus números originales.
9. Validar que el XLSX tenga una fila por página y correspondencia sin desfase (`sourcePageNumber` 1-based).
10. Verificar que el instalador funcione sin conexión ni Python/CLI externos y que no envíe tráfico documental.

### Pruebas de integración

- Procesar el PDF maestro de 178 páginas con el flujo de división y extracción; validar página, código y salida para todo el lote.
- Procesar los dos PDF individuales aportados (modo secundario): salida esperada `20267672I.pdf` y `20267676D.pdf`, sin modificar los originales.
- Procesar carpeta con nombres genéricos `constancia_001.pdf`, `constancia_002.pdf` que contengan el texto de las muestras.
- Procesar ZIP con subcarpetas y archivos ajenos a PDF; ignorar los ajenos y reportar cantidad de PDF encontrada.
- Probar al menos un lote real de ~178 PDFs antes de dar por válida la expresión regular.
- Probar en Windows 10/11 limpio y en Ubuntu LTS limpio, sin Python ni Poppler instalados en el sistema; documentar dependencias nativas Linux (WebKitGTK y otras requeridas por Tauri).

### Definición de terminado (DoD)

- Instalador de Windows y distribución Linux ejecutan el programa offline.
- Selección de **PDF maestro**, carpeta o ZIP y destino desde la UI.
- La vista previa expone claramente el código y las incidencias.
- El maestro real de 178 páginas pasa la detección y división en 178 PDFs de una página, con verificación de código interno.
- Las dos muestras individuales pasan la extracción y generación de copias.
- Ningún archivo fuente se modifica.
- Códigos duplicados y colisiones quedan bloqueados y reportados.
- Se puede corregir una fila y reprocesar solo los pendientes.
- Excel con `Detalle`, `Resumen` e `Incidencias` abre correctamente en Microsoft Excel.
- Pruebas automatizadas y pasos de compilación documentados.

## 11. Plan de implementación para el agente

**Fase 1 — Bootstrap:** crear Tauri 2/React/TS; configurar toolchain Rust y builds nativos Windows/Linux; preparar interfaz base y modelos.

**Fase 2 — Motor PRIORITARIO:** lectura del maestro de 178 páginas, extracción **por página**, validación contextual y división a PDFs individuales sin rasterizar. Probar 178/178 localmente. Luego agregar lector carpeta/ZIP como compatibilidad secundaria.

**Fase 3 — Vista previa:** tabla, estados, duplicados, acceso a PDF, corrección manual y revalidación.

**Fase 4 — Salida:** exportar página individual por cada código válido, escribir archivos temporales con publicación segura, prevención de colisiones, progreso y cancelación. Mantener copia/renombrado para modos secundarios.

**Fase 5 — XLSX:** generar hojas `Detalle`, `Resumen`, `Incidencias`; compatibilidad Excel y protección de celdas.

**Fase 6 — QA y distribución:** lote maestro real de 178 páginas en entorno local autorizado, verificación de 178 PDFs, paquetes Windows/Linux, README, revisión de privacidad offline y checklist final.

### Primera instrucción sugerida para el agente

> Implementa el MVP de `SPEC_DocFlow_v1.4.md` empezando por el **PDF maestro multipágina**. Vertical slice: seleccionar el PDF de 178 páginas, extraer código y nombre por página, mostrar la tabla con número de página, **dividir cada constancia en PDF de una página sin rasterizar**, renombrar `<CODIGO>.pdf` y verificar que código interior y nombre coincidan. No modificar el maestro ni sobrescribir destinos. Genera Excel con correspondencia página → alumno → código → archivo → estado. Agrega corrección manual, incidencias, carpeta/ZIP como modos secundarios y empaquetado Windows y Linux. Todo offline, sin IA en la nube ni telemetría. Usa fixtures sintéticos en Git y valida el PDF real únicamente en entorno autorizado.

## 12. Compilación multiplataforma y distribución — obligatorio (v1.4)

- **Base de código única:** React + TypeScript + Tauri 2 + Rust; sin rutas, comandos ni separadores de Windows codificados directamente. Usar `PathBuf`, APIs de Tauri y diálogos nativos.
- **Windows:** target x86_64; generar instalador `.exe` mediante NSIS. MSI puede ser opcional; no confundir un `.exe` instalador con el ejecutable portable.
- **Linux:** target x86_64; generar `.deb` para Ubuntu/Debian y `.AppImage` para distribuciones compatibles. Documentar las dependencias del entorno gráfico/WebKitGTK y limitaciones de cada distribución.
- **Compilación nativa por sistema:** job de Windows en runner Windows y job de Linux en runner Ubuntu. No asumir que desde Ubuntu se puede producir directamente un instalador Windows sin toolchain y dependencias adicionales.
- **Sin CI/CD (decisión del usuario):** no se crean GitHub Actions. Se compila localmente con `build.sh` (Linux y Git Bash) o `build.ps1` (Windows nativo); `run.sh`/`run.ps1` levantan el proyecto en desarrollo.
- **Funcionamiento offline real:** sin API remota, telemetría, servicios externos de OCR ni chequeo automático que transmita documentos. Las compilaciones deben incluir cualquier librería necesaria para manipular PDF y generar XLSX; Linux puede necesitar dependencias de sistema documentadas.
- **Seguridad de empaquetado:** no incluir PDFs reales, reportes de estudiantes ni logs sensibles en instaladores, repositorio, CI o releases. Registrar checksums SHA-256 de paquetes, y considerar firma de código en una fase posterior.
- **Compatibilidad de rutas:** rutas con tildes, eñes, espacios y nombres extensos; diálogo de selección y apertura del PDF original tanto en Windows como Linux. Duplicados en sistemas de archivos con sensibilidad de mayúsculas diferente.
- **Criterios de aceptación por sistema:** instalación, carga de PDF maestro, análisis 178 páginas, exportación de constancias, Excel, apertura de carpeta, cancelación y ausencia de red en Windows 10/11 y Ubuntu LTS.
- **Comandos de referencia:** `npm run tauri build` ejecutado nativamente en cada plataforma, con `bundle.targets` acorde a ese sistema y scripts reproducibles documentados en README.

## 13. Decisiones pendientes / supuestos

- **Formato institucional del código:** el PDF maestro real de 178 páginas confirmó 178 coincidencias con ocho dígitos más una letra; mantener configurable para otras plantillas.
- **Nombre del alumno:** el patrón observado permitió extraerlo en 178/178 páginas; no bloquear renombrado cuando el código sea válido pero el nombre no se pueda extraer.
- **Subcarpetas en salida:** por defecto aplanar todos los PDFs en la carpeta destino; los duplicados se bloquean.
- **Documentos escaneados:** marcar `PDF_SIN_TEXTO`; OCR puede añadirse después.
- **Contenido distinto al de las dos muestras:** tratar como incidencia, no renombrar con heurísticas de baja confianza.
- **Archivos ya renombrados:** se consideran candidatos válidos, pero se debe verificar el código **interno**, nunca confiar solo en el nombre de archivo.
- **Mapeo entre capturas:** el PDF maestro suministrado permite establecer la correspondencia real por página y código. No deducir correspondencias basándose en el orden de las capturas.
- **Regla de una página por constancia:** confirmada para este lote visual/textualmente; no extrapolarla sin comprobación a otras plantillas o documentos.
- **Librería de división PDF (decidido v1.4):** usar `lopdf` (Rust puro, sin DLL) con poda de recursos por página y validación contra el maestro real; `pdfium-render` queda como plan B si falla fidelidad.
- **Firma digital / validez documental:** confirmar con responsables de la UNI si la división afecta requisitos de firma, seguridad o aceptación del documento; no garantizar conservación criptográfica de firmas.

---

**Principio del producto:** **un PDF maestro entra; constancias individuales correctamente nombradas y un Excel de trazabilidad salen**, sin exponer datos de estudiantes a servicios externos ni modificar originales.
