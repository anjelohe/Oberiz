// Optional real-client probe. Creates ONLY synthetic media in a fresh profile.
// Never accepts an existing WebUI URL, credentials, profile or torrent hash.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import net from 'node:net';
import { createHash, randomBytes, pbkdf2Sync } from 'node:crypto';
import { spawn, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { setTimeout as delay } from 'node:timers/promises';

const binary=process.argv[2];
if(!binary||!path.isAbsolute(binary))throw new Error('Pass the absolute path to a qBittorrent binary');
const base=path.resolve('backend/backups/storage-simplification-20261015');
await fs.mkdir(base,{recursive:true});
const root=await fs.mkdtemp(path.join(base,'real-qbt-'));
const profile=path.join(root,'profile');
const configuration='oberiz-proof';
const config=path.join(profile,`qBittorrent_${configuration}`,'config');
const media=path.join(root,'synthetic-media');
await fs.mkdir(config,{recursive:true});await fs.mkdir(media);
async function freePort(){const server=net.createServer();await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const port=server.address().port;await new Promise(resolve=>server.close(resolve));return port;}
const port=await freePort();
const torrentingPort=await freePort();
const password=randomBytes(32).toString('base64url');
const salt=randomBytes(16);
const passwordHash=`${salt.toString('base64')}:${pbkdf2Sync(password,salt,100000,64,'sha512').toString('base64')}`;
await fs.writeFile(path.join(config,'qBittorrent.ini'),String.raw`[Application]
FileLogger\Enabled=true
FileLogger\Path=${root.replaceAll('\\','/')}
[BitTorrent]
Session\DHTEnabled=false
Session\LSDEnabled=false
Session\PeXEnabled=false
Session\UseUPnP=false
Session\InterfaceAddress=127.0.0.1
Session\DefaultSavePath=${media.replaceAll('\\','/')}
[WebUI]
Enabled=true
Address=127.0.0.1
Port=${port}
Username=oberiz-proof
Password_PBKDF2="@ByteArray(${passwordHash})"
LocalHostAuth=false
HostHeaderValidation=true
CSRFProtection=true
[Preferences]
WebUI\Enabled=true
WebUI\Address=127.0.0.1
WebUI\Port=${port}
WebUI\LocalHostAuth=false
WebUI\HostHeaderValidation=true
WebUI\CSRFProtection=true
Connection\UPnP=false
WebUI\Username=oberiz-proof
WebUI\Password_PBKDF2="@ByteArray(${passwordHash})"
Downloads\SavePath=${media.replaceAll('\\','/')}
`);
const child=spawn(binary,[`--profile=${profile}`,`--configuration=${configuration}`,`--webui-port=${port}`,`--torrenting-port=${torrentingPort}`,'--no-splash','--confirm-legal-notice'],{windowsHide:true,stdio:'ignore'});
let verifiedProfile=false;
const url=`http://127.0.0.1:${port}`;
async function api(route,fields){const response=await fetch(`${url}/api/v2/${route}`,{method:fields?'POST':'GET',body:fields instanceof FormData?fields:fields?new URLSearchParams(fields):undefined,headers:{Referer:url,Origin:url},signal:AbortSignal.timeout(5000),redirect:'error'});if(!response.ok)throw new Error(`${route}: HTTP ${response.status}`);return response;}
async function until(fn){for(let i=0;i<100;i++){const value=await fn();if(value)return value;await delay(200);}throw new Error('Probe timed out');}
function encode(value){if(Buffer.isBuffer(value))return Buffer.concat([Buffer.from(`${value.length}:`),value]);if(typeof value==='string')return encode(Buffer.from(value));if(typeof value==='number')return Buffer.from(`i${value}e`);return Buffer.concat([Buffer.from('d'),...Object.keys(value).sort().flatMap(key=>[encode(key),encode(value[key])]),Buffer.from('e')]);}
const digest=data=>createHash('sha1').update(data).digest('hex');
try{
  let lastStartupError;
  const prefs=await until(async()=>{try{return await(await api('app/preferences')).json();}catch(error){lastStartupError=error;return false;}}).catch(()=>{throw new Error(`Isolated WebUI did not become ready: ${lastStartupError?.message} (${lastStartupError?.cause?.code}); child exit=${child.exitCode}; proof=${root}`)});
  assert.equal(path.resolve(prefs.save_path),media,'Not the isolated profile: refusing API writes');
  verifiedProfile=true;
  assert.deepEqual(await(await api('torrents/info')).json(),[],'Fresh isolated profile must be empty');
  await api('app/setPreferences',{json:JSON.stringify({dht:false,pex:false,lsd:false,upnp:false,autorun_enabled:false,rss_auto_downloading_enabled:false})});
  const version=await(await api('app/version')).text();
  const payload=Buffer.alloc(8192,0x4f);
  const original='Original.2026.mkv',named='Fixture Film (2026) - 1080p.mkv';
  await fs.writeFile(path.join(media,original),payload);
  const info=encode({length:payload.length,name:original,'piece length':16384,pieces:createHash('sha1').update(payload).digest(),private:1});
  const torrent=Buffer.concat([Buffer.from('d4:info'),info,Buffer.from('e')]);
  const hash=digest(info);
  async function add(bytes){const form=new FormData();form.append('torrents',new Blob([bytes]),'synthetic.torrent');for(const[key,value]of Object.entries({savepath:media,paused:'true',stopped:'true',autoTMM:'false',skip_checking:'false',contentLayout:'Original'}))form.append(key,value);await api('torrents/add',form);await until(async()=>{const rows=await(await api('torrents/info')).json();return rows.find(row=>row.hash===hash);});}
  const files=async()=>await(await api(`torrents/files?hash=${hash}`)).json();
  async function rename(oldPath,newPath){await api('torrents/renameFile',{hash,oldPath,newPath});await until(async()=>(await files())[0]?.name===newPath);}
  async function recheck(){await api('torrents/recheck',{hashes:hash});await until(async()=>{const row=(await(await api('torrents/info')).json()).find(row=>row.hash===hash);return row?.progress===1&&/^(stopped|paused)UP$/.test(row.state);});}
  await add(torrent);await recheck();
  await rename(original,named);
  assert.equal(digest(await fs.readFile(path.join(media,named))),digest(payload));
  const exported=Buffer.from(await(await api(`torrents/export?hash=${hash}`)).arrayBuffer());
  assert.ok(exported.includes(info),'Export must preserve original info dictionary and hash');
  await api('torrents/delete',{hashes:hash,deleteFiles:'false'});
  await until(async()=>(await(await api('torrents/info')).json()).length===0);
  assert.equal(digest(await fs.readFile(path.join(media,named))),digest(payload));
  await add(exported);
  const current=(await files())[0].name;
  await rename(current,named);await recheck();
  assert.equal(digest(await fs.readFile(path.join(media,named))),digest(payload));
  const occupied='Occupied.mkv';const sentinel=Buffer.from('Do not overwrite synthetic sentinel');
  await fs.writeFile(path.join(media,occupied),sentinel);
  // qBittorrent itself can overwrite an unrelated target. Exercise Oberiz's
  // actual Rust guard, not a duplicated JavaScript approximation of it.
  const proof=await promisify(execFile)('cargo',['test','--manifest-path','backend/Cargo.toml','importer::remote::tests::real_qbt_conflict_guard','--','--ignored','--exact'],{windowsHide:true,timeout:120000,env:{...process.env,OBERIZ_SYNTHETIC_QB_PORT:String(port),OBERIZ_SYNTHETIC_QB_DIRECTORY:media}});
  assert.ok(proof.stdout.includes('1 passed'));
  const preserved=Buffer.compare(await fs.readFile(path.join(media,occupied)),sentinel)===0;
  assert.ok(preserved,'Oberiz must preserve the unrelated physical target');
  assert.equal(digest(await fs.readFile(path.join(media,named))),digest(payload));
  const result={version,renaming:true,exportPreservesHash:true,recoveryStoppedAndRechecked:true,oberizPhysicalConflictGuard:true,syntheticOnly:true};
  await fs.writeFile(path.join(root,'result.json'),JSON.stringify(result,null,2));
  console.log(JSON.stringify(result));
  console.log(`Isolated proof retained at ${root}`);
}finally{
  if(verifiedProfile){try{await api('app/shutdown',{});}catch{}}
  await delay(500);
  if(child.exitCode===null)child.kill();
}
