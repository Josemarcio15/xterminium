//! Leitura de OSC 7 / OSC 9;9 no stream do PTY.
//!
//! O terminal não tem como *perguntar* ao shell qual é o diretório atual. O
//! PowerShell, por exemplo, nunca chama `chdir` em `Set-Location`: o cwd do
//! processo (`platform::process_cwd`) fica congelado no diretório em que o shell
//! foi lançado. A fonte correta é o próprio shell informando pelo prompt via
//! escape sequence — a mesma técnica de Windows Terminal, VS Code e WezTerm.
//!
//! - **OSC 9;9** — ConEmu/Windows Terminal: `ESC ] 9 ; 9 ; <caminho> BEL`
//! - **OSC 7** — padrão (bash/zsh/fish): `ESC ] 7 ; file://<host>/<caminho> BEL`
//!
//! Sequências terminam em `BEL` (0x07) ou `ST` (`ESC \`).

const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;

/// Tamanho máximo do corpo de um OSC. Protege contra lixo sem terminador.
const MAX_BODY: usize = 4096;

#[derive(Default)]
enum State {
    #[default]
    Idle,
    /// Viu `ESC`, pode virar `ESC ]` (OSC) ou qualquer outra sequência.
    Esc,
    /// Dentro do corpo do OSC, acumulando em `body`.
    Body,
    /// Viu `ESC` dentro do corpo: pode ser o `ST` (`ESC \`).
    BodyEsc,
}

/// Scanner incremental: o PTY entrega bytes em chunks, então uma sequência pode
/// chegar partida entre duas leituras.
#[derive(Default)]
pub struct OscScanner {
    state: State,
    body: Vec<u8>,
}

impl OscScanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Varre um pedaço do stream e devolve o último diretório reportado nele.
    pub fn push(&mut self, data: &[u8]) -> Option<String> {
        let mut found = None;

        for &byte in data {
            match self.state {
                State::Idle => {
                    if byte == ESC {
                        self.state = State::Esc;
                    }
                }
                State::Esc => {
                    if byte == b']' {
                        self.body.clear();
                        self.state = State::Body;
                    } else if byte != ESC {
                        self.state = State::Idle;
                    }
                }
                State::Body => {
                    if byte == BEL {
                        found = decode(&self.body).or(found);
                        self.state = State::Idle;
                    } else if byte == ESC {
                        self.state = State::BodyEsc;
                    } else if self.body.len() < MAX_BODY {
                        self.body.push(byte);
                    } else {
                        // Corpo absurdo (sem terminador): descarta e volta ao início.
                        self.body.clear();
                        self.state = State::Idle;
                    }
                }
                State::BodyEsc => {
                    if byte == b'\\' {
                        found = decode(&self.body).or(found);
                    }
                    self.state = State::Idle;
                }
            }
        }

        found
    }
}

/// Extrai o caminho de um corpo de OSC, se for `9;9` ou `7`.
fn decode(body: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(body);

    if let Some(path) = text.strip_prefix("9;9;") {
        return non_empty(path);
    }

    if let Some(rest) = text.strip_prefix("7;") {
        let after_scheme = rest.strip_prefix("file://")?;
        // `file://host/path` -> descarta o host
        let path = &after_scheme[after_scheme.find('/')?..];
        // O `percent_decode` é genérico; o ajuste de formato que depende do SO
        // (`/C:/...` -> `C:/...` no Windows) vive em `crate::platform`.
        let path = crate::platform::normalize_reported_path(percent_decode(path));
        return non_empty(&path);
    }

    None
}

fn non_empty(path: &str) -> Option<String> {
    let path = path.trim();
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

/// Decodifica `%XX` (OSC 7 codifica espaços e acentos).
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|h| u8::from_str_radix(h, 16).ok());
            if let Some(byte) = hex {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Alimenta o scanner com vários chunks e devolve o último cwd visto.
    fn scan(chunks: &[&[u8]]) -> Option<String> {
        let mut scanner = OscScanner::new();
        let mut last = None;
        for chunk in chunks {
            last = scanner.push(chunk).or(last);
        }
        last
    }

    #[test]
    fn le_osc_9_9_do_powershell() {
        assert_eq!(
            scan(&[b"\x1b]9;9;C:\\Users\\Marcio\\git\\xterminium\x07"]),
            Some("C:\\Users\\Marcio\\git\\xterminium".to_string())
        );
    }

    #[test]
    fn remonta_sequencia_partida_entre_chunks() {
        assert_eq!(
            scan(&[b"\x1b]9;9;C:\\Use", b"rs\\Marcio\x07"]),
            Some("C:\\Users\\Marcio".to_string())
        );
    }

    #[test]
    fn aceita_terminador_st() {
        assert_eq!(
            scan(&[b"\x1b]9;9;/home/marcio\x1b\\"]),
            Some("/home/marcio".to_string())
        );
    }

    #[test]
    fn le_osc_7_com_percent_encoding() {
        assert_eq!(
            scan(&[b"\x1b]7;file://pc/home/marcio/meus%20docs\x07"]),
            Some("/home/marcio/meus docs".to_string())
        );
    }

    #[test]
    fn le_osc_7_de_drive_windows() {
        // `/C:/...` só é reescrito para `C:/...` no Windows; no POSIX a barra
        // inicial faz parte de um caminho válido e é preservada.
        #[cfg(windows)]
        assert_eq!(
            scan(&[b"\x1b]7;file://pc/C:/Users/Marcio\x07"]),
            Some("C:/Users/Marcio".to_string())
        );
        #[cfg(not(windows))]
        assert_eq!(
            scan(&[b"\x1b]7;file://pc/C:/Users/Marcio\x07"]),
            Some("/C:/Users/Marcio".to_string())
        );
    }

    #[test]
    fn ignora_sequencias_de_cor() {
        assert_eq!(scan(&[b"\x1b[38;2;84;110;122mtexto\x1b[0m"]), None);
    }

    #[test]
    fn ignora_osc_desconhecido() {
        assert_eq!(scan(&[b"\x1b]0;titulo da janela\x07"]), None);
    }

    #[test]
    fn devolve_o_ultimo_do_mesmo_chunk() {
        assert_eq!(
            scan(&[b"\x1b]9;9;/primeiro\x07\x1b]9;9;/segundo\x07"]),
            Some("/segundo".to_string())
        );
    }

    #[test]
    fn acha_oscilacao_depois_de_texto_e_cores() {
        assert_eq!(
            scan(&[b"\x1b[0m\xe2\x9d\xaf\x1b]9;9;/tmp\x07 resto"]),
            Some("/tmp".to_string())
        );
    }

    #[test]
    fn corpo_sem_terminador_nao_trava_nem_vaza() {
        let mut scanner = OscScanner::new();
        let lixo = vec![b'x'; MAX_BODY * 2];
        assert_eq!(scanner.push(b"\x1b]9;9;"), None);
        assert_eq!(scanner.push(&lixo), None);
        // Depois do descarte o scanner volta a funcionar normalmente.
        assert_eq!(scanner.push(b"\x1b]9;9;/ok\x07"), Some("/ok".to_string()));
    }
}
