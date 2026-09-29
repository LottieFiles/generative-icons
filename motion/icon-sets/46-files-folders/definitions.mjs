// Files & folders (46). Drawings and motion live in batches/; this file fixes order, numbers and design constraints.
import {icons as a} from './batches/folders.mjs';
import {icons as b} from './batches/badges.mjs';
import {icons as c} from './batches/types.mjs';
const all=new Map([...a,...b,...c].map(i=>[i.name,i]));
// Sample the existing rigid arrow poses as native path keys: a full-turn layer
// transform can rasterize differently from zero degrees at the loop boundary.
const fileSync=all.get('file-sync');
fileSync.parts=fileSync.parts.map(part=>part.pose?.transform
 ? {...part,pose:(x,y,t,s,r)=>part.pose(x,y,t,s,r)}
 : part);
const ORDER=['new-folder','folder-open','move-file','rename-file','duplicate-file','restore-file','approve-file','lock-file','file-search','file-preview','file-tag','file-star','file-sync','import-file','export-file','image-file','code-file','spreadsheet-file','audio-file','video-file'];
if(ORDER.length!==all.size||ORDER.some(n=>!all.has(n)))throw Error('Files & folders order and batches disagree');
// Fixed housings stay still for every action; rigid pieces keep their internal distances.
const fixed={
 'move-file':['mark-a','mark-a-solid','mark-b','mark-b-solid'],
 'import-file':['sheet','sheet-fold','sheet-interior'],'export-file':['sheet','sheet-fold','sheet-interior'],
 'rename-file':['sheet-page','sheet-fold','sheet-interior','line-2'],
 'restore-file':['sheet-page','sheet-fold','sheet-interior'],'file-sync':['sheet-page','sheet-fold','sheet-interior'],
 'lock-file':['sheet-page','sheet-fold','sheet-interior'],'file-search':['sheet-page','sheet-fold','sheet-interior'],
 'file-preview':['sheet-page','sheet-fold','sheet-interior','lower-lid'],'file-tag':['sheet-page','sheet-fold','sheet-interior'],
 'file-star':['sheet-page','sheet-fold','sheet-interior'],'image-file':['sheet-page','sheet-fold','sheet-interior','peaks'],
 'code-file':['sheet-page','sheet-fold','sheet-interior'],'spreadsheet-file':['sheet-page','sheet-fold','sheet-interior','table','column','row','dividers-solid'],
 'audio-file':['sheet-page','sheet-fold','sheet-interior'],'video-file':['sheet-page','sheet-fold','sheet-interior']};
const rigidParts={
 'new-folder':['body','front-edge'],'move-file':['sheet','sheet-fold'],'duplicate-file':['copy','copy-fold'],
 'import-file':['shaft','head'],'export-file':['shaft','head'],
 'restore-file':['arrow-arc','arrow-head'],'file-sync':['arrow-a-arc','arrow-b-arc'],'lock-file':['body','shackle'],
 'file-search':['lens','handle'],'file-tag':['tag'],'image-file':['sun'],'code-file':['bracket-l','bracket-r'],
 'approve-file':['sheet-page','sheet-fold']};
export const icons=ORDER.map(n=>all.get(n)).map((icon,index)=>({...icon,number:746+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
