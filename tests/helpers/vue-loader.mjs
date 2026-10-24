import { readFile } from 'node:fs/promises';
import { parse, compileScript } from '@vue/compiler-sfc';
import ts from 'typescript';

// Compile the real SFC in the isolated Node DOM test, without a browser bundler.
export async function resolve(specifier, context, nextResolve) {
  if (specifier.startsWith('@/')) {
    const path = specifier.slice(2);
    const url = new URL(`../../src/${path}${/\.(vue|ts)$/.test(path) ? '' : '.ts'}`, import.meta.url);
    return { url: url.href, shortCircuit: true };
  }
  return nextResolve(specifier, context);
}

export async function load(url, context, nextLoad) {
  if (url.endsWith('.vue') || url.endsWith('.ts')) {
    let source = await readFile(new URL(url), 'utf8');
    if (url.endsWith('.vue')) {
      const { descriptor } = parse(source);
      source = compileScript(descriptor, { id: 'select-test', inlineTemplate: true }).content;
    }
    return {
      format: 'module', shortCircuit: true,
      source: ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 } }).outputText,
    };
  }
  return nextLoad(url, context);
}
