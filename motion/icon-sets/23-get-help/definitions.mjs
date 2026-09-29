import {icons as help} from './batches/help.mjs';
// Supervisor constraints test object mechanics independently of authored tracks.
const fixed={
 'support-agent':['shoulders'],
 'support-ticket':['body-frame','line-1','line-2'],
 'live-support':['customer-frame','customer-line'],
};
const rigid={
 'support-agent':['head-frame','band','ear-left','ear-right','boom'],
 'help-center':['outer','inner','grip-n','grip-s','grip-w','grip-e'],
 'support-ticket':['stub-frame'],
 'live-support':['agent-frame'],
 'callback':['handset-frame'],
};
export const icons=help.map((icon,i)=>({...icon,number:382+i,constraints:[
 ...(fixed[icon.name]||[]).map(part=>({type:'fixed',part:icon.name+'-'+part})),
 ...(rigid[icon.name]||[]).map(part=>({type:'rigid',part:icon.name+'-'+part})),
]}));
