// forms-input-a.mjs — checkbox, radio-button, toggle-switch, text-field, number-field (revision 02)
import {make,toggle,geometricRearm,F,tube,stroke,fill,rect,circle,rigid,posed,shown,holes,annulus,framed,mix} from './helpers.mjs';
const R=(fwd,rev)=>(t,s,r)=>(r?rev:fwd)(t,s);
const strokePair=(name,d,extra={})=>[{name,d,style:'outline',type:'stroke',width:2,...extra},{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6,...extra}];

// ---------- CHECKBOX: unchecked -> checked. Pen-tip collapse until each segment is reached. ----------
const checkbox=(()=>{
 const P0=[10.5,16.6],P1=[14.3,20.4],P2=[21.8,12.2],L1=Math.hypot(3.8,3.8),L2=Math.hypot(7.5,8.2),LT=L1+L2;
 const draw=R(F([[0,0],[.08,0],[.44,1,'smooth']],[[0,0],[.14,0],[.2,.22,'pass'],[.30,1]]),
              F([[0,1],[.32,0,'smooth']],[[0,1],[.26,0]]));
 const mark=(t,s,r)=>{const d=draw(t,s,r)*LT,q=Math.min(d,L1)/L1,w=Math.max(0,d-L1)/L2;
  const a=[P0[0]+(P1[0]-P0[0])*q,P0[1]+(P1[1]-P0[1])*q];
  // second vertex sits on the pen tip until the short arm has arrived at P1; topology stays M L L
  const b=d>L1?[P1[0]+(P2[0]-P1[0])*w,P1[1]+(P2[1]-P1[1])*w]:a;
  return `M${P0} L${a} L${b}`;};
 const vis=(t,s,r)=>Math.min(1,draw(t,s,r)*30);
 const press=rigid((t,s,r)=>({dy:r?0:F([[0,0]],[[0,0],[.12,1.4],[.3,0]])(t,s)}));
 const parts=posed([
  ...framed('box',rect(5.5,5.5,21,21,4),holes(rect(4.2,4.2,23.6,23.6,5.3),rect(6.8,6.8,18.4,18.4,2.7))),
  ...strokePair('mark',mark,{visibility:vis})],press);
 return {...toggle('checkbox','Checkbox','Box presses, check whips on; toggle retracts the check.',parts,.5,.5,'unchecked','checked',.35,.3),duotoneFill:true};
})();
// Static reusable dash for a product "mixed" rendering on the same box. Not a native exported state.
export const checkboxMixedMark=strokePair('mark','M10.5 16 L21.5 16');

// ---------- RADIO BUTTON (unchanged, accepted) ----------
const radio=(()=>{
 const grow=R(F([[0,0],[.06,0],[.42,4,'smooth']],[[0,0],[.1,0],[.16,.7],[.3,5],[.46,4]]),
              F([[0,4],[.32,0,'smooth']],[[0,4],[.24,0]]));
 const dot=(t,s,r)=>circle(16,16,Math.max(.02,grow(t,s,r)));
 const vis=(t,s,r)=>Math.min(1,grow(t,s,r)*20);
 const parts=[
  ...framed('ring',circle(16,16,9),annulus(16,16,9,2.6)),
  fill('dot',dot,'outline',{visibility:vis}),fill('dot-solid',dot,'solid',{visibility:vis})];
 return {...toggle('radio-button','Radio button','A nucleus appears then the dot bursts to size; toggle empties it.',parts,.5,.5,'unselected','selected',.35,.3),duotoneFill:true};
})();

// ---------- TOGGLE SWITCH: off -> on. Wider track, Solid knob = Outline painted exterior. ----------
// Track 3.5..28.5 x 8.5..23.5 (r7.5). Knob centers 11 / 21, travel 10. Strong prep -0.7, overshoot +0.7.
const toggleSwitch=(()=>{
 const slide=R(F([[0,0],[.42,10,'smooth']],[[0,0],[.09,-.7],[.22,10.7],[.36,10]]),
               F([[0,10],[.42,0,'smooth']],[[0,10],[.09,10.7],[.22,-.7],[.36,0]]));
 const knobPose=rigid((t,s,r)=>({dx:slide(t,s,r)}));
 const track=rect(3.5,8.5,25,15,7.5);
 const parts=[
  fill('track-interior',track,'tint'),
  stroke('track-frame',track),
  fill('track-solid',holes(rect(2.2,7.2,27.6,17.6,8.8),rect(4.8,9.8,22.4,12.4,6.2)),'solid'),
  ...posed([stroke('knob',circle(11,16,3)),fill('knob-solid',circle(11,16,4.3),'solid')],knobPose)];
 return {...toggle('toggle-switch','Toggle switch','Knob pulls back, snaps across the track, and settles; toggle returns it.',parts,.5,.5,'off','on',.5,.5),duotoneFill:true};
})();

