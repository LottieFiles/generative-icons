import test from 'node:test';
import assert from 'node:assert/strict';
import {flow,drive,track} from './direction.mjs';
test('flow passes through authored keys and stays inside each interval',()=>{
 for(const keys of [ [[0,0],[.08,-2,'launch'],[.3,12,'hit'],[.39,9],[.52,10]], [[0,0],[.05,.1],[.14,.3,'pass'],[.4,.8],[.53,1]], [[0,2],[.2,2],[.4,-3],[.7,-3],[1,0]] ]){
  for(const [t,v] of keys)assert.equal(flow(keys,t),v);
  for(let k=0;k<keys.length-1;k++)for(let j=0;j<=100;j++){
   const a=keys[k],b=keys[k+1],v=flow(keys,a[0]+(b[0]-a[0])*j/100);
   assert.ok(v>=Math.min(a[1],b[1])-1e-10&&v<=Math.max(a[1],b[1])+1e-10);
  }
  assert.equal(flow(keys,9),keys.at(-1)[1]);
 }
});
test('same-direction waypoints carry the same nonzero velocity on both sides',()=>{
 const keys=[[0,0],[.05,.1],[.14,.3,'pass'],[.4,.8],[.53,1]],h=1e-7;
 for(const [t]of keys.slice(1,-1)){
  const left=(flow(keys,t)-flow(keys,t-h))/h,right=(flow(keys,t+h)-flow(keys,t))/h;
  assert.ok(left>.1);assert.ok(Math.abs(left-right)<.0001);
 }
});
test('explicit holds and endpoints are exact; Light remains on original easing',()=>{
 const keys=[[0,0],[.1,1],[.2,1],[.4,0]];
 for(let t=.1;t<=.2;t+=.001)assert.equal(flow(keys,t),1);
 for(let t=0;t<1;t+=.001)assert.equal(drive(t,false,keys,[[0,7],[.8,2]]),track(keys,t));
 assert.equal(flow(keys,2.4),0);
});
