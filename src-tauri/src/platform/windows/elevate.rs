//! Elevação de privilégio local no Windows.

/// Não há equivalente ao `sudo` no Windows: o controle de acesso é feito por
/// ACL, não por um comando que promove o processo. Elevar de verdade exigiria
/// relançar o app inteiro via UAC (`Start-Process -Verb RunAs`), o que não se
/// encaixa no fluxo de uma operação pontual do gerenciador de arquivos.
pub async fn exec_local_sudo(_password: &str, _command: &str) -> Result<(), String> {
    Err("Elevação de privilégio local (sudo) não é suportada no Windows.".to_string())
}
