// Filesystem isolation for an explicitly owned disposable process only.
// This is harness instrumentation, NOT a read-only probe or a realm hook.
'use strict';
const root='C:\\Users\\arian\\.cache\\tfm2-rea\\disposable-game';
const expected=(root+'\\TeamfightManager2.exe').toLowerCase();
if(Process.mainModule.path.toLowerCase()!==expected)throw Error('Refusing non-disposable target');
const roaming=root+'\\appdata\\Roaming',local=root+'\\appdata\\Local';
const k32=Process.getModuleByName('kernel32.dll');
const setEnv=new NativeFunction(k32.getExportByName('SetEnvironmentVariableW'),'bool',['pointer','pointer']);
for(const [key,value]of Object.entries({APPDATA:roaming,LOCALAPPDATA:local,SteamAppId:'3009300',SteamGameId:'3009300'})){
 if(!setEnv(Memory.allocUtf16String(key),Memory.allocUtf16String(value)))throw Error('Cannot isolate environment');
}
const setDirectory=new NativeFunction(k32.getExportByName('SetCurrentDirectoryW'),'bool',['pointer']);
if(!setDirectory(Memory.allocUtf16String(root)))throw Error('Cannot set disposable working directory');
let redirects=0;const installed=new Set();
function installShell(module){
 if(module.name.toLowerCase()!=='shell32.dll'||installed.has(module.base.toString()))return;
 installed.add(module.base.toString());
 const ole=Process.getModuleByName('ole32.dll');
 const alloc=new NativeFunction(ole.getExportByName('CoTaskMemAlloc'),'pointer',['size_t']);
 const free=new NativeFunction(ole.getExportByName('CoTaskMemFree'),'void',['pointer']);
 const folders={'db85b63ef965f64ca03ae3ef65729f3d':roaming,'8527b3f1ba6fcf4f9d557b8e7f157091':local,
  'a4a120a58017664fbd18167343c5af16':local+'\\Low'};
 Interceptor.attach(module.getExportByName('SHGetKnownFolderPath'),{
  onEnter(args){
   const bytes=new Uint8Array(args[0].readByteArray(16));
   this.path=folders[Array.from(bytes,b=>b.toString(16).padStart(2,'0')).join('')];this.out=args[3];
  },
  onLeave(result){
   if(!this.path||result.toInt32()<0)return;
   const buffer=alloc((this.path.length+1)*2);
   if(buffer.isNull()){const original=this.out.readPointer();this.out.writePointer(ptr(0));free(original);result.replace(0x8007000e);send({kind:'fatal-isolation-allocation-failure'});return;}
   buffer.writeUtf16String(this.path);const original=this.out.readPointer();this.out.writePointer(buffer);free(original);
   redirects++;send({kind:'known-folder-redirect',count:redirects});
  }
 });
 Interceptor.attach(module.getExportByName('SHGetFolderPathW'),{
  onEnter(args){const id=args[1].toUInt32()&0x3fff;this.path=id===0x1a?roaming:id===0x1c?local:null;this.out=args[4];},
  onLeave(result){if(this.path&&result.toInt32()>=0){this.out.writeUtf16String(this.path);redirects++;send({kind:'legacy-folder-redirect',count:redirects});}}
 });
 send({kind:'shell-isolation-ready'});
}
Process.attachModuleObserver({onAdded:installShell});
// Guard direct accesses too; the game is known to use Windows known-folder APIs,
// which ignore APPDATA alone. This never redirects another process.
const real='c:\\users\\arian\\appdata\\roaming\\teamsamoyed\\teamfightmanager2';
Interceptor.attach(k32.getExportByName('CreateFileW'),{
 onEnter(args){
  const path=args[0].readUtf16String();if(!path)return;
  if(path.toLowerCase().startsWith(real)){
   this.redirect=Memory.allocUtf16String(roaming+'\\TeamSamoyed\\TeamfightManager2'+path.slice(real.length));args[0]=this.redirect;
   send({kind:'direct-career-path-redirect'});
  }
 }
});
send({kind:'disposable-filesystem-ready',gameplayHooksInstalled:false});
