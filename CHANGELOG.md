# Changelog

Todas as alterações notáveis deste projeto serão documentadas neste arquivo.

## [0.0.31-alpha]
- Correção no autocomplete inline do comando `cd` para preservar letras maiúsculas e minúsculas exatas do sistema de arquivos
- Suporte a navegação por múltiplos níveis de pastas ao digitar `/` ou `\` (ex: `cd pasta/subpasta/`)
- Paginação dinâmica no autocomplete de `cd`: exibe 10 itens por vez e carrega mais 10 ao navegar com `Tab` até o final
- Correção no atalho `Ctrl + C` quando configurado para cópia e interrupção (`stop`): sem texto selecionado, envia sinal de interrupção ao processo

## [0.0.29-alpha]
- Autocomplete inline dinâmico para o comando `cd` com listagem e filtragem de pastas em tempo real
- Limite inteligente de 10 pastas com navegação suave por `Tab` e setas
- Suporte a nomes de diretórios com espaços (escape automático de aspas)
- Compatibilidade com prompts customizados (Oh My Posh, Starship, Powerlevel10k)
- Bloqueio automático de sugestões de aliases ao digitar `cd`
- Fechamento automático dos popups inline ao abrir o modal de diretórios favoritados (`Shift + Espaço`)

## [0.0.26-alpha]
- Correção no autocomplete de comandos VPS ao passar caminhos locais no `rsync`
- Suporte a changelog automático na tela de atualização
- Melhorias de estabilidade e performance no terminal

## [0.0.25-alpha]
- Otimização do monitor de CPU nativo para Windows e Linux
- Melhorias visuais nos modais e transparência da janela
