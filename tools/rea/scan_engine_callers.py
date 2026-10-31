"""Find instruction-decoded direct-call candidates within PE unwind ranges.

Unwind-backed decoding is an anchor, not proof of whole-function control flow;
leaf callers without unwind entries are omitted. Validate candidates in Ghidra.
"""
import argparse,json,time
from pathlib import Path
from iced_x86 import Decoder,FlowControl
from realm_discovery import PE
def scan(pe,targets):
    hits={hex(x):[]for x in targets};count=0;total=0
    for start,end,info in pe.unwind:
        if end-start>1_000_000:continue
        for instruction in Decoder(64,pe.bytes(start,end-start),ip=pe.base+start):
            if instruction.is_invalid:continue
            count+=1
            if instruction.flow_control in[FlowControl.CALL,FlowControl.UNCONDITIONAL_BRANCH]:
                target=instruction.near_branch_target-pe.base
                if target in targets:
                    hits[hex(target)].append({'callerStartRva':hex(start),'callerEndRva':hex(end),
                        'instructionRva':hex(instruction.ip-pe.base),'kind':'call'if instruction.flow_control==FlowControl.CALL else'jump',
                        'ghidraValidated':False})
        total+=1
    return {'unwindRangesExamined':total,'instructionsDecoded':count,'targets':hits,
        'coverage':'Unwind-backed instruction windows only. Inline data and overlapping function bodies require validation; missing leaf callers are not evidence of absence.'}
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--exe',type=Path,required=True);p.add_argument('--out',type=Path,required=True)
    p.add_argument('--targets',default='0x16aa0c0,0x16ad6e0,0x33cb210,0x33d1140');a=p.parse_args();pe=PE(a.exe)
    report=scan(pe,{int(x,16)for x in a.targets.split(',')});report['exeSha256']=pe.identity()['sha256'];a.out.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'ranges':report['unwindRangesExamined'],'instructions':report['instructionsDecoded'],'directCandidates':{k:len(v)for k,v in report['targets'].items()}},indent=2))
if __name__=='__main__':main()
