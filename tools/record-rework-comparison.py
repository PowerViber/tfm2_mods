"""Record Unified's rank ladder from the native fixture (unified_core_tests::rank_fixtures).

Raw mock-arena results from the shared core, not engine-attributed damage or live win rates.
"""
from pathlib import Path
import json,os,re,subprocess,sys
ROOT=Path(__file__).resolve().parents[1]
def main(seeds=40):
 cmd=['cargo','test','--release','--manifest-path','native/tfm2_custom_ai/Cargo.toml','unified_core_tests::rank_','--','--nocapture']
 if sys.platform=='win32':cmd=['rtk','proxy','rustup','run','stable-x86_64-pc-windows-gnu']+cmd
 p=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,env=dict(os.environ,UNIFIED_SEEDS=str(seeds)))
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 text=p.stdout+p.stderr
 rows=[{'rank':int(r),'name':n.strip(),'damage':int(d),'averageSteps':float(c),'signatures':int(g),'insights':int(i),'efficiency':int(e),'misplays':int(m)}
       for r,n,d,c,g,i,e,m in re.findall(r'rank (\d) (.{14}) damage +(\d+) combo ([\d.]+) signatures +(\d+) insights +(\d+) efficiency (\d+) misplays (\d+)',text)]
 assert len(rows)==8,text
 duel=re.search(r'rank 7 won (\d+)/(\d+)',text)
 report={'scope':__doc__.split('\n\n')[1].strip(),'seeds':seeds,'ranks':rows,'headToHead':{'rank7Wins':int(duel[1]),'games':int(duel[2])}}
 out=ROOT/'docs/rework-results/unified-comparison.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(report,indent=2)+'\n')
 print('Recorded the 8-rank ladder and the rank 7 vs rank 0 duel at',seeds,'seeds')
if __name__=='__main__':main()
