import {icon,bubble,surface,tube,fill,bothFill,circle,rect,polygon,rigid,posed,F,gesture,state,appear,track} from './helpers.mjs';
// A sticker peels from its fixed top-left region. The fold joins the changing diagonal edge.
const peel=F([[0,0],[.22,1],[.7,0,'smooth']],[[0,0],[.1,-.5],[.32,2],[.5,-.4],[.82,0]]);
const edge=(t,s)=>21-peel(t,s);
const sticker=icon('sticker','Sticker',
 'Pick sticker → its lower corner peels back → the diagonal fold follows the edge → the sticker lies flat again. Light: 1-unit peel. Strong: 2-unit peel with a short wind-up and settling response.',
 [...surface('paper',(t,s)=>{const e=edge(t,s);return `M8 5 L24 5 C25.7 5 27 6.3 27 8 L27 ${e} L${e} 27 L8 27 C6.3 27 5 25.7 5 24 L5 8 C5 6.3 6.3 5 8 5 Z`;}),...tube('fold',(t,s)=>{const e=edge(t,s);return `M${e} 27 L${e} ${e+2} C${e} ${e+.9} ${e+.9} ${e} ${e+2} ${e} L27 ${e}`;}),...bothFill('eye-a',circle(11,12,1)),...bothFill('eye-b',circle(19,12,1)),...tube('smile','M10 16 C12 18.5 16 18.5 18 16')],.8,.86);
const gifY=gesture(-1.1,-2.2);
const gif=icon('gif-picker','GIF picker',
 'Browse GIFs → the preview tile lifts from its stacked tray → the GIF label rides with it → the selected tile settles. Light: 1.1-unit lift. Strong: 2.2-unit lift with a brief preparation.',
 [...tube('stack','M7 26 L25 26'),...posed([...surface('tile',rect(4.5,7,23,14,3)),...tube('g','M12 10.5 L7.5 10.5 L7.5 17.5 L12 17.5 L12 14.5 L10.5 14.5'),...tube('i','M16 10.5 L16 17.5'),...tube('f','M20 17 L20 11.5 L24 11.5'),...tube('f-bar','M20 14 L23 14')],rigid((t,s)=>({dy:gifY(t,s),pivot:[16,21]})))],.8,.8);
// The pen tip and the written baseline share x; connected paint stays opaque.
const ink=(t,s,r)=>state(t,s,r);
const edit=icon('edit-message','Edit message',
 'Edit → the pen moves along the message baseline → ink extends exactly to its tip → the edited line is retained. Light: a 7.5-unit writing stroke. Strong: the same meaningful stroke arrives in .30 s after preparation. Reverse erases back along the baseline.',
 [...surface('bubble',bubble(5,5,22,17)),...tube('ink',(t,s,r)=>`M10 18 L${10+7.5*ink(t,s,r)} 18`,{visibility:(t,s,r)=>ink(t,s,r)>.01?1:0}),...posed([...bothFill('pen','M10 18 L10.6 14.5 L12.7 15.7 Z'),...bothFill('pen-shaft',polygon([[11.5,13],[14,8.7],[16.2,10],[13.7,14.3]],.35))],rigid((t,s,r)=>({dx:7.5*ink(t,s,r),pivot:[10,18]})))],.8,.72,['Original','Edited']);
const deletion=icon('delete-message','Delete message',
 'Delete → two message lines retract into a removal mark → the bubble becomes a deleted-message placeholder → deletion is retained. Light: gradual replacement. Strong: fast leftward removal. Reverse restores the message content.',
 [...surface('bubble',bubble(5,5,22,18)),...['M10 11 L22 11','M10 17 L18 17'].flatMap((d,i)=>tube('text-'+i,d,{pose:rigid((t,s,r)=>({dx:-(s?2:1)*Math.sin(Math.PI*state(t,s,r)),pivot:[16,14]})),visibility:(t,s,r)=>1-state(t,s,r)})),...appear(tube('removed','M11 14 L21 14'),0,2,[16,14])],.8,.72,['Message','Deleted']);
const sep=gesture(1.3,3);
const unread=icon('unread-divider','Unread divider',
 'Jump to unread → the unread separator moves down between two message rows → its centre marker leads → the boundary rests. Light: 1.3-unit travel. Strong: 3-unit travel with a short rebound.',
 [...surface('upper-message',rect(7,5,18,4,2)),...surface('lower-message',rect(7,24,18,4,2)),...posed([...tube('divider-l','M4.5 16 L11 16'),...bothFill('unread-dot',circle(16,16,1.6)),...tube('divider-r','M21 16 L27.5 16')],rigid((t,s)=>({dy:sep(t,s),pivot:[16,16]})))],.8,.8);
const archive=icon('archive-chat','Archive chat',
 'Archive → the message slides into the storage tray → the chat remains visible as the stored record → archived is retained. Light: 7 units downward. Strong: a decisive 7-unit delivery with an accelerated middle stroke. Reverse lifts the record out.',
 [...tube('tray','M5 22 L5 26 C5 27.1 5.9 28 7 28 L25 28 C26.1 28 27 27.1 27 26 L27 22'),...posed(surface('message',bubble(7,5,18,8)),rigid((t,s,r)=>{const u=state(t,s,r);return {dy:7*u+(s?Math.sin(Math.PI*u):0),pivot:[16,12]};}))],.8,.72,['Inbox','Archived']);
// Timed-message control: hourglass inside a bubble; the sand settles and resets as a control gesture.
const sand=F([[0,0],[.5,1],[.9,0,'smooth']],[[0,0],[.12,0],[.38,1],[.58,1],[.95,0]]);
const timed=icon('disappearing-message','Disappearing message',
 'Inspect expiry → sand runs through the message timer → the lower chamber fills → the expiry control returns ready. Light: a quiet transfer. Strong: the timer turns 12 degrees while sand transfers faster. This previews the duration setting, not message deletion.',
 [...surface('bubble',bubble(5,5,22,19)),...posed([...tube('timer','M12 9 L20 9 C20 13 18 13.5 16 15 C14 16.5 12 17 12 21 L20 21 C20 17 18 16.5 16 15 C14 13.5 12 13 12 9 Z'),...bothFill('sand',(t,s)=>{const u=sand(t,s);return polygon([[13.5,19.5],[18.5,19.5],[16,19.2-2.7*u]],.25);})],rigid((t,s)=>({angle:s?12*Math.sin(Math.PI*sand(t,s)):0,pivot:[16,15]})))],.92,.98);
const seek=F([[0,0],[.2,-1.3],[.45,1.3],[.8,0,'smooth']],[[0,0],[.18,-2.5],[.42,2.5],[.62,-.4],[.9,0]]);
const search=icon('chat-search','Chat search',
 'Search transcript → the lens sweeps beneath the message row → its handle follows rigidly → the lens returns to its centred result. Light: a 2.6-unit scan. Strong: a 5-unit scan with a bounded response.',
 [...surface('bubble',bubble(5,5,22,8)),...tube('text','M10 9 L21 9'),...posed([...surface('lens',circle(18,23,4)),...tube('handle','M20.8 25.8 L23.6 28.6')],rigid((t,s)=>({dx:seek(t,s),pivot:[18,23]})))],.82,.92);
export const icons=[sticker,gif,edit,deletion,unread,archive,timed,search];
