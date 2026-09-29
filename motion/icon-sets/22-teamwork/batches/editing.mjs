// teamwork batch 04 · shared editing · first draft (Claude), unverified geometry
import {stroke,fill,tube,framed,posed,circle,rect,polygon,capsule,arc,hole,holes,rigid,track,mix,clamp,timePair,make,curve} from './helpers.mjs';

// Custom reverse for state icons whose Solid housing carries cutouts: rigid slide back, nothing fades.
const rv=t=>track([[0,1],[.3,0]],t);
const sv=(t,s,r,L,S)=>r?mix(L[0][1],L.at(-1)[1],rv(t)):curve(t,s,L,S);
const state=i=>({...i,stateful:true,initialState:'prepared',finalState:'result',previewReset:true,reverseSuffix:'-reset',forwardLabel:'Play action',reverseLabel:'Reset preview',reverseTimings:timePair(.3,.3)});

function sharedDocument(){
 // forward drivers (L/S) and explicit reverse tracks; endpoints match exactly
 const fw=(t,s,L,S)=>curve(t,s,L,S),rv=(t,K)=>track(K,t);
 const xa=(t,s,r)=>r?rv(t,[[.1,12.4],[.3,16,'smooth']]):fw(t,s,[[0,16],[.08,16],[.5,12.4,'smooth']],[[0,16],[.06,16,'hold'],[.24,12.1],[.36,12.4]]);
 const rr=(t,s,r)=>r?rv(t,[[.1,23.4],[.3,19.4,'smooth']]):fw(t,s,[[0,19.4],[.08,19.4],[.5,23.4,'smooth']],[[0,19.4],[.06,19.4,'hold'],[.3,23.4]]);
 const xb=(t,s,r)=>r?20:fw(t,s,[[0,20]],[[0,20],[.3,21.2],[.46,20]]);
 const vb=(t,s,r)=>clamp(r?rv(t,[[0,1],[.1,0,'out']]):fw(t,s,[[.5,0],[.76,1,'out']],[[.3,0],[.44,1]]));
 const bust=(L,R,z)=>`M${L} 24.6 L${L} 23.2 C${L} 22 ${L+1} 21 ${L+2.2} 21 L${R-2.2} 21 C${R-1} 21 ${R} 22 ${R} 23.2 L${R} 24.6${z?' Z':''}`;
 const page=polygon([[5,4],[22,4],[27,9],[27,28.4],[5,28.4]],[2.2,1,1,2.2,2.2]);
 const parts=[
  fill('page-interior',page,'tint'),
  stroke('page',page),
  stroke('fold','M22 5 L22 7.2 C22 8.2 22.8 9 23.8 9 L26.2 9'), // opaque joins: fold-start into corner cut, fold-end into page right edge
  stroke('head-a',circle(16,15,1.9),{pose:rigid((t,s,r)=>({dx:xa(t,s,r)-16}))}),
  stroke('head-b',circle(20,15,1.9),{pose:rigid((t,s,r)=>({dx:xb(t,s,r)-20})),visibility:vb}),
  stroke('shoulders',(t,s,r)=>bust(xa(t,s,r)-3.4,rr(t,s,r),0)),
  fill('page-solid',(t,s,r)=>holes(page,circle(xa(t,s,r),15,2.3),circle(xb(t,s,r),15,Math.max(0,2.3*vb(t,s,r))),bust(xa(t,s,r)-3.4,rr(t,s,r),1)),'solid'),
 ];
 return {...make('shared-document','Shared document','A collaborator joins the page: the person shifts aside, the shared shoulder line opens, and a second head appears; the pair stays.',parts,.8,.5),
  stateful:true,initialState:'prepared',finalState:'result',previewReset:true,reverseSuffix:'-reset',forwardLabel:'Play action',reverseLabel:'Reset preview',reverseTimings:timePair(.3,.3),duotoneFill:true};
}

function coEdit(){
 const EA=[[0,6],[.06,6],[.9,19,'smooth']],SEA=[[0,6],[.04,6],[.4,19]];
 const EC=[[.12,26],[1,13,'smooth']],SEC=[[.08,26],[.45,13]];
 const ea=(t,s)=>curve(t,s,EA,SEA),ec=(t,s)=>curve(t,s,EC,SEC);
 const parts=[
  ...tube('line-b','M6 16 L26 16'),
  ...tube('line-a',(t,s)=>`M6 9 L${ea(t,s)} 9`,{visibility:(t,s)=>ea(t,s)>6.4?1:0}),
  ...tube('line-c',(t,s)=>`M26 23 L${ec(t,s)} 23`,{visibility:(t,s)=>ec(t,s)<25.6?1:0}),
  ...tube('caret-a','M9.7 5.6 L9.7 12.4',{pose:rigid((t,s)=>({dx:ea(t,s)-6}))}),
  ...tube('caret-c','M22.3 19.6 L22.3 26.4',{pose:rigid((t,s)=>({dx:ec(t,s)-26}))}),
 ];
 return {...make('co-edit','Co-edit','Two carets type toward each other on the same document; both written lines remain.',parts,1,.5,['line-a','line-c','caret-a','caret-c']),duotoneFill:false};
}

