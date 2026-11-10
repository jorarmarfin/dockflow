//! Extracción de código y nombre desde el texto de UNA página.
//! La regex es configurable: es una hipótesis validada con el lote real (spec §2).
use regex::Regex;
use std::collections::BTreeSet;

pub const DEFAULT_CODE_PATTERN: &str =
    r"(?i)c[oó]digo\s+de\s+alumn[oa]\s+de\s+posgrado\s*:\s*([0-9]{8}[A-Za-z])\b";
/// Formato aceptado para un código efectivo (también tras corrección manual).
pub const DEFAULT_CODE_FORMAT: &str = r"^[0-9]{8}[A-Z]$";
const NAME_PATTERN: &str = r"(?is)\bQue,\s*(?:Don|Doña|Dona|Don/Doña)\s+(.+?)\s+ha\s+sido\s+admitid[oa]";

#[derive(Debug, PartialEq, Eq)]
pub enum CodeResult {
    None,
    One(String),
    Ambiguous(Vec<String>),
}

pub struct Parser {
    code_re: Regex,
    format_re: Regex,
    name_re: Regex,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new(DEFAULT_CODE_PATTERN, DEFAULT_CODE_FORMAT).expect("regex por defecto válida")
    }
}

/// Colapsa cualquier espacio o salto de línea a un solo espacio.
pub fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl Parser {
    pub fn new(code_pattern: &str, code_format: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            code_re: Regex::new(code_pattern)?,
            format_re: Regex::new(code_format)?,
            name_re: Regex::new(NAME_PATTERN)?,
        })
    }

    /// Letra final en mayúscula, sin espacios; los ceros a la izquierda se conservan (cadena).
    pub fn normalize_code(&self, raw: &str) -> String {
        raw.trim().to_uppercase()
    }

    pub fn is_valid_code(&self, code: &str) -> bool {
        self.format_re.is_match(code)
    }

    pub fn extract_code(&self, page_text: &str) -> CodeResult {
        let text = normalize_ws(page_text);
        let found: BTreeSet<String> = self
            .code_re
            .captures_iter(&text)
            .filter_map(|c| c.get(1))
            .map(|m| self.normalize_code(m.as_str()))
            .collect();
        match found.len() {
            0 => CodeResult::None,
            1 => CodeResult::One(found.into_iter().next().unwrap()),
            _ => CodeResult::Ambiguous(found.into_iter().collect()),
        }
    }

    pub fn extract_name(&self, page_text: &str) -> Option<String> {
        let text = normalize_ws(page_text);
        let name = self.name_re.captures(&text)?.get(1)?.as_str().trim().to_string();
        (!name.is_empty() && name.len() < 120).then_some(name)
    }

    /// Indicador estructural de que la página es una constancia.
    pub fn looks_like_constancia(&self, page_text: &str) -> bool {
        let t = normalize_ws(page_text).to_uppercase();
        t.contains("CONSTANCIA DE ADMISI") || t.contains("HACE CONSTAR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> Parser { Parser::default() }

    #[test]
    fn extrae_codigo_alumno() {
        assert_eq!(p().extract_code("con código de alumno de posgrado: 20267672I"), CodeResult::One("20267672I".into()));
    }
    #[test]
    fn extrae_codigo_alumna() {
        assert_eq!(p().extract_code("con código de alumna de posgrado: 20267676D"), CodeResult::One("20267676D".into()));
    }
    #[test]
    fn tolera_saltos_mayusculas_y_sin_tilde() {
        let t = "CON  CODIGO\nDE ALUMNO  de\nposgrado:\n20267722f,";
        assert_eq!(p().extract_code(t), CodeResult::One("20267722F".into()));
    }
    #[test]
    fn no_confunde_periodo() {
        assert_eq!(p().extract_code("Periodo Académico 2026-2 constancia_2026-2-10.pdf"), CodeResult::None);
    }
    #[test]
    fn sin_frase_es_none() {
        assert_eq!(p().extract_code("texto cualquiera 20267722F"), CodeResult::None);
    }
    #[test]
    fn dos_codigos_son_ambiguos() {
        let t = "código de alumno de posgrado: 20267722F y código de alumna de posgrado: 20267731E";
        assert!(matches!(p().extract_code(t), CodeResult::Ambiguous(v) if v.len() == 2));
    }
    #[test]
    fn mismo_codigo_repetido_no_es_ambiguo() {
        let t = "código de alumno de posgrado: 20267722F ... código de alumno de posgrado: 20267722F";
        assert_eq!(p().extract_code(t), CodeResult::One("20267722F".into()));
    }
    #[test]
    fn conserva_ceros_a_la_izquierda() {
        assert_eq!(p().normalize_code(" 00001234a "), "00001234A");
    }
    #[test]
    fn valida_formato() {
        assert!(p().is_valid_code("20267722F"));
        assert!(!p().is_valid_code("2026772F"));
        assert!(!p().is_valid_code("20267722f"));
    }
    #[test]
    fn extrae_nombre_con_saltos() {
        let t = "Que, Don JUAN  CARLOS\nPEREZ GOMEZ ha sido\nadmitido en el Periodo";
        assert_eq!(p().extract_name(t).as_deref(), Some("JUAN CARLOS PEREZ GOMEZ"));
    }
    #[test]
    fn extrae_nombre_dona_con_tilde() {
        let t = "Que, Doña MARIA RÍOS PRUEBA ha sido admitida en";
        assert_eq!(p().extract_name(t).as_deref(), Some("MARIA RÍOS PRUEBA"));
    }
    #[test]
    fn detecta_constancia() {
        assert!(p().looks_like_constancia("CONSTANCIA DE ADMISIÓN 2026 - 2"));
        assert!(!p().looks_like_constancia("Página de carátula"));
    }
}
