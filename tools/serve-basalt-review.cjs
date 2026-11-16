// Local read-only art review. No installation or career endpoints.
const http=require('http'),fs=require('fs'),path=require('path');
const root=path.resolve(__dirname,'..'),port=Number(process.argv[2]||7308);
http.createServer((req,res)=>{let p;try{p=decodeURIComponent(new URL(req.url,'http://127.0.0.1').pathname);}catch{res.writeHead(400);return res.end();}
 if(p==='/')p='/editor/basalt-review.html';
 let file=path.resolve(root,'.'+p);if(!file.startsWith(root+path.sep)){res.writeHead(403);return res.end();}
 if(!fs.existsSync(file))file=path.resolve(root,'editor','.'+p);
 if(!file.startsWith(root+path.sep)){res.writeHead(403);return res.end();}
 fs.readFile(file,(e,b)=>{if(e){res.writeHead(404);return res.end('Not found');}const ext=path.extname(file);res.setHeader('Content-Type',({'.html':'text/html','.js':'text/javascript','.json':'application/json','.wasm':'application/wasm','.png':'image/png'})[ext]||'application/octet-stream');res.setHeader('Cache-Control','no-store');res.end(b);});
}).listen(port,'127.0.0.1',()=>console.log('Basalt review: http://localhost:'+port+'/editor/basalt-review.html'));
