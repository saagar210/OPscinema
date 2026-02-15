import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(process.cwd(), 'src');
const allow = new Set([path.resolve(root, 'ipc/client.ts')]);
const allowClientInvoke = new Set([path.resolve(root, 'ipc/generated.ts')]);

const violations = [];

function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const abs = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(abs);
      continue;
    }
    if (!abs.endsWith('.ts') && !abs.endsWith('.tsx')) {
      continue;
    }
    const src = fs.readFileSync(abs, 'utf8');
    const hasDirectInvoke =
      src.includes('@tauri-apps/api/core') ||
      src.includes('__TAURI__') ||
      src.includes('core.invoke(');
    const hasRawClientInvoke =
      src.includes('client.invoke(') || src.includes('client.invoke<');

    if (hasDirectInvoke && !allow.has(abs)) {
      violations.push(abs);
    }
    if (hasRawClientInvoke && !allowClientInvoke.has(abs)) {
      violations.push(abs);
    }
  }
}

walk(root);

if (violations.length > 0) {
  console.error('Typed IPC violation(s) detected:');
  for (const file of violations) {
    console.error(` - ${file}`);
  }
  process.exit(1);
}

console.log('Typed IPC guard passed.');
