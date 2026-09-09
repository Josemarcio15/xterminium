export interface CustomAlias {
  id: string;
  alias: string;        // Gatilho curto digitado (ex: 'aptupdate', 'dps', 'glog')
  command: string;      // Comando referente expandido (ex: 'apt update && apt upgrade -y')
  description?: string; // Descrição opcional (ex: 'Atualiza repositórios do sistema')
}

export const defaultCustomAliases: CustomAlias[] = [
  {
    id: 'alias-aptupdate',
    alias: 'aptupdate',
    command: 'apt update && apt upgrade -y',
    description: 'Atualizar repositórios e pacotes',
  },
  {
    id: 'alias-dps',
    alias: 'dps',
    command: 'docker ps --format "table {{.Names}}\\t{{.Status}}\\t{{.Ports}}"',
    description: 'Listar containers docker formatados',
  },
  {
    id: 'alias-glog',
    alias: 'glog',
    command: 'git log --oneline -n 10',
    description: 'Últimos 10 commits git resumidos',
  },
  {
    id: 'alias-myip',
    alias: 'myip',
    command: 'curl -s ifconfig.me',
    description: 'Descobrir IP público da máquina',
  },
];
