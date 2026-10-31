"""Close only an exact owned disposable executable, never the installed game.

Keep filesystem redirection loaded until this process exits, then close its
Frida session. A process handle binds identity and termination to one process
even if the numerical PID is later reused.
"""
import argparse,ctypes as c,ctypes.wintypes as w,json
from pathlib import Path

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pid',type=int,required=True)
    p.add_argument('--root',type=Path,required=True);p.add_argument('--report',type=Path,required=True);a=p.parse_args()
    expected=(a.root/'TeamfightManager2.exe').resolve()
    # Require the investigation-specific private directory; refuse broad game roots.
    if a.root.resolve().name!='disposable-game' or a.root.resolve().parent.name!='tfm2-rea':raise ValueError('Not an investigation disposable directory')
    k=c.WinDLL('kernel32',use_last_error=True)
    k.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD];k.OpenProcess.restype=w.HANDLE
    k.QueryFullProcessImageNameW.argtypes=[w.HANDLE,w.DWORD,w.LPWSTR,c.POINTER(w.DWORD)]
    k.TerminateProcess.argtypes=[w.HANDLE,w.UINT];k.WaitForSingleObject.argtypes=[w.HANDLE,w.DWORD]
    k.GetExitCodeProcess.argtypes=[w.HANDLE,c.POINTER(w.DWORD)];k.CloseHandle.argtypes=[w.HANDLE]
    handle=k.OpenProcess(0x1000|0x100000|0x1,False,a.pid)
    if not handle:raise c.WinError(c.get_last_error())
    try:
        path=c.create_unicode_buffer(32768);size=w.DWORD(len(path))
        if not k.QueryFullProcessImageNameW(handle,0,path,c.byref(size)):raise c.WinError(c.get_last_error())
        if Path(path.value).resolve()!=expected:raise ValueError('PID is not the exact disposable executable; refusing termination')
        if not k.TerminateProcess(handle,0):raise c.WinError(c.get_last_error())
        if k.WaitForSingleObject(handle,10000)!=0:raise RuntimeError('Owned process did not exit; retain filesystem isolation')
        code=w.DWORD()
        if not k.GetExitCodeProcess(handle,c.byref(code)):raise c.WinError(c.get_last_error())
        report={'pid':a.pid,'exactDisposablePathVerified':True,'ownedTestProcessExited':True,
            'exitCode':code.value,'installedGameTerminated':False,
            'reason':'Runtime checks pending user permission; close test before releasing filesystem isolation'}
        a.report.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:k.CloseHandle(handle)
if __name__=='__main__':main()
