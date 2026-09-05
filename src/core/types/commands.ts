export interface CustomCommand {
  id: string;
  command: string;      // Ex: 'rsync', 'scp', 'ssh'
  prefixArgs?: string;  // Argumentos antes da VPS (ex: '-avz', '-P')
  template: string;     // Formato base da VPS (ex: '{user}@{ip}', '{ip}')
  suffixArgs?: string;  // Caminho ou argumentos após a VPS (ex: ':~/teste/aqui/', ':~/', '/var/www')
}

export const defaultCustomCommands: CustomCommand[] = [
  {
    id: 'cmd-ssh',
    command: 'ssh',
    prefixArgs: '',
    template: '{user}@{ip}',
    suffixArgs: '',
  },
  {
    id: 'cmd-scp',
    command: 'scp',
    prefixArgs: '',
    template: '{user}@{ip}',
    suffixArgs: ':~/',
  },
  {
    id: 'cmd-rsync',
    command: 'rsync',
    prefixArgs: '-avz',
    template: '{user}@{ip}',
    suffixArgs: ':~/',
  },
  {
    id: 'cmd-ping',
    command: 'ping',
    prefixArgs: '-c 4',
    template: '{ip}',
    suffixArgs: '',
  },
];
