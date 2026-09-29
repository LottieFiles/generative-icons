// Chat & messaging: exactly twenty distinct meanings. Set-owned drawing and motion.
import {icons as status} from './batches/status.mjs';
import {icons as conversation} from './batches/conversation.mjs';
import {icons as tools} from './batches/tools.mjs';
const order=['composing-message','message-sent','message-delivered','message-read','message-reaction','quoted-message','forward-message','pin-message','conversation-history','mute-chat','group-chat','voice-note','sticker','gif-picker','edit-message','delete-message','unread-divider','archive-chat','disappearing-message','chat-search'];
const all=new Map([...status,...conversation,...tools].map(i=>[i.name,i]));
if(all.size!==20||order.some(n=>!all.has(n)))throw Error('Chat set must have all twenty meanings');
export const icons=order.map((n,index)=>({...all.get(n),number:726+index,constraints:all.get(n).parts.filter(p=>p.pose?.transform&&typeof p.d==='string').map(p=>({type:'rigid',part:p.name}))}));
