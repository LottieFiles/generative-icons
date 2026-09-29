import{accepted}from'./batches/accepted.mjs';
import{capabilities}from'./batches/capabilities.mjs';
import{control}from'./batches/control.mjs';
import{runtime}from'./batches/runtime.mjs';
import{automation}from'./batches/automation.mjs';
const map={...accepted,...capabilities,...control,...runtime,...automation};
const order=['agent-skills','tool-discovery','mcp-tools','browser-agent','terminal-agent','agent-sandbox','agent-permissions','human-input','agent-checkpoint','resume-run','retry-step','execution-trace','parallel-tools','merge-results','background-run','scheduled-agent','event-trigger','context-compaction','agent-artifact','budget-limit'];
export const icons=order.map((name,i)=>({...map[name],number:290+i}));
if(icons.some(i=>!i.parts))throw Error('Missing agent operation definition');
