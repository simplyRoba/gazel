import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifestPath = 'scripts/auth-parity.json';
const scriptPath = 'scripts/check-auth-parity.mjs';
const manifest = JSON.parse(await readFile(path.join(root, manifestPath)));

async function fixture(t) {
  const target = path.join(root, 'target');
  await mkdir(target, { recursive: true });
  const directory = await mkdtemp(path.join(target, 'auth-parity-test-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const local = path.join(directory, 'local');
  const peer = path.join(directory, 'peer');
  for (const checkout of [local, peer]) {
    for (const file of manifest.files) {
      const destination = path.join(checkout, file);
      await mkdir(path.dirname(destination), { recursive: true });
      await writeFile(destination, await readFile(path.join(root, file)));
    }
  }
  return { local, peer, directory };
}

function run(local, args) {
  return spawnSync(process.execPath, [path.join(local, scriptPath), ...args], {
    cwd: root, encoding: 'utf8', timeout: 10_000,
  });
}

function failed(result, pattern) {
  assert.equal(result.error, undefined);
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stderr, pattern);
  assert.doesNotMatch(result.stdout, /Auth parity passed/);
}

test('requires an explicit peer and rejects unknown arguments', () => {
  for (const args of [[], ['--peer'], ['--peer', ''], ['--unknown', root], ['--peer', root, '--extra']]) {
    failed(run(root, args), /Usage:/);
  }
});

test('compares identical independently located fixtures in both directions', async (t) => {
  const { local, peer } = await fixture(t);
  for (const [source, other] of [[local, peer], [peer, local]]) {
    const result = run(source, ['--peer', other]);
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stdout, new RegExp(`${manifest.files.length} identical files`));
    assert.match(result.stdout, /not a Git checkout/);
  }
});

test('rejects missing and self peers', async (t) => {
  const { local, directory } = await fixture(t);
  failed(run(local, ['--peer', path.join(directory, 'missing')]), /ENOENT/);
  failed(run(local, ['--peer', local]), /different checkout/);
});

test('rejects missing files on either side', async (t) => {
  const { local, peer } = await fixture(t);
  const file = 'src/auth/session.rs';
  await rm(path.join(peer, file));
  failed(run(local, ['--peer', peer]), /src\/auth\/session\.rs.*ENOENT/);
  failed(run(peer, ['--peer', local]), /src\/auth\/session\.rs.*ENOENT/);
});

test('rejects source, checker, and manifest drift', async (t) => {
  const { local, peer } = await fixture(t);
  for (const file of ['src/auth/routes.rs', scriptPath, manifestPath]) {
    const original = await readFile(path.join(peer, file));
    await writeFile(path.join(peer, file), Buffer.concat([original, Buffer.from('\n')]));
    failed(run(local, ['--peer', peer]), /content differs/);
    await writeFile(path.join(peer, file), original);
  }
});

test('rejects empty, duplicate, unsafe, and non-self-covering manifests', async (t) => {
  const { local, peer } = await fixture(t);
  for (const files of [[], [manifestPath, scriptPath, scriptPath], [manifestPath, scriptPath, '../outside'], [manifestPath]]) {
    await writeFile(path.join(local, manifestPath), JSON.stringify({ version: 1, files }));
    failed(run(local, ['--peer', peer]), /Invalid parity manifest/);
  }
});

test('rejects shared-file symlinks escaping the peer checkout', async (t) => {
  const { local, peer, directory } = await fixture(t);
  const outside = path.join(directory, 'outside.rs');
  const file = 'src/auth/config.rs';
  await writeFile(outside, await readFile(path.join(local, file)));
  await rm(path.join(peer, file));
  await symlink(outside, path.join(peer, file));
  failed(run(local, ['--peer', peer]), /resolves outside checkout/);
});
