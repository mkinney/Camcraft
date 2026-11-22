# drive.py RUN SEP calls.jsonl out.jsonl -- server cmd...
# Runs MCP tool calls in order; {RUN} in string args -> RUN, '/' inside {RUN}-relative paths -> SEP.
import json, subprocess, sys, time
run, sep, callsf, outf = sys.argv[1:5]
cmd = sys.argv[sys.argv.index('--')+1:]
errf = open(outf + '.stderr', 'w')
p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errf, text=True, encoding='utf-8')
def send(o): p.stdin.write(json.dumps(o)+"\n"); p.stdin.flush()
def recv(i):
    while True:
        l = p.stdout.readline()
        if not l: return {"error": "EOF"}
        try: m = json.loads(l)
        except Exception: continue
        if m.get('id') == i: return m
def sub(o):
    if isinstance(o, str) and '{RUN}' in o:
        rest = o.replace('{RUN}', '')
        return run + rest.replace('/', sep)
    if isinstance(o, dict): return {k: sub(v) for k, v in o.items()}
    if isinstance(o, list): return [sub(v) for v in o]
    return o
send({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qa","version":"1"}}})
recv(0); send({"jsonrpc":"2.0","method":"notifications/initialized"})
out = open(outf, 'w')
n = 0
for line in open(callsf):
    line = line.strip()
    if not line or line.startswith('#'): continue
    n += 1
    c = json.loads(line)
    args = sub(c.get('arguments', {}))
    t = time.time()
    r = recv_r = None
    send({"jsonrpc":"2.0","id":n,"method":"tools/call","params":{"name":c['name'],"arguments":args}})
    r = recv(n)
    res = r.get('result', r)
    txt = "".join(x.get('text','') for x in res.get('content', [])) if isinstance(res, dict) and 'content' in res else json.dumps(res)
    err = bool(isinstance(res, dict) and res.get('isError')) or 'error' in r
    rec = {"n": n, "step": c.get('step',''), "name": c['name'], "args": args, "error": err, "secs": round(time.time()-t,2), "text": txt}
    out.write(json.dumps(rec)+"\n"); out.flush()
    print(f"{n:3} {c.get('step',''):4} {c['name']:24} {'ERR' if err else 'ok '} {txt[:160]!r}", flush=True)
    if r.get('error') == 'EOF': print('SERVER EOF'); break
p.stdin.close(); p.wait(timeout=60)
print('exit', p.returncode)
