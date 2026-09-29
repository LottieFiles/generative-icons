// Exercise packed artifacts in clean apps, never workspace symlinks.
import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../', import.meta.url));
const release = path.resolve(process.argv[2] || path.join(root, 'dist/packages'));
const scratch = path.join(root, '.local/consumer-fixtures');
fs.mkdirSync(scratch, { recursive: true });
const runRoot = fs.mkdtempSync(path.join(scratch, 'run-'));
const packages = JSON.parse(fs.readFileSync(path.join(release, 'release.json'))).packages;
const internal = Object.fromEntries(packages.map(item => [item.name, 'file:' + path.join(release, item.file)]));
function run(command, args, cwd) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd, env: { ...process.env, NEXT_TELEMETRY_DISABLED: '1' }, stdio: ['ignore', 'pipe', 'pipe'] });
    let output = ''; child.stdout.on('data', data => { output += data; }); child.stderr.on('data', data => { output += data; });
    child.on('error', reject); child.on('exit', code => { fs.appendFileSync(path.join(cwd, 'verification.log'), output); code === 0 ? resolve(output) : reject(Error(`${command} failed in ${cwd}\n${output.slice(-8000)}`)); });
  });
}
function write(dir, file, value) { const target = path.join(dir, file); fs.mkdirSync(path.dirname(target), { recursive: true }); fs.writeFileSync(target, value); }
async function fixture(kind) {
  const dir = path.join(runRoot, kind); fs.mkdirSync(dir);
  const dependencies = { ...internal, react: '19.3.0', 'react-dom': '19.3.0' };
  const devDependencies = { typescript: '7.0.2', '@types/react': '19.3.0', '@types/react-dom': '19.3.0', '@types/node': '26.6.2', ...(kind === 'next' ? { next: '16.3.6' } : { vite: '8.3.1' }) };
  write(dir, 'package.json', JSON.stringify({ name: `icons-${kind}-fixture`, private: true, type: 'module', dependencies, devDependencies, scripts: { build: kind === 'next' ? 'next build --webpack' : 'tsc --noEmit && vite build', dev: kind === 'next' ? 'next dev --webpack' : 'vite --host 127.0.0.1' } }, null, 2));
  await run('npm', ['install', '--ignore-scripts', '--@lottiefiles:registry=https://registry.npmjs.org/'], dir);
  await run('npm', ['exec', '--offline', '--', 'lottie-icons', 'init'], dir);
  await run('npm', ['exec', '--offline', '--', 'lottie-icons', 'add', 'menu', 'heart'], dir);
  const tsconfig = { compilerOptions: { target: 'ES2022', lib: ['DOM', 'DOM.Iterable', 'ESNext'], module: 'ESNext', moduleResolution: 'Bundler', jsx: 'react-jsx', strict: true, skipLibCheck: false, noEmit: true, allowJs: true, esModuleInterop: true }, include: ['**/*.ts', '**/*.tsx'], exclude: ['node_modules'] };
  write(dir, 'tsconfig.json', JSON.stringify(tsconfig));
  const react = `'use client';\nimport React, {useState} from 'react';\nimport {AnimatedIcon} from '@lottiefiles/generative-icons-react';\nimport icon, {src} from '${kind === 'next' ? '..' : '.'}/lottie-icons/menu.mjs';\nexport default function App(){const [state,setState]=useState<string>(icon.startState);const [color,setColor]=useState('#008c76');const [visible,setVisible]=useState(true);const [ready,setReady]=useState(false);return <main><h1>${kind} consumer</h1><button onClick={()=>setState(state===icon.startState?icon.endState:icon.startState)}>Toggle state</button><button onClick={()=>setColor('#d43155')}>Change color</button><button onClick={()=>setVisible(!visible)}>Mount or remove</button><p id="state">{state}</p><p id="ready">{String(ready)}</p>{visible&&<AnimatedIcon icon={icon} src={src} size={48} state={state} color={color} label="Menu state" onReady={()=>setReady(true)} onError={error=>{document.body.dataset.error=error.message}}/>}</main>};`;
  if (kind === 'next') {
    write(dir, 'app/page.tsx', react); write(dir, 'app/layout.tsx', "import type {ReactNode} from 'react'; export default function Layout({children}:{children:ReactNode}) {return <html lang=\"en\"><body>{children}</body></html>}");
  } else {
    write(dir, 'index.html', '<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Icon consumer</title></head><body><div id="root"></div><script type="module" src="/main.tsx"></script></body></html>');
    if (kind === 'react') { write(dir, 'App.tsx', react); write(dir, 'main.tsx', "import React from 'react';import {createRoot} from 'react-dom/client';import App from './App';createRoot(document.getElementById('root')!).render(<React.StrictMode><App/></React.StrictMode>);"); }
    else write(dir, 'main.tsx', "import {createIcon} from '@lottiefiles/generative-icons-runtime';import icon,{src} from './lottie-icons/menu.mjs';document.getElementById('root')!.innerHTML='<h1>Vanilla consumer</h1><canvas width=48 height=48 aria-label=Menu></canvas><button>Toggle state</button><p id=state></p>';const canvas=document.querySelector('canvas')!;const handle=await createIcon(canvas,{icon,src,state:icon.startState});document.body.dataset.ready='true';document.getElementById('state')!.textContent=handle.state;document.querySelector('button')!.onclick=async()=>{await handle.setState(handle.state===icon.startState?icon.endState:icon.startState);document.getElementById('state')!.textContent=handle.state};window.addEventListener('pagehide',()=>handle.destroy());");
  }
  await run('npm', ['run', 'build'], dir);
  console.log(`PASS ${kind}: packed install, CLI setup/add, type checks and production build (${dir})`);
  return { kind, directory: dir, status: 'build-pass-browser-review-separate' };
}
const results = await Promise.allSettled(['vanilla', 'react', 'next'].map(fixture));
fs.writeFileSync(path.join(runRoot, 'results.json'), JSON.stringify(results.map(result => result.status === 'fulfilled' ? result.value : { error: result.reason.message }), null, 2));
for (const result of results) if (result.status === 'rejected') { console.error(result.reason.message); process.exitCode = 1; }
console.log('Fixture evidence: ' + runRoot);
