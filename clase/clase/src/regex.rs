use std::collections::{HashSet, VecDeque};
use std::str::Chars;

use crate::caracter::Caracter;
use crate::clase_char::ClaseChar;
use crate::errors::Error;
use crate::repeticion::Repeticion;
use crate::paso_evaluado::PasoEvaluado;
use crate::paso_regex::PasoRegex;

const CORCHETE_ABIERTO: char = '[';
const CORCHETE_CERRADO: char = ']';
const LLAVE_ABIERTA: char = '{';
const LLAVE_CERRADA: char = '}';
const ASTERISCO: char = '*';
const INTERROGACION: char = '?';
const MAS: char = '+';
const PUNTO: char = '.';
const BARRA: char = '\\';
const DOLAR: char = '$';
const CARET: char = '^';
const INDICADOR_CLASE: char = ':';
const SEPARADOR_RANGO: char = '-';
const FUNCION_OR: char = '|';

///este struct representa una expresion regular
pub struct Regex {
    pasos: Vec<PasoRegex>,
}

fn obtener_auxiliar(chars_iter: &mut Chars<'_>) -> (Vec<char>, bool, bool) {
    let mut cantidad_llaves = 0;
    let mut hay_clase = false;
    let mut es_negado = false;
    let mut auxiliar: Vec<char> = Vec::new();

    while let Some(c) = chars_iter.next() {
        match c {
            CORCHETE_CERRADO if cantidad_llaves == 1 || c == CORCHETE_CERRADO && !hay_clase => break,
            CORCHETE_CERRADO => cantidad_llaves += 1,
            CARET => es_negado = true,
            INDICADOR_CLASE => continue,
            CORCHETE_ABIERTO => hay_clase = true,
            _ => auxiliar.push(c),
        }
    }
    (auxiliar, hay_clase, es_negado)
}

fn determinar_contenido_a_evaluar(auxiliar: Vec<char>) -> Result<HashSet<char>, Error> {
    let mut contenido: HashSet<char> = HashSet::new();

    for i in 0..auxiliar.len() {
        if auxiliar[i] == SEPARADOR_RANGO {
            if let (Some(inicio), Some(fin)) = (auxiliar.get(i - 1), auxiliar.get(i + 1)) {
                contenido.extend(*inicio..=*fin);
            }
        } else if auxiliar[i] == FUNCION_OR {
            return Err(Error::CaracterNoProcesable);
        } else {
            contenido.insert(auxiliar[i]);
        }
    }
    Ok(contenido)
}

fn conseguir_lista(chars_iter: &mut Chars<'_>) -> Result<(ClaseChar, bool), Error> {
    let (auxiliar, hay_clase, es_negado) = obtener_auxiliar(chars_iter);

    if hay_clase {
        let class: String = auxiliar.iter().collect();

        match class.to_string().as_str() {
            "alpha" => return Ok((ClaseChar::Alpha, es_negado)),
            "alnum" => return Ok((ClaseChar::Alnum, es_negado)),
            "digit" => return Ok((ClaseChar::Digit, es_negado)),
            "lower" => return Ok((ClaseChar::Lower, es_negado)),
            "upper" => return Ok((ClaseChar::Upper, es_negado)),
            "space" => return Ok((ClaseChar::Space, es_negado)),
            "punct" => return Ok((ClaseChar::Punct, es_negado)),
            _ => {}
        }
    }

    let contenido = determinar_contenido_a_evaluar(auxiliar);

    match contenido {
        Ok(content) => Ok((ClaseChar::Simple(content), es_negado)),
        Err(error) => Err(error),
    }
}

