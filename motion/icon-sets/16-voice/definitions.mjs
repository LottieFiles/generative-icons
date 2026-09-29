import{icons as indicators}from'./batches/01.mjs';
import{icons as controls}from'./batches/02.mjs';
export const icons=[...indicators,...controls].map((icon,i)=>({...icon,number:240+i,revision:'voice-claude-reviewed-v1'}));
if(icons.length!==10||new Set(icons.map(i=>i.name)).size!==10)throw Error('Voice scope must contain ten distinct icons');
