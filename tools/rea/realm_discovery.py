"""Read-only PE/SDK anchor recovery and instruction-aware realm discovery.

Requires iced-x86==1.21.0. Reads the executable and, with --pid, selected module
bytes through QUERY_INFORMATION | VM_READ. Never executes target functions,
changes protection, installs a hook, scans career data, or writes game files.
"""
from __future__ import annotations
import argparse,bisect,hashlib,json,re,struct,sys
from collections import deque
from dataclasses import dataclass
from pathlib import Path
from iced_x86 import Decoder,Formatter,FormatterSyntax,FlowControl,Register

def sha(data):return hashlib.sha256(data).hexdigest()
@dataclass(frozen=True)
class Section:
    name:str;virtual_size:int;rva:int;raw_size:int;offset:int;flags:int

class PE:
    def __init__(self,path):
        self.path=Path(path).resolve();self.data=self.path.read_bytes();b=self.data
        if b[:2]!=b'MZ':raise ValueError('Not PE')
        pe=struct.unpack_from('<I',b,60)[0]
        if b[pe:pe+4]!=b'PE\0\0':raise ValueError('Invalid PE signature')
        machine,count,self.timestamp=struct.unpack_from('<HHI',b,pe+4)
        optional=pe+24
        if machine!=0x8664 or struct.unpack_from('<H',b,optional)[0]!=0x20b:raise ValueError('Requires x64 PE32+')
        self.base=struct.unpack_from('<Q',b,optional+24)[0];self.image_size=struct.unpack_from('<I',b,optional+56)[0]
        sz=struct.unpack_from('<H',b,pe+20)[0];self.sections=[]
        for i in range(count):
            o=optional+sz+i*40;name=b[o:o+8].split(b'\0')[0].decode('ascii');vs,rv,rs,fo=struct.unpack_from('<IIII',b,o+8)
            if fo+rs>len(b):raise ValueError('Section exceeds file')
            self.sections.append(Section(name,vs,rv,rs,fo,struct.unpack_from('<I',b,o+36)[0]))
        self.unwind=[]
        for s in self.sections:
            if s.name=='.pdata':
                for o in range(s.offset,s.offset+s.virtual_size-11,12):
                    start,end,info=struct.unpack_from('<III',b,o)
                    if 0<start<end<=self.image_size:self.unwind.append((start,end,info))
        self.unwind.sort();self.starts=[x[0]for x in self.unwind]
    def section(self,name):return next(s for s in self.sections if s.name==name)
    def offset(self,rva):
        for s in self.sections:
            delta=rva-s.rva
            if 0<=delta<s.raw_size:return s.offset+delta
        raise ValueError('RVA has no file-backed bytes: '+hex(rva))
    def bytes(self,rva,length):
        s=next(s for s in self.sections if s.rva<=rva<s.rva+s.raw_size)
        length=min(length,s.raw_size-(rva-s.rva));o=self.offset(rva);return self.data[o:o+length]
    def code(self,va):return any(s.flags&0x20000000 and self.base+s.rva<=va<self.base+s.rva+s.virtual_size for s in self.sections)
    def extent(self,rva):
        i=bisect.bisect_right(self.starts,rva)-1
        # A missing leaf entry must never inherit the previous function's bounds.
        if i>=0 and self.unwind[i][0]<=rva<self.unwind[i][1]:return self.unwind[i]
        return None
    def identity(self):
        s=self.section('.text');raw=self.data[s.offset:s.offset+s.raw_size]
        return {'path':str(self.path),'format':'PE32+','architecture':'x86_64','bytes':len(self.data),
            'sha256':sha(self.data),'peTimestamp':hex(self.timestamp),'sizeOfImage':self.image_size,
            'preferredImageBase':hex(self.base),'text':{'fileOffset':s.offset,'rva':hex(s.rva),
                'virtualSize':s.virtual_size,'rawSize':s.raw_size,
                'virtualSizeFileBytesSha256':sha(raw[:s.virtual_size]),'rawSizeFileBytesSha256':sha(raw)}}

