// Files & folders batch C: file types. image-file, code-file, spreadsheet-file, audio-file, video-file.
import {make,F,tube,rigid,posed,shown,fill,stroke,circle,polygon,capsule,rect,page,polylineTube,movePath,cross,scaleAbout} from './helpers.mjs';

// ---------- IMAGE FILE ----------
// Two peaks and a sun on the sheet. Light: the sun lifts 0.9 and settles back. Strong: the sun dips, swoops up
// and right with a hit, then sinks back to its place.
const SUN=[12.4,12.8],SR=1.5;
const mountain=polygon([[10.8,23.4],[14.2,17.4],[16.6,20.4],[18.8,17.8],[21.2,23.4]],[.6,.8,.6,.8,.6]);
const sunX=F([[0,0],[.62,0]],[[0,0],[.08,-.3],[.3,.8,'hit'],[.42,.7],[.7,0,'smooth']]);
const sunY=F([[0,0],[.3,-.9],[.62,0,'smooth']],[[0,0],[.08,.3],[.3,-1.8,'hit'],[.42,-1.5],[.7,0,'smooth']]);
const imageFile=make('image-file','Image file',
 'Two peaks and a sun on the sheet. Light: the sun lifts 0.9 and settles back. Strong: the sun dips, swoops up and right with a hit, then sinks back.',[
 ...page('sheet',(t,s)=>[mountain,circle(SUN[0]+sunX(t,s),SUN[1]+sunY(t,s),SR)]),
 stroke('peaks',mountain),
 ...posed([fill('sun',circle(...SUN,SR),'outline')],rigid((t,s)=>({dx:sunX(t,s),dy:sunY(t,s)})))
],.62,.7);
imageFile.duotoneFill=true;

// ---------- CODE FILE ----------
// Two angle brackets on the sheet. Light: they spread 0.5 apart and return. Strong: they close 0.5 first,
// spring 1.0 apart with a hit and settle back.
const LEFT=[[13.9,13.8],[11.4,17.8],[13.9,21.8]],RIGHT=[[18.1,13.8],[20.6,17.8],[18.1,21.8]];
const spread=F([[0,0],[.3,.5],[.6,0,'smooth']],[[0,0],[.1,-.5],[.3,1,'hit'],[.42,.85],[.68,0,'smooth']]);
const poly=p=>`M${p[0]} L${p[1]} L${p[2]}`;
const codeFile=make('code-file','Code file',
 'Two angle brackets on the sheet. Light: they spread half a unit apart and return. Strong: they close first, spring a full unit apart with a hit and settle back.',[
 ...page('sheet',(t,s)=>{const d=spread(t,s);return [movePath(polylineTube(LEFT,1.3),0,0,0,-d,0),movePath(polylineTube(RIGHT,1.3),0,0,0,d,0)];}),
 ...posed([stroke('bracket-l',poly(LEFT))],rigid((t,s)=>({dx:-spread(t,s)}))),
 ...posed([stroke('bracket-r',poly(RIGHT))],rigid((t,s)=>({dx:spread(t,s)})))
],.6,.68);
codeFile.duotoneFill=true;

