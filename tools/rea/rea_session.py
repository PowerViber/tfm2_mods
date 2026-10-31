"""Private REA MCP session for long Windows analysis; no persistent client configuration.

Drop numbered {tool, arguments} JSON requests into QUEUE/requests. Full MCP replies
go to QUEUE/responses. A stop.json sentinel closes the binary and owned server.
"""
import argparse,json,os,queue,shutil,subprocess,threading,time
from pathlib import Path

class Client:
    def __init__(self, server, env, folder):
        self.messages=queue.Queue();self.serial=0;self.active=None;self.folder=folder;self.write_lock=threading.Lock()
        self.errors=(folder/'server-stderr.log').open('w',encoding='utf8')
        self.process=subprocess.Popen([shutil.which('node'),str(server),'--mcp'],env=env,
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.errors,text=True,
            encoding='utf8',bufsize=1)
        def read():
            for line in self.process.stdout:
                try:self.messages.put(json.loads(line))
                except json.JSONDecodeError:pass
            self.messages.put({'closed':self.process.poll()})
        threading.Thread(target=read,daemon=True).start()
        def controls():
            while self.process.poll()is None:
                cancel=folder/'cancel.json'
                if cancel.exists()and self.active is not None:
                    self.send({'jsonrpc':'2.0','method':'notifications/cancelled',
                        'params':{'requestId':self.active,'reason':'Investigation requested cancellation'}})
                    cancel.unlink()
                time.sleep(.25)
        threading.Thread(target=controls,daemon=True).start()
        self.request('initialize',{'protocolVersion':'2025-06-18','capabilities':{},
            'clientInfo':{'name':'tfm2-realm-investigation','version':'1.0'}})
        self.send({'jsonrpc':'2.0','method':'notifications/initialized'})
    def send(self,value):
        with self.write_lock:
            self.process.stdin.write(json.dumps(value)+'\n');self.process.stdin.flush()
    def request(self,method,params,timeout=2400):
        self.serial+=1;identifier=self.serial
        self.active=identifier
        self.send({'jsonrpc':'2.0','id':identifier,'method':method,'params':params})
        end=time.monotonic()+timeout
        while time.monotonic()<end:
            try:value=self.messages.get(timeout=min(30,max(.1,end-time.monotonic())))
            except queue.Empty:
                print('REA request',identifier,'still running',flush=True);continue
            if 'closed'in value:raise RuntimeError('REA server exited: '+str(value))
            if value.get('id')==identifier:
                self.active=None
                if 'error'in value:raise RuntimeError(value['error'])
                return value['result']
        raise TimeoutError('REA request did not complete; inspect owned server before retrying')
    def tool(self,name,args):return self.request('tools/call',{'name':name,'arguments':args})
    def close(self):
        result={}
        try:result['binaryClose']=self.tool('close_binary',{})
        finally:
            self.process.stdin.close()
            try:self.process.wait(timeout=60)
            except subprocess.TimeoutExpired:self.process.terminate();self.process.wait(timeout=30)
            self.errors.close()
            result['serverExitCode']=self.process.returncode
            (self.folder/'cleanup.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf8')

def main():
    p=argparse.ArgumentParser();p.add_argument('--queue',type=Path,required=True)
    p.add_argument('--ghidra',type=Path,required=True);p.add_argument('--server',type=Path,required=True)
    p.add_argument('--java-home',type=Path,required=True);a=p.parse_args()
    a.queue.mkdir(parents=True,exist_ok=True)
    for d in ['requests','responses']:(a.queue/d).mkdir(exist_ok=True)
    env=dict(os.environ,GHIDRA_INSTALL_DIR=str(a.ghidra),JAVA_HOME=str(a.java_home),
        REA_GHIDRA_STARTUP_TIMEOUT_MS='1200000',GHIDRA_HEADLESS_MAXMEM='4G',
        REA_GHIDRA_DISABLED_ANALYZERS='Decompiler Parameter ID,Decompiler Switch Analysis',
        REA_OBJDUMP_COMMAND='C:/msys64/mingw64/bin/objdump.exe')
    client=None
    try:
        client=Client(a.server,env,a.queue);print('Private REA MCP session ready',flush=True)
        while not(a.queue/'stop.json').exists():
            if(a.queue/'halted.json').exists():time.sleep(.25);continue
            pending=[f for f in sorted((a.queue/'requests').glob('*.json'))if not(a.queue/'responses'/f.name).exists()]
            if not pending:time.sleep(.25);continue
            for f in pending:
                request=json.loads(f.read_text(encoding='utf8'));print('START',f.name,request['tool'],flush=True)
                result=client.tool(request['tool'],request.get('arguments',{}))
                (a.queue/'responses'/f.name).write_text(json.dumps(result,indent=2)+'\n',encoding='utf8')
                print('DONE',f.name,'error='+str(result.get('isError',False)),flush=True)
                if result.get('isError',False):
                    (a.queue/'halted.json').write_text(json.dumps({'failedRequest':f.name}))
                    break
    finally:
        if client:client.close();print('Owned REA binary/server closed',flush=True)
if __name__=='__main__':main()
