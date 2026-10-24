import { expect, test } from 'bun:test';

test('comparison calendar selects only valid sessions and stays synchronized with the dropdown', async () => {
  const child = Bun.spawn(['node', '--no-warnings', '--loader', './tests/helpers/vue-loader.mjs', 'tests/helpers/check-session-calendar.ts'], {
    stdout: 'pipe', stderr: 'pipe',
  });
  const [code, stdout, stderr] = await Promise.all([
    child.exited, new Response(child.stdout).text(), new Response(child.stderr).text(),
  ]);
  if (code !== 0) throw new Error(`${stdout}\n${stderr}`);
  expect(stdout).toContain('checks passed');
}, 15000);