// ---------- SPREADSHEET FILE ----------
// A 2 x 2 grid on the sheet; a selection cell appears in the first cell and travels through the table.
// Light: appears, glides to the next column, fades. Strong: pops in, hops across and down with hits, fades.
const GRID=[10.8,12.4,21.2,23.6],GX=16,GY=18,CELL=[[13.4,15.2],[18.6,15.2],[18.6,20.8]];
const cellX=F([[0,CELL[0][0]],[.14,CELL[0][0]],[.4,CELL[1][0],'smooth'],[.6,CELL[1][0]]],[[0,CELL[0][0]],[.08,CELL[0][0],'hold'],[.22,CELL[1][0],'hit'],[.34,CELL[1][0],'hold'],[.62,CELL[1][0]]]);
const cellY=F([[0,CELL[0][1]],[.6,CELL[0][1]]],[[0,CELL[0][1]],[.34,CELL[0][1],'hold'],[.46,CELL[2][1],'hit'],[.62,CELL[2][1]]]);
const cellShow=F([[0,0],[.14,1],[.42,1],[.6,0]],[[0,0],[.08,1],[.5,1],[.62,0]]);
const marker=(t,s)=>rect(cellX(t,s)-1.2,cellY(t,s)-1.2,2.4,2.4,.6);
const spreadsheetFile=make('spreadsheet-file','Spreadsheet file',
 'A 2 x 2 table on the sheet; a selected cell appears in the first cell and travels through the table. Light: it glides to the next column and fades. Strong: it pops in, hops across and down with hits, then fades.',[
 ...page('sheet',()=>[rect(GRID[0],GRID[1],GRID[2]-GRID[0],GRID[3]-GRID[1],1)]),
 stroke('table',rect(GRID[0],GRID[1],GRID[2]-GRID[0],GRID[3]-GRID[1],1)),stroke('column',`M${GX} ${GRID[1]} L${GX} ${GRID[3]}`),stroke('row',`M${GRID[0]} ${GY} L${GRID[2]} ${GY}`),
 fill('dividers-solid',cross(GX,GY,GRID[3]-GY,1,.2),'solid'),
 ...shown([fill('cell',marker,'outline'),fill('cell-solid',marker,'solid')],cellShow)
],.6,.62);
spreadsheetFile.duotoneFill=true;

// ---------- AUDIO FILE ----------
// Three level bars on the sheet. Light: each bar bumps up in turn and settles. Strong: each bar leaps with a hit,
// dips below rest and settles, one after the other.
const BARS=[[11.8,2.2,1.2,2.2],[15.4,4.2,1.4,2.2],[19,3,1,1.6]],BY=18;
const bump=(i,l,st)=>F([[0,0],[.08*i,0],[.08*i+.16,l],[.08*i+.44,0,'smooth']],[[0,0],[.06*i,0],[.06*i+.14,st,'hit'],[.06*i+.28,-.5],[.06*i+.44,0]]);
const barH=BARS.map(([,half,l,st],i)=>{const b=bump(i,l,st);return (t,s)=>half+b(t,s);});
const audioFile=make('audio-file','Audio file',
 'Three level bars on the sheet. Light: each bar bumps up in turn and settles. Strong: each bar leaps with a hit, dips below rest and settles, one after the other.',[
 ...page('sheet',(t,s)=>BARS.map(([x],i)=>capsule([x,BY-barH[i](t,s)],[x,BY+barH[i](t,s)],2.2))),
 ...BARS.map(([x],i)=>stroke('bar-'+i,(t,s)=>`M${x} ${BY-barH[i](t,s)} L${x} ${BY+barH[i](t,s)}`))
],.62,.58);
audioFile.duotoneFill=true;

// ---------- VIDEO FILE ----------
// A play triangle on the sheet. Light: it nudges 0.4 right with a 6% swell and returns. Strong: it presses to 85%,
// pops to 110% and 0.5 right with a hit, then settles.
const TC=[15.4,17.8];
const play=polygon([[12.6,12.6],[20.6,17.8],[12.6,23]],[.9,1.1,.9]);
const playScale=F([[0,1],[.22,1.06],[.5,1,'smooth']],[[0,1],[.1,.85],[.26,1.1,'hit'],[.38,.97],[.55,1]]);
const playX=F([[0,0],[.22,.4],[.5,0,'smooth']],[[0,0],[.1,-.2],[.26,.5,'hit'],[.38,.35],[.55,0]]);
const playPose=(x,y,t,s)=>{const p=scaleAbout(x,y,playScale(t,s),...TC);return [p[0]+playX(t,s),p[1]];};
const videoFile=make('video-file','Video file',
 'A play triangle on the sheet. Light: it nudges right with a small swell and returns. Strong: it presses down, pops past size and forward with a hit, then settles.',[
 ...page('sheet',(t,s)=>[movePath(polygon([[12.6,12.6],[20.6,17.8],[12.6,23]].map(p=>scaleAbout(...p,playScale(t,s),...TC)),[.9,1.1,.9]),0,0,0,playX(t,s),0)]),
 ...posed([stroke('play',play)],playPose)
],.5,.55);
videoFile.duotoneFill=true;

export const icons=[imageFile,codeFile,spreadsheetFile,audioFile,videoFile];
