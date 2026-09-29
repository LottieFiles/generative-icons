// Food & Drink batch C: chef-hat, cutlery, pizza, burger, cake.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,bothFill,rotate,holes} from './helpers.mjs';

// ---------- CHEF HAT ----------
// Toque: a puffed crown over a straight band, one outline, with a crease across at the band's top.
// A tip of the hat: it lifts and tilts about its lower-left corner. Strong hops and lands with a squash.
const hatD='M10.2 26.4 L10.2 16.2 C7.3 15.8 5.8 13.3 6.6 10.8 C7.5 8.3 10.5 7.2 12.8 8.3 C13.6 5.9 15.7 4.9 17.8 5.4 C19.6 5.8 20.9 7.2 21.2 8.9 C23.7 8.6 26 10.7 25.6 13.3 C25.3 15.2 23.8 16.3 21.8 16.2 L21.8 26.4 Z';
const CORNER=[10.2,26.4];
const tipA=F([[0,0],[.16,4],[.44,0,'smooth']],[[0,0],[.06,-1.5],[.22,9],[.4,0,'hit'],[.5,1.2],[.6,0]]);
const hop=F([[0,0],[.44,0]],[[0,0],[.06,.3],[.22,-1.8],[.4,0,'hit'],[.6,0]]);
const squash=F([[0,1],[.44,1]],[[0,1],[.4,1],[.44,.95],[.52,1.01],[.6,1]]);
const hatPose=(x,y,t,s)=>{const [rx,ry]=rotate(x,y,tipA(t,s),...CORNER),q=squash(t,s);return [16+(rx-16)*(1+(1-q)*.5),26.4+(ry-26.4)*q+hop(t,s)];};
const chefHat=make('chef-hat','Chef hat',
 'Light: the hat tips 4° off its lower corner and settles back. Strong: it winds back, hops 1.8 units while tipping 9°, lands with a small squash and one little rock.',
 posed([...framed('hat',hatD),stroke('crease','M10.2 21.6 L21.8 21.6')],hatPose),.44,.6);
chefHat.duotoneFill=true;

// ---------- CUTLERY ----------
// Fork (left) and knife (right) stand side by side, each pivoting on its handle end. They tip toward
// each other and clink at the top: Light a soft near-miss, Strong a spread, a hard clink and a rebound.
const FORK=[11,27],KNIFE=[21.9,27];
const lean=F([[0,0],[.16,3.5],[.44,0,'smooth']],[[0,0],[.08,-4],[.22,5.2,'hit'],[.34,-1.6],[.46,.6],[.56,0]]);
const knifeD='M20.4 16.4 L20.4 6.4 C20.4 5.4 21.2 4.9 22 5.4 C23.8 6.8 24.5 10.8 24.2 16.4 Z';
const cutlery=make('cutlery','Cutlery',
 'Light: fork and knife tip 3.5° toward each other and back. Strong: they spread apart, swing in and clink at the tips, then rebound and settle.',[
 ...posed([...tube('tines','M7.5 5 L7.5 10.6 C7.5 12.5 9 13.6 11 13.6 C13 13.6 14.5 12.5 14.5 10.6 L14.5 5'),...tube('fork','M11 5 L11 27')],rigid((t,s)=>({angle:lean(t,s),pivot:FORK}))),
 ...posed([...framed('blade',knifeD),...tube('handle','M21.9 16.4 L21.9 27')],rigid((t,s)=>({angle:-lean(t,s),pivot:KNIFE})))
],.44,.56);
cutlery.duotoneFill=true;

// ---------- PIZZA ----------
// A slice lifted by its crust: the soft tip droops under its weight and springs back. Strong lifts the
// slice 2 units and the droop swings deeper with one rebound. Toppings ride the bending cheese.
// Solid is the slice with the toppings cut out as real counters; the crust line is Outline detail.
const TOPS=[[13.9,14.3],[18.1,14.6],[16,18.7]];
const sliceD='M6.5 8.5 C11.5 5.8 20.5 5.8 25.5 8.5 L16.4 26 C16.2 26.4 15.8 26.4 15.6 26 Z';
const droop=F([[0,0],[.14,1.2],[.48,0,'smooth']],[[0,0],[.08,-.4],[.26,3],[.42,-.8],[.56,.3],[.66,0]]);
const lift=F([[0,0],[.48,0]],[[0,0],[.08,0],[.26,-2],[.5,-.6],[.66,0]]);
const slicePose=(x,y,t,s)=>{const w=Math.max(0,(y-8.5)/17.5)**2;return [x,y+droop(t,s)*w+lift(t,s)];};
const pizza=make('pizza','Pizza',
 'Light: the slice\'s tip sags 1.2 units and springs back. Strong: the slice is lifted 2 units, its tip droops 3 units, rebounds once and settles.',
 posed([...framed('slice',sliceD,holes(sliceD,...TOPS.map(([x,y])=>circle(x,y,1.35)))),stroke('crust','M8 11.2 C12.5 8.9 19.5 8.9 24 11.2'),
  ...TOPS.map(([x,y],i)=>fill('topping-'+i,circle(x,y,1.25),'outline'))],slicePose),.48,.66);
