import { execFileSync } from 'node:child_process';
import { readFile, realpath, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const manifestPath = 'scripts/auth-parity.json';
const scriptPath = 'scripts/check-auth-parity.mjs';

function revision(root) {
  const git = (...args) => execFileSync('git', ['-C', root, ...args], {
    encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'],
  }).trim();
  try {
    if (git('rev-parse', '--show-toplevel') !== root) return 'not a Git checkout';
    const branch = git('branch', '--show-current') || '(detached)';
    const head = git('rev-parse', '--verify', 'HEAD');
    const status = git('status', '--short');
    return `${branch} @ ${head}\n${status || 'clean'}`;
  } catch {
    return 'revision/status unavailable';
  }
}

async function sharedBytes(root, file) {
  const resolved = await realpath(path.join(root, file));
  const relative = path.relative(root, resolved);
  if (relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    throw new Error(`${file}: resolves outside checkout`);
  }
  if (!(await stat(resolved)).isFile()) throw new Error(`${file}: not a file`);
  return readFile(resolved);
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length !== 2 || args[0] !== '--peer' || !args[1].trim()) {
    throw new Error('Usage: node scripts/check-auth-parity.mjs --peer <checkout>');
  }
  const root = await realpath(path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'));
  const peer = await realpath(path.resolve(args[1]));
  if (root === peer) throw new Error('Peer must be a different checkout');
  if (!(await stat(peer)).isDirectory()) throw new Error('Peer must be a directory');

  const manifest = JSON.parse(await sharedBytes(root, manifestPath));
  if (
    manifest.version !== 1 || !Array.isArray(manifest.files) || !manifest.files.length ||
    new Set(manifest.files).size !== manifest.files.length ||
    ![manifestPath, scriptPath].every((file) => manifest.files.includes(file)) ||
    manifest.files.some((file) => typeof file !== 'string' || !file ||
      file.includes('\\') || path.posix.isAbsolute(file) ||
      path.posix.normalize(file) !== file || file === '..' || file.startsWith('../'))
  ) {
    throw new Error('Invalid parity manifest');
  }

  const failures = [];
  for (const file of manifest.files) {
    try {
      const [local, remote] = await Promise.all([sharedBytes(root, file), sharedBytes(peer, file)]);
      if (!local.equals(remote)) failures.push(`${file}: content differs`);
    } catch (error) {
      failures.push(`${file}: ${error.message}`);
    }
  }
  console.log(`Local: ${root}\n${revision(root)}`);
  console.log(`Peer: ${peer}\n${revision(peer)}`);
  if (failures.length) throw new Error(failures.join('\n'));
  console.log(`Auth parity passed: ${manifest.files.length} identical files`);
}

main().catch((error) => {
  console.error(`Auth parity failed: ${error.message}`);
  process.exitCode = 1;
});