function liveCursors(){
 const P=[[0,0],[0,9.4],[2.55,7.05],[4.6,11.05],[6.45,10.2],[4.5,6.3],[7.65,6.3]];
 const ptr=(x,y)=>polygon(P.map(([a,b])=>[a+x,b+y]),[.5,.6,.5,.5,.5,.5,.6]);
 const ax=[[0,0],[.35,-1.2],[.9,0]],sax=[[0,0],[.22,-3],[.48,.4],[.7,0]],ay=[[0,0],[.35,-.8],[.9,0]],say=[[0,0],[.22,-2.2],[.48,.3],[.7,0]];
 const bx=[[.06,0],[.42,1.2],[.9,0]],sbx=[[.04,0],[.26,2.4],[.52,-.3],[.7,0]],by=[[.06,0],[.42,1.5],[.9,0]],sby=[[.04,0],[.26,3],[.52,-.3],[.7,0]];
 const parts=[
  ...posed(framed('a',ptr(7,6)),rigid((t,s)=>({dx:curve(t,s,ax,sax),dy:curve(t,s,ay,say)}))),
  ...posed(framed('b',ptr(18.3,14)),rigid((t,s)=>({dx:curve(t,s,bx,sbx),dy:curve(t,s,by,sby)}))),
 ];
 return {...make('live-cursors','Live cursors','Two collaborator pointers glide apart on the canvas and settle back to rest.',parts,.9,.7),duotoneFill:true};
}

function followCollaborator(){
 const LD=[[0,0],[.06,0],[.55,3.6]],SLD=[[0,0],[.05,0],[.28,3.6]],FR=[[.16,0],[.8,3.6]],SFR=[[.12,0],[.36,4],[.5,3.6]];
 const lead=rigid((t,s,r)=>({dx:sv(t,s,r,LD,SLD)})),frame=rigid((t,s,r)=>({dx:sv(t,s,r,FR,SFR)}));
 const parts=[
  ...posed(tube('corner-tl','M5 10 L5 7.9 C5 6.57 6.07 5.5 7.4 5.5 L9.5 5.5'),frame),
  ...posed(tube('corner-tr','M19.5 5.5 L21.6 5.5 C22.93 5.5 24 6.57 24 7.9 L24 10'),frame),
  ...posed(tube('corner-br','M24 22 L24 24.1 C24 25.43 22.93 26.5 21.6 26.5 L19.5 26.5'),frame),
  ...posed(tube('corner-bl','M9.5 26.5 L7.4 26.5 C6.07 26.5 5 25.43 5 24.1 L5 22'),frame),
  ...posed(framed('head',circle(14.5,12.5,2.4)),lead),
  ...posed(tube('shoulders',arc(14.5,22.6,4,180,360)),lead),
 ];
 return {...state(make('follow-collaborator','Follow collaborator','The followed person moves; your viewport brackets catch up and hold on them.',parts,.8,.55)),duotoneFill:true};
}

function suggestedEdit(){
 const W=[[0,8],[.05,8],[.5,14]],SW=[[0,8],[.04,8],[.22,14.6],[.34,14]],V=[[.05,0],[.3,1,'out']],SV=[[.04,0],[.16,1]],E=[[.5,15.6],[1.05,22.4,'smooth']],SE=[[.3,15.6],[.55,22.4]];
 const w=(t,s)=>curve(t,s,W,SW),v=(t,s)=>clamp(curve(t,s,V,SV)),e=(t,s)=>curve(t,s,E,SE);
 const pill=(t,s)=>rect(12,12,w(t,s),8,4);
 const parts=[
  ...tube('line-1','M6 8 L26 8'),...tube('line-3','M6 24 L26 24'),...tube('stub','M6 16 L8.4 16'),
  fill('pill-interior',pill,'tint',{visibility:v}),
  stroke('pill-frame',pill,{visibility:v}),
  // Solid: the inserted text is a transparent slot inside the pill, so it stays legible in one colour.
  fill('pill-solid',(t,s)=>hole(pill(t,s),capsule([15.6,16],[Math.max(e(t,s),16.4),16],2.6)),'solid',{visibility:v}),
  stroke('insert',(t,s)=>`M15.6 16 L${e(t,s)} 16`,{visibility:(t,s)=>e(t,s)>16?1:0}),
 ];
 return {...make('suggested-edit','Suggested edit','A suggestion pill opens after the existing text and the proposed insertion writes inside it; the suggestion stays pending.',parts,1.05,.55,['pill','insert']),duotoneFill:true};
}

export const icons=[sharedDocument(),coEdit(),liveCursors(),followCollaborator(),suggestedEdit()];