pizza.duotoneFill=true;

// ---------- BURGER ----------
// Top bun, patty and bottom bun, each 1 unit clear at Bold. Light: the top bun hops and lands.
// Strong: it lifts, slams down and the patty squashes and widens under it, then everything rebounds.
const topBun='M5.5 12.4 C5.5 7.4 10.3 5 16 5 C21.7 5 26.5 7.4 26.5 12.4 Z';
const botBun='M6 23.5 L26 23.5 L26 25.4 C26 26.7 25 27.6 23.7 27.6 L8.3 27.6 C7 27.6 6 26.7 6 25.4 Z';
const bunY=F([[0,0],[.12,-1.1],[.3,0,'hit'],[.4,0]],[[0,0],[.12,-1.6],[.26,.18,'hit'],[.36,-.25],[.46,0]]);
const press=F([[0,1],[.4,1]],[[0,1],[.24,1],[.28,.82,'hit'],[.38,1.04],[.48,.98],[.56,1]]);
const pattyPose=(x,y,t,s)=>{const k=press(t,s);return [16+(x-16)*(1+(1-k)*.25),17.9+(y-17.9)*k];};
const burger=make('burger','Burger',
 'Light: the top bun hops 1.1 units and lands back on the stack. Strong: it lifts 1.6 units and slams down; the patty squashes to 82% and widens, then the stack rebounds and settles.',[
 // Solid top bun carries the seeds as counters.
 ...posed([...framed('top-bun',topBun,holes(topBun,circle(12.4,8.8,.9),circle(18.8,8.4,.9))),fill('seed-0',circle(12.4,8.8,.8),'outline'),fill('seed-1',circle(18.8,8.4,.8),'outline')],rigid((t,s)=>({dy:bunY(t,s)}))),
 ...posed(framed('patty',rect(5.2,15.9,21.6,4,2)),pattyPose),
 ...framed('bottom-bun',botBun)
],.4,.56);
burger.duotoneFill=true;

// ---------- CAKE ----------
// The candle lights: a flame grows from the wick and flickers. Prepared state is the unlit cake.
// Strong: the flame flares to 120% and sways before settling. The lit flame is retained.
const cakeD='M6 17 L26 17 L26 25.4 C26 26.6 25 27.6 23.8 27.6 L8.2 27.6 C7 27.6 6 26.6 6 25.4 Z';
const flameD='M16 4.4 C17.6 6.3 18.1 7.3 18.1 8.2 C18.1 9.4 17.2 10.2 16 10.2 C14.8 10.2 13.9 9.4 13.9 8.2 C13.9 7.3 14.4 6.3 16 4.4 Z';
const WICK=[16,10.2];
const grow=F([[0,0],[.06,0],[.4,1,'smooth']],[[0,0],[.06,0,'hold'],[.2,1.2],[.34,.92],[.46,1]]);
const sway=F([[0,0],[.3,0],[.4,.25],[.56,0,'smooth']],[[0,0],[.18,0],[.28,-.6],[.4,.4],[.52,-.15],[.62,0]]);
const flamePose=(x,y,t,s)=>{const k=Math.max(.001,grow(t,s)),h=(WICK[1]-y)/5.8;return [WICK[0]+(x-WICK[0])*k+sway(t,s)*h*2,WICK[1]+(y-WICK[1])*k];};
const cake=make('cake','Cake',
 'Light: a flame grows from the candle\'s wick and flickers once. Strong: it catches with a flare to 120%, sways, and settles into a steady flame. The lit candle is retained; preview reset rearms.',[
 ...framed('cake',cakeD),
 stroke('icing','M6 20.6 C7.7 22.2 9.3 22.2 11 20.6 C12.7 22.2 14.3 22.2 16 20.6 C17.7 22.2 19.3 22.2 21 20.6 C22.7 22.2 24.3 22.2 26 20.6'),
 ...tube('candle','M16 12.6 L16 17'),
 ...shown(posed(bothFill('flame',flameD),flamePose),(t,s)=>grow(t,s)>.06?1:0)
],.56,.62,['flame','flame-solid']);
cake.duotoneFill=true;

export const icons=[chefHat,cutlery,pizza,burger,cake];