// ---------- TEXT FIELD (unchanged, accepted) ----------
const textField=(()=>{
 const type=F([[0,0],[.56,1,'smooth']],[[0,0],[.14,0],[.4,1]]);
 const blink=F([[0,1]],[[0,1],[.05,0],[.12,0],[.17,1]]);
 const textEnd=(t,s)=>9+12*type(t,s)-3.9;
 const parts=[
  ...framed('field',rect(4,10,24,12,3),holes(rect(2.7,8.7,26.6,14.6,4.3),rect(5.3,11.3,21.4,9.4,1.7))),
  ...strokePair('text',(t,s)=>`M9 16 L${Math.max(9,textEnd(t,s))} 16`,{visibility:(t,s)=>Math.min(1,Math.max(0,(textEnd(t,s)-9)*20))}),
  ...shown(posed(tube('caret','M9 14 L9 18'),rigid((t,s)=>({dx:12*type(t,s)}))),(t,s)=>blink(t,s))];
 return {...geometricRearm(make('text-field','Text field','Caret blinks then advances as a line of text is entered.',parts,.6,.5)),duotoneFill:true};
})();

// ---------- NUMBER FIELD: up stepper pressed, glyph 1 morphs in place to 2, result retained ----------
// Field 3.5..21 x 9..23 (r3). Glyph centred (10,16), height ±2.6; one 4-vertex cubic path for both digits.
const numberField=(()=>{
 const ONE=[[-1.5,-1.3],[-1,-1.73],[-.5,-2.17],[0,-2.6],[0,-.87],[0,.87],[0,2.6],[0,2.6],[0,2.6],[0,2.6]];
 const TWO=[[-1.6,-1.2],[-1.2,-2.9],[1.7,-2.9],[1.5,-.9],[1.35,.1],[-1.7,1.7],[-1.7,2.6],[-.57,2.6],[.57,2.6],[1.7,2.6]];
 const glyph=u=>{const p=ONE.map((a,i)=>[10+mix(a[0],TWO[i][0],u),16+mix(a[1],TWO[i][1],u)]);
  return `M${p[0]} C${p[1]} ${p[2]} ${p[3]} C${p[4]} ${p[5]} ${p[6]} C${p[7]} ${p[8]} ${p[9]}`;};
 const morph=F([[0,0],[.12,0],[.5,1,'smooth']],[[0,0],[.1,0],[.24,1]]);            // monotone value change
 const kick=rigid((t,s)=>({dy:F([[0,0]],[[0,0],[.1,0],[.2,-.7],[.34,.2],[.46,0]])(t,s)})); // Strong physical response only
 const press=rigid((t,s)=>({dy:F([[0,0],[.1,-.5,'smooth'],[.32,0,'smooth']],[[0,0],[.07,-1.2],[.24,0]])(t,s)}));
 const parts=[
  ...framed('field',rect(3.5,9,17.5,14,3),holes(rect(2.2,7.7,20.1,16.6,4.3),rect(4.8,10.3,14.9,11.4,1.7))),
  ...posed(strokePair('value',(t,s)=>glyph(morph(t,s))),kick),
  ...posed(tube('step-up','M25.4 14 L26.9 12.4 L28.4 14'),press),
  ...tube('step-down','M25.4 18 L26.9 19.6 L28.4 18')];
 return {...geometricRearm(make('number-field','Number field','Up stepper is pressed; the value changes from 1 to 2 and stays.',parts,.6,.5)),duotoneFill:true};
})();

export const icons=[checkbox,radio,toggleSwitch,textField,numberField];