def sdk_fields(source,typename='SimVtableV1'):
    text=Path(source).read_text(encoding='utf8');start=text.index('pub struct '+typename)
    end=text.index('\n}',start);names=re.findall(r'^    pub ([a-z0-9_]+):',text[start:end],re.M)
    if not names or names[0]!='size':raise ValueError('Unexpected size-prefixed table')
    return names

def find_table(pe,names):
    s=pe.section('.rdata');size=len(names)*8;magic=struct.pack('<Q',size);found=[];start=s.offset
    while True:
        start=pe.data.find(magic,start,s.offset+s.raw_size)
        if start<0:break
        if start%8==0 and start+size<=s.offset+s.raw_size:
            pointers=struct.unpack_from('<'+str(len(names)-1)+'Q',pe.data,start+8)
            if all(pe.code(x)for x in pointers):found.append((s.rva+start-s.offset,pointers))
        start+=1
    if len(found)!=1:raise ValueError('SDK table not uniquely established: '+str(len(found)))
    rva,pointers=found[0]
    return {'rva':hex(rva),'fileOffset':pe.offset(rva),'size':size,
        'classification':'SDK-layout anchor; execution is not established by table membership',
        'entries':[{'name':name,'slot':i,'rva':hex(pointers[i-1]-pe.base)}for i,name in enumerate(names)if i]}

def decode(pe,rva,limit=4096):
    extent=pe.extent(rva);known=extent is not None and extent[0]==rva
    stop=min(extent[1],rva+limit)if known else rva+min(limit,512)
    fmt=Formatter(FormatterSyntax.INTEL);todo=deque([rva]);seen=set();instructions={};edges=[];unknowns=[]
    while todo:
        cursor=todo.popleft()
        if cursor in seen or not(rva<=cursor<stop):continue
        decoder=Decoder(64,pe.bytes(cursor,stop-cursor),ip=pe.base+cursor)
        for inst in decoder:
            address=inst.ip-pe.base
            if address in seen:break
            seen.add(address)
            if inst.is_invalid:unknowns.append('Invalid instruction at '+hex(address));break
            row={'rva':hex(address),'bytes':pe.bytes(address,inst.len).hex(),'assembly':fmt.format(inst)}
            instructions[address]=row;flow=inst.flow_control
            if flow in[FlowControl.CALL,FlowControl.UNCONDITIONAL_BRANCH,FlowControl.CONDITIONAL_BRANCH]:
                target=inst.near_branch_target-pe.base
                kind='direct-call'if flow==FlowControl.CALL else'conditional-branch'if flow==FlowControl.CONDITIONAL_BRANCH else'jump'
                edges.append({'from':hex(address),'kind':kind,'targetRva':hex(target),'insideWindow':rva<=target<stop})
                if flow!=FlowControl.CALL and rva<=target<stop:todo.append(target)
            if flow in[FlowControl.INDIRECT_CALL,FlowControl.INDIRECT_BRANCH]:
                edge={'from':hex(address),'kind':'indirect-call'if flow==FlowControl.INDIRECT_CALL else'indirect-jump',
                    'expression':fmt.format(inst),'resolved':False}
                if inst.memory_base!=Register.NONE:
                    edge.update(memoryBase=inst.memory_base,displacement=inst.memory_displacement)
                edges.append(edge)
            if flow in[FlowControl.RETURN,FlowControl.UNCONDITIONAL_BRANCH,FlowControl.INDIRECT_BRANCH,FlowControl.INTERRUPT,FlowControl.EXCEPTION]:break
    if not known:unknowns.append('No exact unwind entry; decoding is a bounded reachable window, not complete function ownership')
    if known and extent[1]>stop:unknowns.append('Function exceeds decode budget')
    if any(e['kind'].startswith('indirect')for e in edges):unknowns.append('Indirect destinations require receiver/vtable evidence')
    return {'rva':hex(rva),'unwindRange':{'start':hex(extent[0]),'endExclusive':hex(extent[1])}if known else None,
        'classification':'decoded static candidate','runtimeExecutionObserved':False,'hookValidated':False,
        'signature':pe.bytes(rva,min(32,stop-rva)).hex(),'instructions':[instructions[x]for x in sorted(instructions)],
        'edges':edges,'unknowns':unknowns}