pub fn agregar_pasos(steps: &mut Vec<PasoRegex>, chars_iter: &mut Chars<'_>,) -> Result<Vec<PasoRegex>, Error> {
    while let Some(c) = chars_iter.next() {
        let step = match c {

            PUNTO => Some(PasoRegex {
                repeticiones: Repeticion::Exacta(1, false),
                caracter_interno: Caracter::Comodin,
            }),

            'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' => Some(PasoRegex {
                repeticiones: Repeticion::Exacta(1, false),
                caracter_interno: Caracter::Literal(c),
            }),

            LLAVE_ABIERTA => { 
                match steps.last_mut() {
                    Some(last) => {
                        if last.caracter_interno == Caracter::Comodin
                            && last.repeticiones == Repeticion::Alguna(false) {
                            return Err(Error::CaracterNoProcesable);
                        } else {
                            if let Some(last) = steps.last_mut() {
                                let mut contenido: Vec<char> = Vec::new();
                                let mut rangos: Vec<usize> = Vec::new();
                                while let Some(c) = chars_iter.next() {
                                    if c == ',' {
                                        contenido.push(c);
                                    } else if c == LLAVE_CERRADA {
                                        break;
                                    } else {
                                        contenido.push(c);
                                        match c.to_string().parse::<usize>() {
                                            Ok(cant) => rangos.push(cant),
                                            Err(_) => return Err(Error::CaracterNoProcesable),
                                        }
                                    }
                                }
            
                                if contenido.len() >= 2 {
                                    if contenido[0] == ',' {
                                        last.repeticiones = Repeticion::Rango {
                                            min: None,
                                            max: Some(rangos[0]),
                                        };
                                    } else if contenido[contenido.len() - 1] == ',' {
                                        last.repeticiones = Repeticion::Rango {
                                            min: Some(rangos[0]),
                                            max: None,
                                        };
                                    } else {
                                        last.repeticiones = Repeticion::Rango {
                                            min: Some(rangos[0]),
                                            max: Some(rangos[1]),
                                        };
                                    }
                                } else if contenido.len() == 1 && contenido[0].is_ascii_digit() {
                                    last.repeticiones = Repeticion::Exacta(rangos[0], false);
                                } else {
                                    return Err(Error::CaracterNoProcesable);
                                }
                            };
                        }
                    }
                    None => {}
                }
                None
            }

            CORCHETE_ABIERTO => match conseguir_lista(chars_iter) {
                Ok(contenido) => Some(PasoRegex {
                    repeticiones: Repeticion::Exacta(1, contenido.1),
                    caracter_interno: Caracter::Lista(contenido.0),
                }),
                Err(error) => return Err(error),
            },

            INTERROGACION => {
                match steps.last_mut() {
                    Some(last) => {
                        if last.caracter_interno == Caracter::Comodin
                            && last.repeticiones == Repeticion::Alguna(false){
                            return Err(Error::CaracterNoProcesable);
                        } else {
                            last.repeticiones = Repeticion::Rango {
                                min: Some(0),
                                max: Some(1),
                            };
                        }
                    }
                    None => {}
                }
                None
            }

            ASTERISCO => {
                match steps.last_mut() {
                    Some(last) => {
                        if last.caracter_interno == Caracter::Comodin
                            && last.repeticiones == Repeticion::Alguna(false)
                        {
                            return Err(Error::CaracterNoProcesable);
                        } else {
                            match conseguir_lista(chars_iter) {
                                Ok((_, negado)) => last.repeticiones = Repeticion::Alguna(negado),
                                Err(error) => return Err(error),
                            };
                        }
                    }
                    None => {}
                }
                None
            }

            MAS => {
                match steps.last_mut() {
                    Some(last) => {
                        println!("el paso anterior es: {:?}", last);

                        if last.caracter_interno == Caracter::Comodin
                            && last.repeticiones == Repeticion::Alguna(false)
                        {
                            return Err(Error::CaracterNoProcesable);
                        } else {
                            last.repeticiones = Repeticion::Rango {
                                min: Some(1),
                                max: None,
                            };
                        }
                    }
                    None => {}
                }
                None
            }

            BARRA => match chars_iter.next() {
                Some(literal) => Some(PasoRegex {
                    repeticiones: Repeticion::Exacta(1, false),
                    caracter_interno: Caracter::Literal(literal),
                }),
                None => return Err(Error::CaracterNoProcesable),
            },

            DOLAR => Some(PasoRegex {
                repeticiones: Repeticion::Exacta(1, false),
                caracter_interno: Caracter::Dollar,
            }),

            CARET => None,

            FUNCION_OR => None,

            _ => return Err(Error::CaracterNoProcesable),
        };

        if let Some(p) = step {
            steps.push(p);
        }
    }

    Ok(steps.to_vec())
}

fn definir_uso_de_caret(expression: &str, steps: &mut Vec<PasoRegex>) {
    if !expression.starts_with(CARET) {
        let paso = Some(PasoRegex {
            repeticiones: Repeticion::Alguna(false),
            caracter_interno: Caracter::Comodin,
        });
        if let Some(p) = paso {
            steps.push(p);
        }
    }
}

