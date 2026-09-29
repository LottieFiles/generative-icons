// Add source manifests to ../catalog.json. All icons use this one preview.
export function normalizeSet(set, metadataUrl, offset=0) {
  const normalize = item => ({direction:'forward',stateful:false,startState:'rest',endState:'rest',...item});
  return set.icons.map((icon,index) => ({...normalize(icon),number:String(offset+index+1).padStart(2,'0'),assetBase:new URL('.',metadataUrl).href,
    motions:icon.motions.map(m=>({...normalize(m),...(m.reverse?{reverse:normalize(m.reverse)}:{})}))}));
}
export async function loadCatalog(catalogUrl, request=fetch) {
  const get = async url => {const response=await request(url,{cache:'no-store'});if(!response.ok)throw new Error(`Could not load ${url}`);return response.json();};
  const catalog=await get(catalogUrl),icons=[],sets=new Map();
  for(const entry of catalog.sets){const url=new URL(entry,catalogUrl).href;const normalized=normalizeSet(await get(url),url,icons.length);sets.set(entry,normalized);icons.push(...normalized);}
  if(new Set(icons.map(i=>i.name)).size!==icons.length)throw new Error('Duplicate icon name in catalog');
  const assigned=new Set();
  const categories=catalog.categories.map(category=>({ ...category, icons:category.sets.flatMap(entry=>{
    if(!sets.has(entry)||assigned.has(entry))throw new Error('Invalid or repeated category set: '+entry);
    assigned.add(entry);return sets.get(entry);
  })}));
  if(assigned.size!==sets.size)throw new Error('Every source set needs a browse category');
  return {...catalog,icons,categories};
}
export const assetUrl=(icon,meta=icon)=>new URL(`${meta.name}.lottie?v=${meta.sha256}`,icon.assetBase).href;