def internal_tables(pe,minimum_slots=77):
    s=pe.section('.rdata');found=[]
    for o in range(s.offset,s.offset+s.raw_size-(minimum_slots+3)*8,8):
        drop,size,align=struct.unpack_from('<QQQ',pe.data,o)
        if (drop==0 or pe.code(drop))and 0<size<0x400000 and align in[1,2,4,8,16,32,64,128]and size%align==0:
            pointers=struct.unpack_from('<'+str(minimum_slots)+'Q',pe.data,o+24)
            if all(pe.code(p)for p in pointers):
                found.append({'rva':hex(s.rva+o-s.offset),'objectSize':size,'alignment':align,
                    'classification':'unbound Rust trait-table candidate; receiver not observed',
                    'entries':[{'vtableOffset':hex((i+3)*8),'targetRva':hex(p-pe.base)}for i,p in enumerate(pointers)]})
    return found

def live_read(pe,table,pid):
    if sys.platform!='win32':raise RuntimeError('Live inspection requires Windows')
    import ctypes as c,ctypes.wintypes as w
    class Module(c.Structure):
        _fields_=[('size',w.DWORD),('moduleId',w.DWORD),('processId',w.DWORD),('globalUsage',w.DWORD),
            ('processUsage',w.DWORD),('base',c.c_void_p),('imageSize',w.DWORD),('module',w.HMODULE),
            ('name',w.WCHAR*256),('path',w.WCHAR*260)]
    k=c.WinDLL('kernel32',use_last_error=True)
    k.CreateToolhelp32Snapshot.argtypes=[w.DWORD,w.DWORD];k.CreateToolhelp32Snapshot.restype=w.HANDLE
    k.Module32FirstW.argtypes=[w.HANDLE,c.POINTER(Module)];k.Module32NextW.argtypes=[w.HANDLE,c.POINTER(Module)]
    k.CloseHandle.argtypes=[w.HANDLE];k.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD];k.OpenProcess.restype=w.HANDLE
    k.ReadProcessMemory.argtypes=[w.HANDLE,c.c_void_p,c.c_void_p,c.c_size_t,c.POINTER(c.c_size_t)];k.ReadProcessMemory.restype=w.BOOL
    sn=k.CreateToolhelp32Snapshot(0x18,pid)
    if sn==c.c_void_p(-1).value:raise c.WinError(c.get_last_error())
    modules=[]
    try:
        e=Module();e.size=c.sizeof(e);ok=k.Module32FirstW(sn,c.byref(e))
        while ok:
            if Path(e.path).resolve()==pe.path:modules.append((e.base,e.imageSize))
            ok=k.Module32NextW(sn,c.byref(e))
    finally:k.CloseHandle(sn)
    if len(modules)!=1:raise ValueError('Exact selected executable is not uniquely loaded in PID')
    base,image_size=modules[0];handle=k.OpenProcess(0x410,False,pid)
    if not handle:raise c.WinError(c.get_last_error())
    def read(address,size):
        b=c.create_string_buffer(size);count=c.c_size_t()
        if not k.ReadProcessMemory(handle,c.c_void_p(address),b,size,c.byref(count))or count.value!=size:raise c.WinError(c.get_last_error())
        return b.raw
    try:
        raw=read(base+int(table['rva'],16),table['size']);pointers=struct.unpack('<'+str(table['size']//8)+'Q',raw)
        checks=[]
        for entry in table['entries']:
            rva=int(entry['rva'],16);length=min(32,(pe.extent(rva)or(rva,rva+32,0))[1]-rva)
            checks.append({'name':entry['name'],'rva':entry['rva'],
                'pointerMatches':pointers[entry['slot']]==base+rva,
                'codeBytesMatch':read(base+rva,length)==pe.bytes(rva,length),'bytesCompared':length,
                'executionObserved':False})
        if pointers[0]!=table['size']or not all(x['pointerMatches']and x['codeBytesMatch']for x in checks):raise ValueError('Live SDK anchors differ from the selected file')
        return {'pid':pid,'loadedImageSize':image_size,'processAccess':'QUERY_INFORMATION | VM_READ (0x410)',
            'injection':False,'processPaused':False,'memoryWrites':False,'scannedHeapOrCareerData':False,
            'sdkTableMatches':True,'checks':checks,'limitation':'Matching loaded bytes and pointers does not establish executed calls or hook safety'}
    finally:k.CloseHandle(handle)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--exe',type=Path,required=True)
    p.add_argument('--sdk',type=Path,required=True);p.add_argument('--out',type=Path,required=True)
    p.add_argument('--pid',type=int);p.add_argument('--depth',type=int,default=2);p.add_argument('--max-functions',type=int,default=192)
    p.add_argument('--seed-rva',action='append',default=[],type=lambda x:int(x,0),
        help='Additional caller-asserted entry RVA; this is not evidence of function ownership')
    p.add_argument('--seed-byte-budget',type=int,default=4096,choices=range(32,65537),metavar='32..65536')
    a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True);pe=PE(a.exe);source=a.sdk/'src/host.rs'
    fields=sdk_fields(source);table=find_table(pe,fields);candidates=internal_tables(pe)
    probe={'version':1,'exe':pe.identity(),'sdk':{'hostSourceSha256':sha(source.read_bytes()),'simFields':fields},
        'simVtable':table,'internalVtableCandidates':candidates,'readOnly':True,
        'live':live_read(pe,table,a.pid)if a.pid else None}
    (a.out/'realm_probe.json').write_text(json.dumps(probe,indent=2)+'\n',encoding='utf8')
    todo=deque((int(x['rva'],16),0)for x in table['entries']);functions={}
    for rva in a.seed_rva:todo.appendleft((rva,0))
    for c in candidates:
        for e in c['entries']:
            if e['vtableOffset']in['0xf8','0x208','0x210','0x258']:todo.append((int(e['targetRva'],16),0))
    while todo and len(functions)<a.max_functions:
        rva,depth=todo.popleft()
        if rva in functions or not pe.code(pe.base+rva):continue
        row=decode(pe,rva,limit=a.seed_byte_budget if rva in a.seed_rva else 4096);functions[rva]=row
        if depth<a.depth:
            for e in row['edges']:
                if e['kind']=='direct-call'or e['kind']=='jump'and not e['insideWindow']:todo.append((int(e['targetRva'],16),depth+1))
    hooks={'version':1,'exeSha256':probe['exe']['sha256'],'decoder':'iced-x86 1.21.0',
        'bridge':'none','bridgeScope':'This diagnostic only; production stopgap status is not measured',
        'nativeBackendEnabled':False,'capabilities':{k:False for k in['observerContext','targetFiltering','cleanFloorNavigation','collisionIsolation','preImpactInteractionGates','projectileIsolation','realmRendering']},
        'scope':'Static function candidates and optional live byte identity. No hooks installed or validated.',
        'coverage':{'functions':len(functions),'maxFunctions':a.max_functions,'callDepth':a.depth,'budgetExhausted':bool(todo),
            'callerAssertedSeedRvas':[hex(x)for x in a.seed_rva]},
        'functions':[functions[x]for x in sorted(functions)],
        'unknowns':['SDK wrapper coverage does not prove stock-engine coverage','Trait candidates are not bound to live receivers','Per-team visibility lacks an observing actor','Native realm behaviour requires disposable-match validation']}
    wrappers={entry['rva']:entry['name']for entry in table['entries']}
    for function in hooks['functions']:
        function['buildIdentityRef']='realm_probe.json#/exe'
        function['sdkWrapperName']=wrappers.get(function['rva'])
        function['callingConventionEvidence']='Windows x64 extern C, installed SDK'if function['rva']in wrappers else'unknown private Rust ABI'
        function['signatureLimit']='Exact prefix bytes, not a relocation mask or a safe trampoline proof'
    (a.out/'realm-hooks.json').write_text(json.dumps(hooks,indent=2)+'\n',encoding='utf8')
    print(json.dumps({'exeSha256':probe['exe']['sha256'],'simTable':table['rva'],'liveBytesMatch':probe['live']['sdkTableMatches']if probe['live']else None,'decodedFunctions':len(functions),'unboundInternalTables':len(candidates),'nativeEnabled':False},indent=2))
if __name__=='__main__':main()
