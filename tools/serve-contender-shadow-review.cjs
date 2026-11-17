// Read-only local review server. No installation, save or career endpoints.
const http=require('http'),fs=require('fs'),path=require('path');
const root=path.resolve(__dirname,'..'),port=Number(process.argv[2]||7309);
http.createServer((req,res)=>{let pathname;try{pathname=decodeURIComponent(new URL(req.url,'http://127.0.0.1').pathname);}catch{res.writeHead(400);return res.end();}
 if(pathname==='/')pathname='/editor/contender-shadow-review.html';
 let file=path.resolve(root,'.'+pathname);if(!file.startsWith(root+path.sep)){res.writeHead(403);return res.end();}
 if(!fs.existsSync(file))file=path.resolve(root,'editor','.'+pathname);
 if(!file.startsWith(root+path.sep)){res.writeHead(403);return res.end();}
 fs.readFile(file,(e,b)=>{if(e){res.writeHead(404);return res.end('Not found');}res.setHeader('Content-Type',({'.html':'text/html','.js':'text/javascript','.json':'application/json','.wasm':'application/wasm','.png':'image/png','.gif':'image/gif'})[path.extname(file)]||'application/octet-stream');res.setHeader('Cache-Control','no-store');res.end(b);});
}).listen(port,'127.0.0.1',()=>console.log('Contender review: http://localhost:'+port+'/editor/contender-shadow-review.html'));
