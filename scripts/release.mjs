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
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
const updateServicePath = path.join(rootDir, 'src', 'core', 'services', 'update.service.ts');

function readCurrentVersion() {
  const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
  return pkg.version;
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

function ask(rl, query, defaultValue) {
  return new Promise((resolve) => {
    rl.question(query, (answer) => {
      resolve(answer.trim() || defaultValue);
    });
  });
}

async function main() {
  console.log('\n🚀 \x1b[1m\x1b[36mXTERMINIUM - Release & Bump CLI\x1b[0m\n');

  const currentVersion = readCurrentVersion();

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  try {
    // 1. Pergunta a nova versão
    const targetVersion = await ask(
      rl,
      `📦 Versão do app [atual: \x1b[33m${currentVersion}\x1b[0m]: `,
      currentVersion
    );

    // Sugere a tag padronizada (ex: v0.0.9-alpha)
    const suggestedTag = targetVersion.startsWith('v') ? targetVersion : `v${targetVersion}`;

    // 2. Pergunta o nome da tag
    const targetTag = await ask(
      rl,
      `🏷️  Nome da Tag Git [sugestão: \x1b[32m${suggestedTag}\x1b[0m]: `,
      suggestedTag
    );

    console.log('\n----------------------------------------');
    console.log(`📌 Nova versão : \x1b[33m${targetVersion}\x1b[0m`);
    console.log(`🏷️  Nova Tag    : \x1b[32m${targetTag}\x1b[0m`);
    console.log('----------------------------------------\n');

    const confirm = await ask(rl, 'Confirma as alterações e gerar o bundle? (s/N): ', 'n');
    if (confirm.toLowerCase() !== 's') {
      console.log('❌ Cancelado pelo usuário.');
      rl.close();
      process.exit(0);
    }
    rl.close();

    console.log('\n📝 1/5 Atualizando arquivos de versão...');
    updateVersionsInFiles(targetVersion);
    console.log('   ✓ package.json');
    console.log('   ✓ tauri.conf.json');
    console.log('   ✓ Cargo.toml');
    console.log('   ✓ update.service.ts');

    console.log('\n🔒 2/5 Sincronizando Cargo.lock...');
    execSync('cargo check', { cwd: path.join(rootDir, 'src-tauri'), stdio: 'inherit' });

    console.log('\n📦 3/5 Gerando o bundle .deb...');
    execSync('npx tauri build --bundles deb', { cwd: rootDir, stdio: 'inherit' });

    console.log('\n🌿 4/5 Criando commit e tag Git...');
    execSync('git add package.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json', {
      cwd: rootDir,
      stdio: 'inherit',
    });

    try {
      execSync(`git commit -m "chore: bump version to ${targetVersion}"`, {
        cwd: rootDir,
        stdio: 'inherit',
      });
    } catch {
      console.log('   (Nenhuma mudança para commitar)');
    }

    // Cria a tag local
    execSync(`git tag -a ${targetTag} -m "Release ${targetTag}"`, {
      cwd: rootDir,
      stdio: 'inherit',
    });
    console.log(`   ✓ Tag ${targetTag} criada com sucesso!`);

    console.log('\n🚀 5/5 Deseja enviar para o GitHub agora?');
    const pushRl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });
    const doPush = await ask(pushRl, `Enviar commit e tag ${targetTag} para origin? (S/n): `, 's');
    pushRl.close();

    if (doPush.toLowerCase() === 's') {
      console.log('Enviando commit...');
      execSync('git push origin main', { cwd: rootDir, stdio: 'inherit' });
      console.log(`Enviando tag ${targetTag}...`);
      execSync(`git push origin ${targetTag}`, { cwd: rootDir, stdio: 'inherit' });
      console.log(`\n🎉 \x1b[32mSucesso total! Tag ${targetTag} enviada para o GitHub!\x1b[0m\n`);
    } else {
      console.log(`\n⚠️  Tag ${targetTag} criada apenas localmente. Quando quiser enviar rode:\n`);
      console.log(`   git push origin main && git push origin ${targetTag}\n`);
    }
  } catch (err) {
    console.error('\n❌ Erro durante o processo:', err.message);
    process.exit(1);
  }
}

main();