impl Regex {
    pub fn es_valida_general(expression: &str, linea: &str) -> Result<bool, Error> {
        let expresiones_a_evaluar: Vec<&str> = expression.split('|').collect();
        let mut coincidencia = false;

        for exp in expresiones_a_evaluar {
            let regex = match Regex::new(exp) {
                Ok(regex) => regex,
                Err(err) => return Err(err),
            };
            if regex.es_valida(linea)? {
                coincidencia = true;
                break;
            }
        }
        Ok(coincidencia)
    }

    pub fn new(expression: &str) -> Result<Self, Error> {
        let mut steps: Vec<PasoRegex> = Vec::new();
        let mut chars_iter = expression.chars();

        definir_uso_de_caret(expression, &mut steps);

        let steps: Vec<PasoRegex> = agregar_pasos(&mut steps, &mut chars_iter)?;

        println!("{:?}", steps);

        Ok(Regex { pasos: steps })
    }

    pub fn es_valida(self, linea: &str) -> Result<bool, Error> {
        if !linea.is_ascii() {
            return Err(Error::FormatoDeLineaNoASCII);
        }
        let mut queue: VecDeque<PasoRegex> = VecDeque::from(self.pasos);
        let mut stack: Vec<PasoEvaluado> = Vec::new();
        let mut index = 0;

        'pasos: while let Some(mut paso) = queue.pop_front() {
            println!("{:?}", paso.caracter_interno);
            match paso.repeticiones {
                Repeticion::Exacta(n, negacion) => {
                    let mut match_size = 0;
                    for _ in 0..n {
                        let avance = paso.caracter_interno.coincide(&linea[index..]);
                        if avance == 0 {
                            match backtrack(paso, &mut stack, &mut queue) {
                                Some(size) => {
                                    index -= size;
                                    continue 'pasos;
                                }
                                None => {
                                    if negacion {
                                        return Ok(true);
                                    } else {
                                        return Ok(false);
                                    }
                                }
                            }
                        } else {
                            match_size += avance;
                            index += avance;
                        }
                    }

                    stack.push(PasoEvaluado {
                        paso: paso,
                        tam_matcheo: match_size,
                        backtrackeable: false,
                    });
                    if negacion {
                        return Ok(false);
                    }
                }
                Repeticion::Alguna(negacion) => {
                    let mut sigo_avanzando = true;
                    while sigo_avanzando {
                        let avance = paso.caracter_interno.coincide(&linea[index..]);

                        if avance != 0 {
                            index += avance;
                            stack.push(PasoEvaluado {
                                paso: paso.clone(),
                                tam_matcheo: avance,
                                backtrackeable: true,
                            })
                        } else {
                            sigo_avanzando = false;
                        }
                        if negacion {
                            return Ok(false);
                        }
                    }
                }
                Repeticion::Rango { min, max } => {
                    let min = match min {
                        Some(min) => min,
                        None => 0,
                    };

                    let max = match max {
                        Some(max) => max,
                        None => linea.len() - index,
                    };
                    let mut aux: Vec<PasoEvaluado> = Vec::new();

                    let mut sigo_avanzando = true;
                    while sigo_avanzando {

                       /* if matches!(paso.caracter_interno, Caracter::Lista(_)) {
                            paso.caracter_interno =
                                Caracter::Literal(linea.as_bytes()[index] as char);
                        }   */

                        let avance = paso.caracter_interno.coincide(&linea[index..]);

                        if avance != 0 {
                            index += avance;
                            aux.push(PasoEvaluado {
                                paso: paso.clone(),
                                tam_matcheo: avance,
                                backtrackeable: true,
                            });
                            stack.push(PasoEvaluado {
                                paso: paso.clone(),
                                tam_matcheo: avance,
                                backtrackeable: true,
                            })
                        } else {
                            sigo_avanzando = false;
                        }
                    }

                    if aux.len() < min || aux.len() > max {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }
}

fn backtrack(actual: PasoRegex, evaluados: &mut Vec<PasoEvaluado>, siguiente: &mut VecDeque<PasoRegex>,) -> Option<usize> {
    
    let mut back_size = 0;
    
    siguiente.push_front(actual);
    
    while let Some(paso_ev) = evaluados.pop() {
        back_size += paso_ev.tam_matcheo;
        if paso_ev.backtrackeable {
            return Some(back_size);
        } else {
            siguiente.push_front(paso_ev.paso);
        }
    }
    None
}


///test unitarios
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_literales() {
        let regex = Regex::new("abcd");
        assert_eq!(regex.unwrap().es_valida("abcdefg").unwrap(), true);
    }

    #[test]
    fn test02_regex_con_literales() {
        let regex = Regex::new("^abcd");
        assert_eq!(regex.unwrap().es_valida("abcdefg").unwrap(), true);
    }

    #[test]
    fn test03_regex_con_literales() {
        let regex = Regex::new("^abcd");
        assert_eq!(regex.unwrap().es_valida("ab abcdefg").unwrap(), false);
    }

    #[test]
    fn test04_literales() {
        let regex = Regex::new("abcd");
        assert_eq!(regex.unwrap().es_valida("efgabcd").unwrap(), true);
    }

    #[test]
    fn test05_literales() {
        let regex = Regex::new("abcd");
        assert_eq!(regex.unwrap().es_valida("abcefg").unwrap(), false);
    }

    #[test]
    fn test06_punto() {
        let regex = Regex::new("ab.cd");
        assert_eq!(regex.unwrap().es_valida("ab0cd").unwrap(), true);
    }

    #[test]
    fn test07_punto() {
        let regex = Regex::new("ab.cd");
        assert_eq!(regex.unwrap().es_valida("abcd").unwrap(), false);
    }

    #[test]
    fn test08_regex_con_asterisk() {
        let regex = Regex::new("ab*c");
        assert_eq!(regex.unwrap().es_valida("abbbbbbc").unwrap(), true);
    }


    #[test]
    fn test08_punto_asterisco() {
        let regex = Regex::new("ab.*cd");
        assert_eq!(regex.unwrap().es_valida("abcd").unwrap(), true);
    }

    #[test]
    fn test09_punto_asterisco() {
        let regex = Regex::new("ab.*cd");
        assert_eq!(regex.unwrap().es_valida("abaaaaaacd").unwrap(), true);
    }

    #[test]
    fn test10_corchete() {
        let regex = Regex::new("a[bc]d");
        assert_eq!(regex.unwrap().es_valida("abd").unwrap(), true);
    }

    #[test]
    fn test11_corchete() {
        let regex = Regex::new("a[bc]d");
        assert_eq!(regex.unwrap().es_valida("acd").unwrap(), true);
    }

    #[test]
    fn test12_corchete() {
        let regex = Regex::new("a[bc]d");
        assert_eq!(regex.unwrap().es_valida("afd").unwrap(), false);
    }

    #[test]
    fn test04_regex_con_asterisk02() {
        let regex = Regex::new("^ab*c");
        assert_eq!(regex.unwrap().es_valida("fdf ac").unwrap(), false);
    }

    #[test]
    fn test05_regex_con_metacaracter_con_backlash() {
        let regex = Regex::new("a\\*");
        assert_eq!(regex.unwrap().es_valida("a*cds").unwrap(), true);
    }

    #[test]
    fn test06_regex_con_plus() {
        let regex = Regex::new("hola+");
        assert_eq!(regex.unwrap().es_valida("holaa").unwrap(), true);
    }

    #[test]
    fn test07_regex_con_plus() {
        let regex = Regex::new("hola+");
        assert_eq!(regex.unwrap().es_valida("hol").unwrap(), false);
    }

    #[test]
    fn test07_regex_con_question() {
        let regex = Regex::new("hola?f");
        assert_eq!(regex.unwrap().es_valida("holaf").unwrap(), true);
    }

    #[test]
    fn test08_regex_con_question2() {
        let regex = Regex::new("hola?s");
        assert_eq!(regex.unwrap().es_valida("hols").unwrap(), true);
    }

    #[test]
    fn test09_regex_con_question3() {
        let regex = Regex::new("hola?");
        assert_eq!(regex.unwrap().es_valida("holaaaaa").unwrap(), false);
    }

    #[test]
    fn test10_regex_con_bracket_exacto() {
        let regex = Regex::new("a{2}");
        assert_eq!(regex.unwrap().es_valida("a").unwrap(), false);
    }

    #[test]
    fn test11_regex_con_bracket_exacto_() {
        let regex = Regex::new("ba{2}");
        assert_eq!(regex.unwrap().es_valida("baa").unwrap(), true);
    }

    #[test]
    fn test12_regex_con_bracket_exacto_() {
        let regex = Regex::new("ba{2}c");
        assert_eq!(regex.unwrap().es_valida("bac").unwrap(), false);
    }

    #[test]
    fn test13_regex_con_bracket_con_minimo_() {
        let regex = Regex::new("ba{2,}c");
        assert_eq!(regex.unwrap().es_valida("baaaac").unwrap(), true);
    }

    #[test]
    fn test14_regex_con_bracket_con_minimo_() {
        let regex = Regex::new("ba{2,}c");
        assert_eq!(regex.unwrap().es_valida("bac").unwrap(), false);
    }

    #[test]
    fn test15_regex_con_bracket_con_rango_() {
        let regex = Regex::new("ba{5,8}c");
        assert_eq!(regex.unwrap().es_valida("baaaaac").unwrap(), true);
    }

    #[test]
    fn test16_regex_con_bracket_con_rango_() {
        let regex = Regex::new("ba{5,8}c");
        assert_eq!(regex.unwrap().es_valida("baaaac").unwrap(), false);
    }

    #[test]
    fn test17_regex_con_bracket_con_rango_() {
        let regex = Regex::new("ba{5,8}c");
        assert_eq!(regex.unwrap().es_valida("baaaaaaaaac").unwrap(), false);
    }

    #[test]
    fn test18_regex_con_bracket_con_maximo1_() {
        let regex = Regex::new("ba{,8}c");
        assert_eq!(regex.unwrap().es_valida("baaaaaac").unwrap(), true);
    }

    #[test]
    fn test19_regex_con_bracket_con_maximo2_() {
        let regex = Regex::new("ba{,8}c");
        assert_eq!(
            regex.unwrap().es_valida("baaaaaaaaaaaaaaaac").unwrap(),
            false
        );
    }
    #[test]
    fn test20_regex_combinado() {
        let regex = Regex::new("ba{5,8}.c");
        assert_eq!(regex.unwrap().es_valida("baaaaaaafc").unwrap(), true);
    }

    #[test]
    fn test21_regex_bracket_literal_01() {
        let regex = Regex::new("ho[lmn]a");
        assert_eq!(regex.unwrap().es_valida("hola").unwrap(), true);
    }

    #[test]
    fn test22_regex_bracket_literal_02() {
        let regex = Regex::new("ho[lmn]a");
        assert_eq!(regex.unwrap().es_valida("hoka").unwrap(), false);
    }

    #[test]
    fn test23_regex_bracket_rango_01() {
        let regex = Regex::new("ho[i-m]a");
        assert_eq!(regex.unwrap().es_valida("hola").unwrap(), true);
    }

    #[test]
    fn test24_regex_bracket_rango_02() {
        let regex = Regex::new("ho[i-m]a");
        assert_eq!(regex.unwrap().es_valida("hosa").unwrap(), false);
    }

    #[test]
    fn test25_regex_combinado() {
        let regex = Regex::new("ho[k-o]a.p{2,4}");
        assert_eq!(regex.unwrap().es_valida("hola3ppp").unwrap(), true);
    }

    #[test]
    fn test26_regex_bracket_rango_03() {
        let regex = Regex::new("ho[a-dA-Cx-z]");
        assert_eq!(regex.unwrap().es_valida("hoAa").unwrap(), true);
    }

    #[test]
    fn test27_regex_bracket_rango_04() {
        let regex = Regex::new("ho[a-dA-Cx-z]");
        assert_eq!(regex.unwrap().es_valida("hoXa").unwrap(), false);
    }

    #[test]
    fn test28_regex_bracket_rango_05_negado() {
        let regex = Regex::new("ho[^a-dA-Cx-z]");
        assert_eq!(regex.unwrap().es_valida("hoXa").unwrap(), true);
    }

    #[test]
    fn test29_regex_bracket_rango_06_negado() {
        let regex = Regex::new("ho[^a-dA-Cx-z]");
        assert_eq!(regex.unwrap().es_valida("hoxa").unwrap(), false);
    }

    #[test]
    fn test30_regex_combinado_bracket_question01() {
        let regex = Regex::new("ho[a-dA-Cx-z]?a");
        assert_eq!(regex.unwrap().es_valida("hoddda").unwrap(), false);
    }

    #[test]
    fn test31_regex_combinado_bracket_question02() {
        let regex = Regex::new("ho[a-dA-Cx-z]?a");
        assert_eq!(regex.unwrap().es_valida("hoa").unwrap(), true);
    }

    #[test]
    fn test32_regex_combinado_bracket_question03() {
        let regex = Regex::new("ho[d-g]?a");
        assert_eq!(regex.unwrap().es_valida("hoea").unwrap(), true);
    }

    #[test]
    fn test33_regex_combinado_bracket_plus01() {
        let regex = Regex::new("ho[a-dA-Cx-z]+a");
        assert_eq!(regex.unwrap().es_valida("hoE").unwrap(), false);
    }

    #[test]
    fn test34_regex_combinado_bracket_plus02() {
        let regex = Regex::new("ho[a-dA-Cx-z]+a");
        assert_eq!(regex.unwrap().es_valida("hoAAAAAa").unwrap(), true);
    }

    #[test]
    fn test35_regex_combinado_bracket_plus03() {
        let regex = Regex::new("ho[a-dA-Cx-z]+a");
        assert_eq!(
            regex.unwrap().es_valida("hoxxxAAAAa").unwrap(),
            true
        );
    }

    #[test]
    fn test36_regex_combinado_bracket_rango01() {
        let regex = Regex::new("ho[a-dA-Cx-z]{2,4}a");
        assert_eq!(regex.unwrap().es_valida("hoaE").unwrap(), false);
    }

    #[test]
    fn test37_regex_combinado_bracket_rango02() {
        let regex = Regex::new("ho[a-dA-Cx-z]{2,4}a");
        assert_eq!(regex.unwrap().es_valida("hoaaaE").unwrap(), true);
    }

    #[test]
    fn test38_regex_combinado_bracket_rango03() {
        let regex = Regex::new("ho[a-dA-Cx-z]{2,4}a");
        assert_eq!(regex.unwrap().es_valida("hoaaaaaaE").unwrap(), false);
    }

    #[test]
    fn test39_regex_combinado_bracket_asterisk01() {
        let regex = Regex::new("ho[a-dA-Cx-z]*a");
        assert_eq!(regex.unwrap().es_valida("hoa").unwrap(), true);
    }

    #[test]
    fn test40_regex_combinado_bracket_asterisk02() {
        let regex = Regex::new("ho[a-dA-Cx-z]*a");
        assert_eq!(regex.unwrap().es_valida("hoAAAa").unwrap(), true);
    }

    #[test]
    fn test41_regex_combinado_bracket_negado_asterisk01() {
        let regex = Regex::new("ho[^a-dA-Cx-z]*a");
        assert_eq!(regex.unwrap().es_valida("hoKa").unwrap(), true);
    }

    #[test]
    fn test42_regex_combinado_bracket_negado_asterisk02() {
        let regex = Regex::new("ho[^a-dA-Cx-z]*a");
        assert_eq!(regex.unwrap().es_valida("hoa").unwrap(), true);
    }

    #[test]
    fn test43_regex_clases01() {
        let regex = Regex::new("ho[[:alpha:]]a");
        assert_eq!(regex.unwrap().es_valida("hola").unwrap(), true);
    }

    #[test]
    fn test44_regex_clases02() {
        let regex = Regex::new("ho[^[:alpha:]]a");
        assert_eq!(regex.unwrap().es_valida("ho8a").unwrap(), true);
    }

    #[test]
    fn test45_regex_clases03() {
        let regex = Regex::new("ho[[:alnum:]]a");
        assert_eq!(regex.unwrap().es_valida("hoKa").unwrap(), true);
    }

    #[test]
    fn test46_regex_clases04() {
        let regex = Regex::new("ho[[:alnum:]]a");
        assert_eq!(regex.unwrap().es_valida("ho4a").unwrap(), true);
    }

    #[test]
    fn test47_regex_clases05() {
        let regex = Regex::new("ho[^[:alnum:]]a");
        assert_eq!(regex.unwrap().es_valida("ho&a").unwrap(), true);
    }

    #[test]
    fn test48_regex_clases06() {
        let regex = Regex::new("ho[[:digit:]]a");
        assert_eq!(regex.unwrap().es_valida("ho2a").unwrap(), true);
    }

    #[test]
    fn test49_regex_clases07() {
        let regex = Regex::new("ho[[:digit:]]+");
        assert_eq!(regex.unwrap().es_valida("ho9999999").unwrap(), true);
    }

    #[test]
    fn test50_regex_clases08() {
        let regex = Regex::new("ho[^[:digit:]]a");
        assert_eq!(regex.unwrap().es_valida("hoea").unwrap(), true);
    }

    #[test]
    fn test51_regex_clases09() {
        let regex = Regex::new("ho[[:lower:]]a");
        assert_eq!(regex.unwrap().es_valida("hoRa").unwrap(), false);
    }

    #[test]
    fn test52_regex_clases10() {
        let regex = Regex::new("ho[[:lower:]]a");
        assert_eq!(regex.unwrap().es_valida("hora").unwrap(), true);
    }

    #[test]
    fn test53_regex_clases11() {
        let regex = Regex::new("ho[[:upper:]]a");
        assert_eq!(regex.unwrap().es_valida("hoRa").unwrap(), true);
    }

    #[test]
    fn test54_regex_clases11() {
        let regex = Regex::new("ho[[:upper:]]a");
        assert_eq!(regex.unwrap().es_valida("hola").unwrap(), false);
    }

    #[test]
    fn test55_regex_clases12() {
        let regex = Regex::new("ho[[:space:]]a");
        assert_eq!(regex.unwrap().es_valida("ho a").unwrap(), true);
    }

    #[test]
    fn test56_regex_clases13() {
        let regex = Regex::new("ho[[:space:]]a");
        assert_eq!(regex.unwrap().es_valida("hola").unwrap(), false);
    }

    #[test]
    fn test57_regex_clases14() {
        let regex = Regex::new("ho[[:punct:]]a");
        assert_eq!(regex.unwrap().es_valida("ho;a").unwrap(), true);
    }

    #[test]
    fn test55_regex_clases15() {
        let regex = Regex::new("ho[[:punct:]]a");
        assert_eq!(regex.unwrap().es_valida("ho9a").unwrap(), false);
    }

    #[test]
    fn test56_regex_combinado_clases01() {
        let regex = Regex::new("^ho[[:punct:]]{2}a+");
        assert_eq!(regex.unwrap().es_valida("ho..aaaaaa").unwrap(), true);
    }

    #[test]
    fn test57_regex_combinado_clases01() {
        let regex = Regex::new("ho[[:punct:]]{2}a+");
        assert_eq!(regex.unwrap().es_valida("aaaaa ho..aaaaaa").unwrap(), true);
    }

    #[test]
    fn test58_regex_combinado_clases01() {
        let regex = Regex::new("[a-kA-G]ho[[:punct:]]*a\\.?");
        assert_eq!(regex.unwrap().es_valida("Dho;.a.").unwrap(), true);
    }

    #[test]
    fn test59_regex_dollar() {
        let regex = Regex::new("hola$");
        assert_eq!(regex.unwrap().es_valida("hola").unwrap(), true);
    }

    #[test]
    fn test60_regex_dollar() {
        let regex = Regex::new("hola$");
        assert_eq!(regex.unwrap().es_valida("ajaja hola").unwrap(), true);
    }

    #[test]
    fn test61_regex_dollar() {
        let regex = Regex::new("hola$");
        assert_eq!(regex.unwrap().es_valida("jaja hol").unwrap(), false);
    }

    #[test]
    fn test61_regex_dollar_caret() {
        let regex = Regex::new("^hola$");
        assert_eq!(regex.unwrap().es_valida("jajaja hola").unwrap(), false);
    }

    #[test]
    fn test62_regex_dollar_caret() {
        let regex = Regex::new("^hola$");
        assert_eq!(regex.unwrap().es_valida("hola ajajaja").unwrap(), false);
    }

    #[test]
    fn test63_regex_dollar_caret() {
        let regex = Regex::new("^hola$");
        assert_eq!(regex.unwrap().es_valida("hola").unwrap(), true);
    }

    #[test]
    fn test64_regex_dollar_caret() {
        let regex = Regex::new("^hola$");
        assert_eq!(regex.unwrap().es_valida("hola").unwrap(), true);
    }

    #[test]
    fn test65_regex_con_or() {
        assert_eq!(
            Regex::es_valida_general("[abc]d[[:alpha:]]|k", "hola").unwrap(),
            false
        );
    }

    #[test]
    fn test65_regex_con_or2() {
        assert_eq!(
            Regex::es_valida_general("[abc]d[[:alpha:]]|k", "adAk").unwrap(),
            true
        );
    }

    #[test]
    fn test01() {
        assert_eq!(Regex::es_valida_general("abc|de+f", "abc").unwrap(),true);}

    #[test]
    fn test02() {
        assert_eq!(Regex::es_valida_general("abc|de+f", "deeeeeeeeeeeeeef").unwrap(),true);}

    #[test]
    fn test03() {
        assert_eq!(Regex::es_valida_general("abc|de+f", "abcdeeeeeeeeeeeeeef").unwrap(),true);}

    #[test]
    fn test04() {
        assert_eq!(Regex::es_valida_general("abc|de+f", "abdeeeeeeeeeeeeee").unwrap(),false);}


    #[test]
    fn test06() {
        let regex = Regex::new("la [aeiou] es una vocal");
        assert_eq!(regex.unwrap().es_valida("la o es una vocal").unwrap(), true);
    }   

    #[test]
    fn test06wetk() {
        let regex = Regex::new("la [aeiou] es una vocal");
        assert_eq!(regex.unwrap().es_valida("la f es una vocal").unwrap(), false);
    } 

    #[test]
    fn test07() {
        let regex = Regex::new("la [^aeiou] es una consonante");
        assert_eq!(regex.unwrap().es_valida("la a es una consonante").unwrap(), false);
    } 

    #[test]
    fn test08() {
        let regex = Regex::new("la [^aeiou] es una consonante");
        assert_eq!(regex.unwrap().es_valida("la r es una consonante").unwrap(), true);
    } 

    #[test]
    fn test09() {
        let regex = Regex::new("hola [[:alpha:]]+");
        assert_eq!(regex.unwrap().es_valida("hola cccccasecc").unwrap(), true);
    } 

    #[test]
    fn test059() {
        let regex = Regex::new("hola [abcd]+");
        assert_eq!(regex.unwrap().es_valida("hola ffffffff").unwrap(), false); 
    } 

    #[test]
    fn test05769() {
        let regex = Regex::new("[[:upper:]]ascal[[:upper:]]ase");
        assert_eq!(regex.unwrap().es_valida("PascalCase").unwrap(), true); 
    } 

    #[test]
    fn test0re5769() {
        let regex = Regex::new("[[:upper:]]ascal[[:upper:]]ase");
        assert_eq!(regex.unwrap().es_valida("Pascalcase").unwrap(), false); 
    } 
}
/*
abcd
abxcd
abxxcd
abcde
ab.cd
abxxxxcd
ab123cd
ab.*cd
ab{2,4}cd
abbbbbcd
abcdeeeef
a[bc]d
ad
abcdabcd
abbc
abbcd
abbbcd
ab{3}cd
ab{4}cd
ab{5}cd
ab{2,4}cd
ab{3,}cd
abcdeee
abcdeeeee
abcdeeeeee
abcdeeeeeee
abc|de+f
abcf
abcfde
abcdeeeeef
abcdeeeeeef
abcde
la a es una vocal
la e es una vocal
la i es una vocal
la o es una vocal
la u es una vocal
la b no es una vocal
la c no es una vocal
la d no es una vocal
la f no es una vocal
la g no es una vocal
la [aeiou] es una vocal
la [^aeiou] no es una vocal
la hola es una vocal
hola 123 mundo
hola mundo
hola hola mundo
hola la mundo
hola a mundo
hola    mundo
hola  mundo
hola       mundo
hola m u n d o
hola[[:space:]]mundo
HascalYase
ascalaYase
ascalAase
aascalAase
abascalYase
abascalYase
abascalase
es el fin$
es el fin
es el fin !
es el fin123 */

/*ab.cd lito
ab.*cd lito
a[bc]d lito
ab{2,4}cd lito
abc|de+f lito
la [aeiou] es una vocal lito
la [^aeiou] no es una vocal lito
hola [[:alpha:]]+ lito
[[:digit:]] es un numero
el caracter [[:alnum:]] no es un simbolo
hola[[:space:]]mundo
[[:upper:]]ascal[[:upper:]]ase
es el fin$ */
