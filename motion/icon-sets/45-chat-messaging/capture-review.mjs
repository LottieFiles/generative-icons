// Native review sheets for this set; run after the full browser gate.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawn} from 'node:child_process';
import os from 'node:os';
const root=path.dirname(fileURLToPath(import.meta.url));
const port=Number(process.env.CHAT_REVIEW_CDP||9446);
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'chat-native-review-'));
const chrome=spawn(process.env.CHROME_BIN||'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new',`--user-data-dir=${profile}`,`--remote-debugging-port=${port}`,'--no-first-run','--window-size=1400,1000','about:blank'],{stdio:'ignore'});
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
try{
 let targets;for(let i=0;i<30&&!targets;i++){await sleep(300);try{targets=await(await fetch(`http://127.0.0.1:${port}/json`)).json();}catch{}}
 const ws=new WebSocket(targets.find(t=>t.type==='page').webSocketDebuggerUrl);let id=0;const pending=new Map();
 ws.onmessage=e=>{const m=JSON.parse(e.data);if(pending.has(m.id)){pending.get(m.id)(m);pending.delete(m.id);}};
 await new Promise(r=>ws.onopen=r);
 const send=(method,params={})=>new Promise(r=>{pending.set(++id,r);ws.send(JSON.stringify({id,method,params}));});
 await send('Page.navigate',{url:'http://127.0.0.1:4191/icons/45/verify.html'});await sleep(700);
 const capture=async()=>{
  const {DotLottie}=await import('/vendor/dotlottie-web.js');DotLottie.setWasmUrl(new URL('/vendor/dotlottie-player.wasm',location.href).href);
  const set=await(await fetch('./assets/set.json',{cache:'no-store'})).json();
  const views=[['outline',1.5,'Light'],['outline',2,'Regular'],['outline',2.5,'Bold'],['solid',2,'Solid'],['duotone',2,'Duotone']];
  const sizes=[24,32,48],times=[0,.12,.24,.4,.6,.86],images=[];
  const bind=(p,style,width)=>{p.setTheme('dark');for(const[n,v]of [['light',1.5],['regular',2],['bold',2.5]])p.setScalarSlot('style.outline.'+n,style!=='solid'&&width===v?100:0);p.setScalarSlot('style.solid',style==='solid'?100:0);p.setScalarSlot('style.tint',style==='duotone'?18:0);};
  const player=async(data,size)=>{const canvas=document.createElement('canvas');canvas.width=canvas.height=size;canvas.style.width=canvas.style.height=size+'px';document.body.append(canvas);const p=new DotLottie({canvas,data,autoplay:false,loop:false,useFrameInterpolation:false,renderConfig:{autoResize:false,devicePixelRatio:1,freezeOnOffscreen:false}});await new Promise((r,j)=>{p.addEventListener('load',r);p.addEventListener('loadError',j);});return {p,canvas};};
  const sheet=(w,h)=>{const c=document.createElement('canvas');c.width=w;c.height=h;const x=c.getContext('2d');x.fillStyle='#10171d';x.fillRect(0,0,w,h);x.font='13px monospace';x.fillStyle='#d7e5e7';return[c,x];};
  for(let group=0;group<4;group++){
   const subset=set.icons.slice(group*5,group*5+5),[small,sx]=sheet(960,560),[timing,tx]=sheet(1280,2240);
   sx.fillText('Native 24 / 32 / 48 px; final states; dark theme',20,24);
   views.forEach((v,j)=>sx.fillText(v[2],220+j*145,48));
   for(let row=0;row<subset.length;row++){
    const i=subset[row],data=await(await fetch('./assets/'+i.motions[1].name+'.lottie')).arrayBuffer();sx.fillText(i.icon,14,92+row*92);
    for(const size of sizes){const {p,canvas}=await player(data,size);for(let col=0;col<views.length;col++){bind(p,...views[col]);p.setFrame(i.motions[1].rest);sx.drawImage(canvas,220+col*145+sizes.slice(0,sizes.indexOf(size)).reduce((a,b)=>a+b+8,0),100+row*92+(48-size)/2);}p.destroy();canvas.remove();}
    tx.fillText(i.icon+' — equal elapsed time, native 192 px',14,row*444+22);
    const {p,canvas}=await player(data,192);
    for(let m=0;m<2;m++){p.loadAnimation(i.motions[m].name);bind(p,'outline',2);tx.fillText(i.motions[m].motionId,10,row*444+65+m*210);for(let col=0;col<times.length;col++){p.setFrame(times[col]*60);tx.drawImage(canvas,100+col*194,row*444+34+m*210);tx.fillText(times[col].toFixed(2)+' s',160+col*194,row*444+230+m*210);}}
    p.destroy();canvas.remove();
   }
   images.push({name:`native-small-sizes-${group+1}.png`,data:small.toDataURL()},{name:`native-equal-time-${group+1}.png`,data:timing.toDataURL()});
  }
  return {images,report:{mode:'capture',sizes,times,styles:views,assets:set.icons.map(i=>({name:i.icon,actions:i.animations.map(a=>({name:a.name,sha256:a.sha256}))})),note:'Native rendered small-size final states and equal-time source-frame samples. Normal-speed completion proof is in browser-verification.json.'}};
 };
 const reply=await send('Runtime.evaluate',{expression:'('+capture.toString()+')()',awaitPromise:true,returnByValue:true});
 if(reply.result?.exceptionDetails)throw Error(JSON.stringify(reply.result.exceptionDetails));
 const result=reply.result.result.value;
 for(const i of result.images)fs.writeFileSync(path.join(root,'evidence',i.name),Buffer.from(i.data.split(',')[1],'base64'));
 fs.writeFileSync(path.join(root,'evidence/native-review-capture.json'),JSON.stringify(result.report,null,2)+'\n');
 console.log('Saved 8 native review sheets: 20 icons, five styles, 24/32/48 px and equal-time Light/Strong poses.');ws.close();
}finally{if(chrome.exitCode===null){const exited=new Promise(r=>chrome.once('exit',r));chrome.kill();await Promise.race([exited,sleep(3000)]);}fs.rmSync(profile,{recursive:true,force:true,maxRetries:5,retryDelay:100});}
