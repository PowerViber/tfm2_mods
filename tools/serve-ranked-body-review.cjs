// Read-only, loopback-only art review server. No game or career write routes.
'use strict';
const http=require('http'),fs=require('fs'),path=require('path');
const root=path.resolve(__dirname,'..'),port=Number(process.argv[2]||7297);
const types={'.html':'text/html; charset=utf-8','.js':'text/javascript; charset=utf-8','.json':'application/json','.fanim':'application/json','.png':'image/png','.gif':'image/gif','.wasm':'application/wasm','.css':'text/css'};
const server=http.createServer((req,res)=>{
  if(!['GET','HEAD'].includes(req.method)){res.writeHead(405,{'Allow':'GET, HEAD'});return res.end();}
  let pathname;try{pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname)}catch{res.writeHead(400);return res.end();}
  if(pathname==='/')pathname='/editor/ranked-body-review.html';
  const file=path.resolve(root,'.'+pathname),relative=path.relative(root,file);
  if(relative.startsWith('..'+path.sep)||path.isAbsolute(relative)||relative==='..'||relative.split(path.sep).includes('.git')){res.writeHead(403);return res.end();}
  fs.readFile(file,(err,body)=>{if(err){res.writeHead(404);return res.end('Not found');}res.writeHead(200,{'Content-Type':types[path.extname(file)]||'application/octet-stream','Cache-Control':'no-store'});res.end(req.method==='HEAD'?undefined:body);});
});
server.listen(port,'127.0.0.1',()=>console.log(`Ranked body review: http://localhost:${port}/editor/ranked-body-review.html`));
