#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import readline from 'readline';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const packageJsonPath = path.join(rootDir, 'package.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');

function readCurrentVersion() {
  const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
  return pkg.version;
}

function suggestNextVersion(current) {
  // Ex: 0.0.9-alpha -> 0.0.10-alpha
  const match = current.match(/^(\d+\.\d+\.)(\d+)(.*)$/);
  if (match) {
    const nextPatch = parseInt(match[2], 10) + 1;
    return `${match[1]}${nextPatch}${match[3]}`;
  }
  return current;
}

function updateVersionsInFiles(newVersion) {
  // 1. package.json (Fonte da verdade para JS/TS e Tauri)
  const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
  pkg.version = newVersion;
  fs.writeFileSync(packageJsonPath, JSON.stringify(pkg, null, 2) + '\n');

  // 2. Cargo.toml (Exigido pelo compilador Rust)
  let cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
  cargoToml = cargoToml.replace(/^version\s*=\s*"[^"]+"/m, `version = "${newVersion}"`);
  fs.writeFileSync(cargoTomlPath, cargoToml);
}

// Pergunta interativa com valor autopreenchido no buffer editável (basta teclar ENTER ou Backspace)
function askPreFilled(query, defaultValue) {
  return new Promise((resolve) => {
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });

    rl.question(query, (answer) => {
      rl.close();
      resolve(answer.trim() || defaultValue);
    });

    if (defaultValue) {
      rl.write(defaultValue);
    }
  });
}

async function main() {
  console.log('\n[XTERMINIUM] Release & Bump CLI\n');

  const currentVersion = readCurrentVersion();
  const nextVersionSuggestion = suggestNextVersion(currentVersion);

  try {
    const noCommit = process.argv.includes('--no-commit');

    // 1. Versão do App
    console.log(`Versao atual: \x1b[33m${currentVersion}\x1b[0m`);
    const targetVersion = await askPreFilled('Nova versao: ', nextVersionSuggestion);

    let targetTag = '';
    let commitMsg = '';

    if (!noCommit) {
      // 2. Nome da Tag Git
      const suggestedTag = targetVersion.startsWith('v') ? targetVersion : `v${targetVersion}`;
      targetTag = await askPreFilled('Tag Git: ', suggestedTag);

      // 3. Mensagem do Commit
      const suggestedCommitMsg = `release: ${targetTag}`;
      commitMsg = await askPreFilled('Mensagem do commit: ', suggestedCommitMsg);
    }

    console.log('\n------------------------------------------------------------');
    console.log(`Versao : \x1b[33m${targetVersion}\x1b[0m`);
    if (!noCommit) {
      console.log(`Tag    : \x1b[32m${targetTag}\x1b[0m`);
      console.log(`Commit : \x1b[36m${commitMsg}\x1b[0m`);
    } else {
      console.log(`Git    : \x1b[33m--no-commit (sem commit/tag/push)\x1b[0m`);
    }
    console.log('------------------------------------------------------------\n');

    const confirm = await askPreFilled('Confirma as alteracoes e gerar o bundle? (s/N): ', 's');
    if (confirm.toLowerCase() !== 's') {
      console.log('Operacao cancelada.');
      process.exit(0);
    }

    console.log('\n[1/5] Atualizando package.json e Cargo.toml...');
    updateVersionsInFiles(targetVersion);
    console.log('   OK: package.json');
    console.log('   OK: Cargo.toml');

    console.log('\n[2/5] Sincronizando Cargo.lock...');
    execSync('cargo check', { cwd: path.join(rootDir, 'src-tauri'), stdio: 'inherit' });

    console.log('\n[3/5] Gerando bundle .deb via Tauri...');
    execSync('npx tauri build --bundles deb', { cwd: rootDir, stdio: 'inherit' });

    if (noCommit) {
      console.log('\n\x1b[33m[Flag --no-commit detectada: pulando git add, commit, tag e push]\x1b[0m');
      console.log('\nSucesso! Versão atualizada e bundle gerado com sucesso.\n');
      return;
    }

    console.log('\n[4/5] Criando commit e tag Git...');
    execSync('git add -A', {
      cwd: rootDir,
      stdio: 'inherit',
    });

    try {
      execSync(`git commit -m "${commitMsg.replace(/"/g, '\\"')}"`, {
        cwd: rootDir,
        stdio: 'inherit',
      });
    } catch {
      console.log('   (Nenhuma mudanca nos arquivos para commitar)');
    }

    // Cria a tag anotada com a mesma mensagem
    execSync(`git tag -a ${targetTag} -m "${commitMsg.replace(/"/g, '\\"')}"`, {
      cwd: rootDir,
      stdio: 'inherit',
    });
    console.log(`   OK: Tag ${targetTag} criada com sucesso!`);

    console.log('\n[5/5] Envio para o GitHub:');
    const doPush = await askPreFilled(`Enviar commit e tag ${targetTag} para origin? (S/n): `, 's');

    if (doPush.toLowerCase() === 's') {
      console.log('Enviando branch main...');
      execSync('git push origin main', { cwd: rootDir, stdio: 'inherit' });
      console.log(`Enviando tag ${targetTag}...`);
      execSync(`git push origin ${targetTag}`, { cwd: rootDir, stdio: 'inherit' });
      console.log(`\nSucesso total! Tag ${targetTag} enviada para o GitHub!\n`);
    } else {
      console.log(`\nTag criada apenas localmente. Para enviar manualmente:\n`);
      console.log(`   git push origin main && git push origin ${targetTag}\n`);
    }
  } catch (err) {
    console.error('\nErro durante o processo:', err.message);
    process.exit(1);
  }
}

main();
