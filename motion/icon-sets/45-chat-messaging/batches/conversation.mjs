import {icon,bubble,surface,tube,bothFill,fill,circle,rect,rigid,posed,F,gesture,track,polygon} from './helpers.mjs';
// Compose text with a caret; unlike the existing Message icon, there are no typing dots.
const caret=F([[0,0],[.12,0],[.44,3],[.82,0,'smooth']],[[0,0],[.08,-.3],[.34,5],[.5,5],[.86,0]]);
const composing=icon('composing-message','Composing message',
 'Compose → the insertion caret advances along a text row → it returns to the insertion point → the draft is ready. Light: a 3-unit sweep. Strong: 5 units with a .3-unit preparation and brisk main stroke.',
 [...surface('bubble',bubble(5,6,22,17)),...tube('text','M10 12 L14 12'),...tube('lower-text','M10 18 L16 18'),...posed(tube('caret','M18 10 L18 14'),rigid((t,s)=>({dx:caret(t,s),pivot:[18,12]})))],.86,.86);
const quoteX=gesture(1.4,3.2);
const quoted=icon('quoted-message','Quoted message',
 'Quote selection → paired quote marks indent toward the quoted text → the inset line follows → quote rests in its source bubble. Light: 1.4 units. Strong: a 3.2-unit indent and one bounded response.',
 [...surface('bubble',bubble(5,5,22,19)),...posed([...bothFill('quote-left',rect(10,10,2.6,3,1)),...tube('quote-left-tail','M11.6 12 L10 15'),...bothFill('quote-right',rect(16,10,2.6,3,1)),...tube('quote-right-tail','M17.6 12 L16 15')],rigid((t,s)=>({dx:quoteX(t,s),pivot:[15,12]}))),...tube('quotation','M10 20 L21 20')],.8,.8);
const fly=F([[0,0],[.1,0],[.32,2],[.72,0,'smooth']],[[0,0],[.1,-1],[.3,4],[.49,3.2],[.78,0]]);
const forward=icon('forward-message','Forward message',
 'Forward command → a right arrow travels away from the source bubble → the route stretches → it rests ready for another forward. Light: 2 units right. Strong: one-unit pullback and a 4-unit drive.',
 [...surface('bubble',bubble(5,5,18,11)),...tube('text','M10 10.5 L18 10.5'),...posed([...tube('arrow-shaft','M10 24 L23 24'),...tube('arrow-head','M19 20 L23 24 L19 28')],rigid((t,s)=>({dx:fly(t,s),pivot:[23,24]})))],.8,.8);
const scroll=F([[0,0],[.12,0],[.4,-2],[.76,0,'smooth']],[[0,0],[.1,.5],[.32,-4],[.52,-3],[.86,0]]);
const history=icon('conversation-history','Conversation history',
 'Browse earlier conversation → stacked transcript rows move upward beneath a fixed top bubble → the history cue stays in place → the transcript returns. Light: a 2-unit scroll. Strong: a 4-unit scroll with preparation.',
 [...surface('top-message',bubble(9,5,18,8)),...tube('history-rail','M5 9 L5 25 C5 26.1 5.9 27 7 27 L8.5 27'),...posed([...tube('older-row','M13 21 L26 21'),...tube('oldest-row','M13 27 L23 27')],rigid((t,s)=>({dy:scroll(t,s),pivot:[19,24]})))],.86,.9);
const groupA=gesture(-1.1,-2.4),groupB=F([[0,0],[.12,0],[.3,-1],[.72,0,'smooth']],[[0,0],[.12,.3],[.36,-2.4],[.55,.3],[.84,0]]);
const group=icon('group-chat','Group chat',
 'Members speak → two participant portraits respond in turn under a common conversation → the shoulders follow their own heads → the group settles. Light: 1.1-unit nods. Strong: 2.4-unit staggered lifts.',
 [...surface('bubble',bubble(5,4.5,22,8)),...posed([...bothFill('member-a',circle(9,21,2.1)),...tube('shoulders-a','M4.8 28 C4.8 23.8 13.2 23.8 13.2 28')],rigid((t,s)=>({dy:groupA(t,s),pivot:[9,27]}))),...posed([...bothFill('member-b',circle(23,21,2.1)),...tube('shoulders-b','M18.8 28 C18.8 23.8 27.2 23.8 27.2 28')],rigid((t,s)=>({dy:groupB(t,s),pivot:[23,27]})))],.8,.86);
const voice=icon('voice-note','Voice note',
 'Play recorded note → three waveform bars articulate around a fixed baseline → the recorded clip remains in its pill → playback rests. Light: shallow staggered amplitude. Strong: a larger travelling waveform. This is a saved clip, distinct from live Voice chat.',
 [...surface('clip',rect(4,8,24,16,5)),...bothFill('play',polygon([[8,12],[8,20],[12.5,16]],.6)),...[15.5,19.5,23.5].flatMap((x,i)=>tube('wave-'+i,(t,s)=>{const env=Math.sin(Math.PI*Math.min(1,t/(s?.88:.8)))**2;const h=([1.8,3.2,2.1][i])+(s?1:.45)*env*Math.sin(t*15-i);return `M${x} ${16-h} L${x} ${16+h}`;}))],.82,.9);
export const icons=[composing,quoted,forward,history,group,voice];
